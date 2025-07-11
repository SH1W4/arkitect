//! # Sistema de Checkpoint & Backup
//!
//! Sistema completo de backup e checkpoint para TaskGraph e dados críticos:
//! - Snapshots periódicos do TaskGraph em MinIO
//! - Checkpoints locais em SQLite a cada N tarefas concluídas
//! - Restauração automática no boot
//! - Gestão de versionamento e recuperação de dados

use chrono::{DateTime, Utc};
use rusoto_core::Region;
use rusoto_s3::{S3Client, S3, PutObjectRequest, GetObjectRequest};
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, Row};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tracing::{info, warn, error, debug};
use uuid::Uuid;

use crate::errors::{OrchestratorError, Result};
use crate::graph::{TaskMesh, TaskId, TaskStatus};
use crate::metrics::SystemMetrics;

/// Configuração do sistema de backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Configuração do MinIO/S3
    pub minio_config: MinioConfig,
    /// Configuração do SQLite local
    pub sqlite_config: SqliteConfig,
    /// Configuração de snapshots
    pub snapshot_config: SnapshotConfig,
    /// Configuração de checkpoints
    pub checkpoint_config: CheckpointConfig,
}

/// Configuração do MinIO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinioConfig {
    pub endpoint: String,
    pub bucket_name: String,
    pub access_key: String,
    pub secret_key: String,
    pub region: String,
}

/// Configuração do SQLite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    pub database_path: PathBuf,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
}

/// Configuração de snapshots
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotConfig {
    /// Intervalo entre snapshots em segundos
    pub interval_seconds: u64,
    /// Número máximo de snapshots a manter
    pub max_snapshots: u32,
    /// Compressão dos snapshots
    pub compression_enabled: bool,
    /// Prefixo dos snapshots no MinIO
    pub snapshot_prefix: String,
}

/// Configuração de checkpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointConfig {
    /// Número de tarefas completadas para trigger de checkpoint
    pub tasks_per_checkpoint: u32,
    /// Retenção de checkpoints em dias
    pub retention_days: u32,
    /// Auto-limpeza de checkpoints antigos
    pub auto_cleanup: bool,
}

/// Dados de um snapshot do TaskGraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraphSnapshot {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub task_graph: TaskMesh,
    pub system_metrics: SystemMetrics,
    pub metadata: SnapshotMetadata,
}

/// Metadados do snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub running_tasks: u32,
    pub compression_ratio: Option<f64>,
    pub size_bytes: u64,
}

/// Dados de um checkpoint local
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCheckpoint {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub task_count: u32,
    pub last_completed_task: Option<TaskId>,
    pub system_state: SystemState,
    pub recovery_data: HashMap<String, serde_json::Value>,
}

/// Estado do sistema para recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemState {
    pub active_tasks: Vec<TaskId>,
    pub pending_tasks: Vec<TaskId>,
    pub failed_tasks: Vec<TaskId>,
    pub resource_usage: HashMap<String, f64>,
    pub configuration_hash: String,
}

/// Resultado de uma operação de backup
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub operation_type: BackupOperationType,
    pub success: bool,
    pub duration_ms: u64,
    pub size_bytes: Option<u64>,
    pub error_message: Option<String>,
}

/// Tipo de operação de backup
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackupOperationType {
    Snapshot,
    Checkpoint,
    Restore,
    Cleanup,
}

/// Sistema principal de backup e checkpoint
pub struct BackupSystem {
    config: BackupConfig,
    minio_client: S3Client,
    sqlite_pool: SqlitePool,
    completed_tasks_count: Arc<std::sync::atomic::AtomicU32>,
    last_snapshot: Arc<tokio::sync::RwLock<Option<DateTime<Utc>>>>,
    last_checkpoint: Arc<tokio::sync::RwLock<Option<DateTime<Utc>>>>,
}

impl BackupSystem {
    /// Cria uma nova instância do sistema de backup
    pub async fn new(config: BackupConfig) -> Result<Self> {
        info!("Inicializando sistema de backup e checkpoint");
        
        // Configurar cliente MinIO
        let minio_client = Self::setup_minio_client(&config.minio_config)?;
        
        // Configurar pool SQLite
        let sqlite_pool = Self::setup_sqlite_pool(&config.sqlite_config).await?;
        
        // Criar tabelas se não existirem
        Self::initialize_database(&sqlite_pool).await?;
        
        Ok(Self {
            config,
            minio_client,
            sqlite_pool,
            completed_tasks_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
            last_snapshot: Arc::new(tokio::sync::RwLock::new(None)),
            last_checkpoint: Arc::new(tokio::sync::RwLock::new(None)),
        })
    }
    
    /// Configura o cliente MinIO
    fn setup_minio_client(config: &MinioConfig) -> Result<S3Client> {
        let region = match config.region.as_str() {
            "us-east-1" => Region::UsEast1,
            "us-west-2" => Region::UsWest2,
            "eu-west-1" => Region::EuWest1,
            custom => Region::Custom {
                name: custom.to_string(),
                endpoint: config.endpoint.clone(),
            },
        };
        
        // Configurar credenciais através de variáveis de ambiente
        std::env::set_var("AWS_ACCESS_KEY_ID", &config.access_key);
        std::env::set_var("AWS_SECRET_ACCESS_KEY", &config.secret_key);
        
        Ok(S3Client::new(region))
    }
    
    /// Configura o pool de conexões SQLite
    async fn setup_sqlite_pool(config: &SqliteConfig) -> Result<SqlitePool> {
        // Criar diretório se não existir
        if let Some(parent) = config.database_path.parent() {
            fs::create_dir_all(parent).await
                .map_err(|e| OrchestratorError::BackupError(format!("Erro ao criar diretório: {}", e)))?;
        }
        
        let database_url = format!("sqlite://{}", config.database_path.display());
        
        let pool = SqlitePool::connect(&database_url).await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao conectar SQLite: {}", e)))?;
        
        Ok(pool)
    }
    
    /// Inicializa as tabelas do banco de dados
    async fn initialize_database(pool: &SqlitePool) -> Result<()> {
        debug!("Inicializando tabelas do banco de dados");
        
        // Tabela de checkpoints
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                task_count INTEGER NOT NULL,
                last_completed_task TEXT,
                system_state TEXT NOT NULL,
                recovery_data TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao criar tabela checkpoints: {}", e)))?;
        
        // Tabela de snapshots (metadados)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshot_metadata (
                id TEXT PRIMARY KEY,
                timestamp TEXT NOT NULL,
                version TEXT NOT NULL,
                minio_key TEXT NOT NULL,
                total_tasks INTEGER NOT NULL,
                completed_tasks INTEGER NOT NULL,
                failed_tasks INTEGER NOT NULL,
                size_bytes INTEGER NOT NULL,
                compression_ratio REAL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao criar tabela snapshot_metadata: {}", e)))?;
        
        // Tabela de operações de backup
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS backup_operations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                operation_type TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                duration_ms INTEGER NOT NULL,
                size_bytes INTEGER,
                error_message TEXT,
                timestamp TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao criar tabela backup_operations: {}", e)))?;
        
        info!("Tabelas do banco de dados inicializadas com sucesso");
        Ok(())
    }
    
    /// Cria um snapshot do TaskGraph e envia para MinIO
    pub async fn create_snapshot(
        &self,
        task_graph: &TaskMesh,
        system_metrics: &SystemMetrics,
    ) -> Result<TaskGraphSnapshot> {
        let start_time = std::time::Instant::now();
        info!("Iniciando criação de snapshot do TaskGraph");
        
        let snapshot_id = Uuid::new_v4();
        let timestamp = Utc::now();
        
        // Calcular metadados
        let metadata = self.calculate_snapshot_metadata(task_graph);
        
        // Criar snapshot
        let snapshot = TaskGraphSnapshot {
            id: snapshot_id,
            timestamp,
            version: crate::VERSION.to_string(),
            task_graph: task_graph.clone(),
            system_metrics: system_metrics.clone(),
            metadata,
        };
        
        // Serializar snapshot
        let snapshot_data = serde_json::to_vec(&snapshot)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao serializar snapshot: {}", e)))?;
        
        // Comprimir se habilitado
        let final_data = if self.config.snapshot_config.compression_enabled {
            self.compress_data(&snapshot_data)?
        } else {
            snapshot_data
        };
        
        // Enviar para MinIO
        let minio_key = format!(
            "{}/snapshot_{}_{}.json{}",
            self.config.snapshot_config.snapshot_prefix,
            timestamp.format("%Y%m%d_%H%M%S"),
            snapshot_id,
            if self.config.snapshot_config.compression_enabled { ".gz" } else { "" }
        );
        
        self.upload_to_minio(&minio_key, final_data.clone()).await?;
        
        // Salvar metadados no SQLite
        self.save_snapshot_metadata(&snapshot, &minio_key, final_data.len() as u64).await?;
        
        // Atualizar última snapshot
        *self.last_snapshot.write().await = Some(timestamp);
        
        // Registrar operação
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.record_backup_operation(BackupResult {
            operation_type: BackupOperationType::Snapshot,
            success: true,
            duration_ms,
            size_bytes: Some(final_data.len() as u64),
            error_message: None,
        }).await?;
        
        info!(
            "Snapshot criado com sucesso: ID={}, tamanho={} bytes, duração={}ms",
            snapshot_id,
            final_data.len(),
            duration_ms
        );
        
        // Limpeza de snapshots antigos
        self.cleanup_old_snapshots().await?;
        
        Ok(snapshot)
    }
    /// Calcula metadados do snapshot
    fn calculate_snapshot_metadata(&self, task_graph: &TaskMesh) -> SnapshotMetadata {
        let total_tasks = task_graph.node_count() as u32;
        let mut completed_tasks = 0;
        let mut failed_tasks = 0;
        let mut running_tasks = 0;
        
        // Contar tarefas por status
        for node_index in task_graph.node_indices() {
            if let Some(task) = task_graph.node_weight(node_index) {
                match task.status {
                    TaskStatus::Completed => completed_tasks += 1,
                    TaskStatus::Failed => failed_tasks += 1,
                    TaskStatus::Running => running_tasks += 1,
                    _ => {}
                }
            }
        }
        
        SnapshotMetadata {
            total_tasks,
            completed_tasks,
            failed_tasks,
            running_tasks,
            compression_ratio: None, // Será calculado após compressão
            size_bytes: 0, // Será atualizado após serialização
        }
    }
    
    /// Comprime dados usando gzip
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use std::io::Write;
        use flate2::{Compression, write::GzEncoder};
        
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro na compressão: {}", e)))?;
        
        encoder.finish()
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao finalizar compressão: {}", e)))
    }
    
    /// Descomprime dados gzip
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        use std::io::Read;
        use flate2::read::GzDecoder;
        
        let mut decoder = GzDecoder::new(data);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro na descompressão: {}", e)))?;
        
        Ok(decompressed)
    }
    
    /// Faz upload de dados para MinIO
    async fn upload_to_minio(&self, key: &str, data: Vec<u8>) -> Result<()> {
        let request = PutObjectRequest {
            bucket: self.config.minio_config.bucket_name.clone(),
            key: key.to_string(),
            body: Some(data.into()),
            content_type: Some("application/json".to_string()),
            ..Default::default()
        };
        
        self.minio_client.put_object(request).await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao enviar para MinIO: {}", e)))?;
        
        debug!("Dados enviados para MinIO com sucesso: {}", key);
        Ok(())
    }
    
    /// Faz download de dados do MinIO
    async fn download_from_minio(&self, key: &str) -> Result<Vec<u8>> {
        let request = GetObjectRequest {
            bucket: self.config.minio_config.bucket_name.clone(),
            key: key.to_string(),
            ..Default::default()
        };
        
        let response = self.minio_client.get_object(request).await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao baixar do MinIO: {}", e)))?;
        
        let mut data = Vec::new();
        if let Some(body) = response.body {
            use tokio::io::AsyncReadExt;
            let mut reader = body.into_async_read();
            reader.read_to_end(&mut data).await
                .map_err(|e| OrchestratorError::BackupError(format!("Erro ao ler dados do MinIO: {}", e)))?;
        }
        
        debug!("Dados baixados do MinIO com sucesso: {}", key);
        Ok(data)
    }
    
    /// Salva metadados do snapshot no SQLite
    async fn save_snapshot_metadata(
        &self,
        snapshot: &TaskGraphSnapshot,
        minio_key: &str,
        size_bytes: u64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO snapshot_metadata (
                id, timestamp, version, minio_key, total_tasks, 
                completed_tasks, failed_tasks, size_bytes, compression_ratio
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(snapshot.id.to_string())
        .bind(snapshot.timestamp.to_rfc3339())
        .bind(&snapshot.version)
        .bind(minio_key)
        .bind(snapshot.metadata.total_tasks as i64)
        .bind(snapshot.metadata.completed_tasks as i64)
        .bind(snapshot.metadata.failed_tasks as i64)
        .bind(size_bytes as i64)
        .bind(snapshot.metadata.compression_ratio)
        .execute(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao salvar metadados: {}", e)))?;
        
        Ok(())
    }
    
    /// Cria um checkpoint local
    pub async fn create_checkpoint(
        &self,
        task_count: u32,
        last_completed_task: Option<TaskId>,
        system_state: SystemState,
        recovery_data: HashMap<String, serde_json::Value>,
    ) -> Result<LocalCheckpoint> {
        let start_time = std::time::Instant::now();
        info!("Iniciando criação de checkpoint local");
        
        let checkpoint_id = Uuid::new_v4();
        let timestamp = Utc::now();
        
        let checkpoint = LocalCheckpoint {
            id: checkpoint_id,
            timestamp,
            task_count,
            last_completed_task,
            system_state,
            recovery_data,
        };
        
        // Serializar dados para salvar no SQLite
        let system_state_json = serde_json::to_string(&checkpoint.system_state)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao serializar system_state: {}", e)))?;
        
        let recovery_data_json = serde_json::to_string(&checkpoint.recovery_data)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao serializar recovery_data: {}", e)))?;
        
        // Salvar checkpoint no SQLite
        sqlx::query(
            r#"
            INSERT INTO checkpoints (
                id, timestamp, task_count, last_completed_task, 
                system_state, recovery_data
            ) VALUES (?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(checkpoint_id.to_string())
        .bind(timestamp.to_rfc3339())
        .bind(task_count as i64)
        .bind(checkpoint.last_completed_task.map(|id| id.to_string()))
        .bind(&system_state_json)
        .bind(&recovery_data_json)
        .execute(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao salvar checkpoint: {}", e)))?;
        
        // Atualizar último checkpoint
        *self.last_checkpoint.write().await = Some(timestamp);
        
        // Registrar operação
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.record_backup_operation(BackupResult {
            operation_type: BackupOperationType::Checkpoint,
            success: true,
            duration_ms,
            size_bytes: Some((system_state_json.len() + recovery_data_json.len()) as u64),
            error_message: None,
        }).await?;
        
        info!(
            "Checkpoint criado com sucesso: ID={}, task_count={}, duração={}ms",
            checkpoint_id,
            task_count,
            duration_ms
        );
        
        // Auto-limpeza se habilitada
        if self.config.checkpoint_config.auto_cleanup {
            self.cleanup_old_checkpoints().await?;
        }
        
        Ok(checkpoint)
    }
    
    /// Notifica conclusão de tarefa para trigger de checkpoint
    pub async fn on_task_completed(&self, task_id: TaskId) -> Result<Option<LocalCheckpoint>> {
        let current_count = self.completed_tasks_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        
        if current_count % self.config.checkpoint_config.tasks_per_checkpoint == 0 {
            info!("Trigger de checkpoint ativado após {} tarefas completadas", current_count);
            
            // Coletar estado atual do sistema
            let system_state = self.collect_system_state().await?;
            let mut recovery_data = HashMap::new();
            recovery_data.insert("trigger_task".to_string(), serde_json::json!(task_id));
            recovery_data.insert("completed_count".to_string(), serde_json::json!(current_count));
            
            let checkpoint = self.create_checkpoint(
                current_count,
                Some(task_id),
                system_state,
                recovery_data,
            ).await?;
            
            return Ok(Some(checkpoint));
        }
        
        Ok(None)
    }
    
    /// Coleta estado atual do sistema
    async fn collect_system_state(&self) -> Result<SystemState> {
        // Esta implementação deveria ser integrada com o sistema real
        // Por agora, retornamos um estado básico
        Ok(SystemState {
            active_tasks: Vec::new(),
            pending_tasks: Vec::new(),
            failed_tasks: Vec::new(),
            resource_usage: HashMap::new(),
            configuration_hash: "placeholder".to_string(),
        })
    }
    
    /// Registra uma operação de backup
    async fn record_backup_operation(&self, result: BackupResult) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO backup_operations (
                operation_type, success, duration_ms, size_bytes, error_message
            ) VALUES (?, ?, ?, ?, ?)
            "#
        )
        .bind(format!("{:?}", result.operation_type))
        .bind(result.success)
        .bind(result.duration_ms as i64)
        .bind(result.size_bytes.map(|s| s as i64))
        .bind(result.error_message)
        .execute(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao registrar operação: {}", e)))?;
        
        Ok(())
    }
    
    /// Limpa snapshots antigos do MinIO
    async fn cleanup_old_snapshots(&self) -> Result<()> {
        let retention_count = self.config.snapshot_config.max_snapshots;
        
        // Buscar snapshots ordenados por timestamp
        let rows = sqlx::query(
            "SELECT id, minio_key FROM snapshot_metadata ORDER BY timestamp DESC LIMIT -1 OFFSET ?"
        )
        .bind(retention_count as i64)
        .fetch_all(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao buscar snapshots antigos: {}", e)))?;
        
        for row in rows {
            let snapshot_id: String = row.get("id");
            let minio_key: String = row.get("minio_key");
            
            // Deletar do MinIO
            if let Err(e) = self.delete_from_minio(&minio_key).await {
                warn!("Erro ao deletar snapshot {} do MinIO: {}", snapshot_id, e);
            }
            
            // Deletar metadados do SQLite
            sqlx::query("DELETE FROM snapshot_metadata WHERE id = ?")
                .bind(&snapshot_id)
                .execute(&self.sqlite_pool)
                .await
                .map_err(|e| OrchestratorError::BackupError(format!("Erro ao deletar metadados: {}", e)))?;
            
            debug!("Snapshot antigo removido: {}", snapshot_id);
        }
        
        Ok(())
    }
    
    /// Limpa checkpoints antigos
    async fn cleanup_old_checkpoints(&self) -> Result<()> {
        let retention_days = self.config.checkpoint_config.retention_days;
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        
        sqlx::query("DELETE FROM checkpoints WHERE timestamp < ?")
            .bind(cutoff_date.to_rfc3339())
            .execute(&self.sqlite_pool)
            .await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao limpar checkpoints: {}", e)))?;
        
        info!("Checkpoints antigos removidos (anteriores a {})", cutoff_date);
        Ok(())
    }
    
    /// Deleta arquivo do MinIO
    async fn delete_from_minio(&self, key: &str) -> Result<()> {
        use rusoto_s3::DeleteObjectRequest;
        
        let request = DeleteObjectRequest {
            bucket: self.config.minio_config.bucket_name.clone(),
            key: key.to_string(),
            ..Default::default()
        };
        
        self.minio_client.delete_object(request).await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao deletar do MinIO: {}", e)))?;
        
        Ok(())
    }
    
    /// Restaura TaskGraph do snapshot mais recente
    pub async fn restore_latest_snapshot(&self) -> Result<Option<TaskGraphSnapshot>> {
        let start_time = std::time::Instant::now();
        info!("Iniciando restauração do snapshot mais recente");
        
        // Buscar snapshot mais recente
        let row = sqlx::query(
            "SELECT id, minio_key, timestamp FROM snapshot_metadata ORDER BY timestamp DESC LIMIT 1"
        )
        .fetch_optional(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao buscar snapshot: {}", e)))?;
        
        let Some(row) = row else {
            info!("Nenhum snapshot encontrado para restauração");
            return Ok(None);
        };
        
        let snapshot_id: String = row.get("id");
        let minio_key: String = row.get("minio_key");
        let timestamp: String = row.get("timestamp");
        
        info!("Restaurando snapshot: ID={}, timestamp={}", snapshot_id, timestamp);
        
        // Baixar dados do MinIO
        let compressed_data = self.download_from_minio(&minio_key).await?;
        
        // Descomprimir se necessário
        let snapshot_data = if minio_key.ends_with(".gz") {
            self.decompress_data(&compressed_data)?
        } else {
            compressed_data
        };
        
        // Deserializar snapshot
        let snapshot: TaskGraphSnapshot = serde_json::from_slice(&snapshot_data)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao deserializar snapshot: {}", e)))?;
        
        // Registrar operação de restauração
        let duration_ms = start_time.elapsed().as_millis() as u64;
        self.record_backup_operation(BackupResult {
            operation_type: BackupOperationType::Restore,
            success: true,
            duration_ms,
            size_bytes: Some(snapshot_data.len() as u64),
            error_message: None,
        }).await?;
        
        info!(
            "Snapshot restaurado com sucesso: ID={}, duração={}ms",
            snapshot.id,
            duration_ms
        );
        
        Ok(Some(snapshot))
    }
    
    /// Restaura checkpoint mais recente
    pub async fn restore_latest_checkpoint(&self) -> Result<Option<LocalCheckpoint>> {
        let start_time = std::time::Instant::now();
        info!("Iniciando restauração do checkpoint mais recente");
        
        let row = sqlx::query(
            "SELECT * FROM checkpoints ORDER BY timestamp DESC LIMIT 1"
        )
        .fetch_optional(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao buscar checkpoint: {}", e)))?;
        
        let Some(row) = row else {
            info!("Nenhum checkpoint encontrado para restauração");
            return Ok(None);
        };
        
        // Extrair dados do checkpoint
        let id: String = row.get("id");
        let timestamp: String = row.get("timestamp");
        let task_count: i64 = row.get("task_count");
        let last_completed_task: Option<String> = row.get("last_completed_task");
        let system_state_json: String = row.get("system_state");
        let recovery_data_json: String = row.get("recovery_data");
        
        // Deserializar dados
        let system_state: SystemState = serde_json::from_str(&system_state_json)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao deserializar system_state: {}", e)))?;
        
        let recovery_data: HashMap<String, serde_json::Value> = serde_json::from_str(&recovery_data_json)
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao deserializar recovery_data: {}", e)))?;
        
        let checkpoint = LocalCheckpoint {
            id: Uuid::parse_str(&id)
                .map_err(|e| OrchestratorError::BackupError(format!("ID inválido: {}", e)))?,
            timestamp: DateTime::parse_from_rfc3339(&timestamp)
                .map_err(|e| OrchestratorError::BackupError(format!("Timestamp inválido: {}", e)))?
                .with_timezone(&Utc),
            task_count: task_count as u32,
            last_completed_task: last_completed_task
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| OrchestratorError::BackupError(format!("TaskId inválido: {}", e)))?,
            system_state,
            recovery_data,
        };
        
        // Atualizar contador de tarefas completadas
        self.completed_tasks_count.store(checkpoint.task_count, std::sync::atomic::Ordering::SeqCst);
        
        let duration_ms = start_time.elapsed().as_millis() as u64;
        
        info!(
            "Checkpoint restaurado com sucesso: ID={}, task_count={}, duração={}ms",
            checkpoint.id,
            checkpoint.task_count,
            duration_ms
        );
        
        Ok(Some(checkpoint))
    }
    
    /// Restauração automática no boot
    pub async fn auto_restore_on_boot(&self) -> Result<(Option<TaskGraphSnapshot>, Option<LocalCheckpoint>)> {
        info!("Iniciando restauração automática no boot");
        
        let snapshot = self.restore_latest_snapshot().await
            .unwrap_or_else(|e| {
                warn!("Erro ao restaurar snapshot: {}", e);
                None
            });
        
        let checkpoint = self.restore_latest_checkpoint().await
            .unwrap_or_else(|e| {
                warn!("Erro ao restaurar checkpoint: {}", e);
                None
            });
        
        match (&snapshot, &checkpoint) {
            (Some(s), Some(c)) => {
                info!("Restauração completa: snapshot {} e checkpoint {}", s.id, c.id);
            }
            (Some(s), None) => {
                info!("Restaurado apenas snapshot: {}", s.id);
            }
            (None, Some(c)) => {
                info!("Restaurado apenas checkpoint: {}", c.id);
            }
            (None, None) => {
                info!("Nenhum backup encontrado para restauração");
            }
        }
        
        Ok((snapshot, checkpoint))
    }
    
    /// Inicia task periódica de snapshots
    pub fn start_periodic_snapshots(
        &self,
        task_graph: Arc<tokio::sync::RwLock<TaskMesh>>,
        system_metrics: Arc<tokio::sync::RwLock<SystemMetrics>>,
    ) {
        let backup_system = Arc::new(self);
        let interval = self.config.snapshot_config.interval_seconds;
        
        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(interval));
            
            loop {
                interval_timer.tick().await;
                
                let graph = task_graph.read().await.clone();
                let metrics = system_metrics.read().await.clone();
                
                if let Err(e) = backup_system.create_snapshot(&graph, &metrics).await {
                    error!("Erro no snapshot periódico: {}", e);
                }
            }
        });
        
        info!("Task periódica de snapshots iniciada (intervalo: {}s)", interval);
    }
    
    /// Estatísticas do sistema de backup
    pub async fn get_backup_stats(&self) -> Result<BackupStats> {
        let snapshot_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM snapshot_metadata")
            .fetch_one(&self.sqlite_pool)
            .await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao contar snapshots: {}", e)))?;
        
        let checkpoint_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM checkpoints")
            .fetch_one(&self.sqlite_pool)
            .await
            .map_err(|e| OrchestratorError::BackupError(format!("Erro ao contar checkpoints: {}", e)))?;
        
        let total_size = sqlx::query_scalar::<_, Option<i64>>(
            "SELECT SUM(size_bytes) FROM snapshot_metadata"
        )
        .fetch_one(&self.sqlite_pool)
        .await
        .map_err(|e| OrchestratorError::BackupError(format!("Erro ao calcular tamanho: {}", e)))?;
        
        let last_snapshot_time = *self.last_snapshot.read().await;
        let last_checkpoint_time = *self.last_checkpoint.read().await;
        
        Ok(BackupStats {
            snapshot_count: snapshot_count as u32,
            checkpoint_count: checkpoint_count as u32,
            total_size_bytes: total_size.unwrap_or(0) as u64,
            last_snapshot_time,
            last_checkpoint_time,
            completed_tasks_count: self.completed_tasks_count.load(std::sync::atomic::Ordering::SeqCst),
        })
    }
}

/// Estatísticas do sistema de backup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupStats {
    pub snapshot_count: u32,
    pub checkpoint_count: u32,
    pub total_size_bytes: u64,
    pub last_snapshot_time: Option<DateTime<Utc>>,
    pub last_checkpoint_time: Option<DateTime<Utc>>,
    pub completed_tasks_count: u32,
}


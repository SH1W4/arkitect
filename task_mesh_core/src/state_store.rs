//! Armazenamento de estado com suporte a SQLite e Redis

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use async_trait::async_trait;
use serde_json;
use sqlx::{Database, Pool, Row, SqlitePool, PgPool};
use redis::{AsyncCommands, Client as RedisClient, aio::Connection as RedisConnection};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, instrument};

use crate::types::*;
use crate::TaskMeshResult;

/// Trait para armazenamento de estado
#[async_trait]
pub trait StateStore: Send + Sync {
    /// Armazena uma tarefa
    async fn store_task(&self, task: &Task) -> TaskMeshResult<()>;
    
    /// Recupera uma tarefa por ID
    async fn get_task(&self, task_id: &TaskId) -> TaskMeshResult<Option<Task>>;
    
    /// Remove uma tarefa
    async fn remove_task(&self, task_id: &TaskId) -> TaskMeshResult<()>;
    
    /// Atualiza status de uma tarefa
    async fn update_task_status(&self, task_id: &TaskId, status: TaskStatus) -> TaskMeshResult<()>;
    
    /// Recupera status de uma tarefa
    async fn get_task_status(&self, task_id: &TaskId) -> TaskMeshResult<TaskStatus>;
    
    /// Lista todas as tarefas
    async fn list_tasks(&self) -> TaskMeshResult<Vec<Task>>;
    
    /// Lista tarefas com status específico
    async fn list_tasks_by_status(&self, status_filter: &[TaskStatus]) -> TaskMeshResult<Vec<Task>>;
    
    /// Armazena evento do sistema
    async fn store_event(&self, event: &SystemEvent) -> TaskMeshResult<()>;
    
    /// Recupera eventos por período
    async fn get_events(
        &self, 
        start_time: Option<SystemTime>, 
        end_time: Option<SystemTime>
    ) -> TaskMeshResult<Vec<SystemEvent>>;
    
    /// Armazena métricas de execução
    async fn store_metrics(&self, task_id: &TaskId, metrics: &ExecutionMetrics) -> TaskMeshResult<()>;
    
    /// Recupera métricas de uma tarefa
    async fn get_metrics(&self, task_id: &TaskId) -> TaskMeshResult<Option<ExecutionMetrics>>;
    
    /// Cria checkpoint do estado
    async fn create_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()>;
    
    /// Restaura estado a partir de checkpoint
    async fn restore_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()>;
    
    /// Lista checkpoints disponíveis
    async fn list_checkpoints(&self) -> TaskMeshResult<Vec<String>>;
    
    /// Limpa dados antigos
    async fn cleanup_old_data(&self, retention_days: u32) -> TaskMeshResult<()>;
}

/// Backend de armazenamento
#[derive(Debug, Clone)]
pub enum StorageBackend {
    SQLite(String),
    PostgreSQL(String),
    Redis(String),
    Memory,
}

/// Implementação com SQLite
pub struct SqliteStateStore {
    pool: SqlitePool,
}

/// Implementação com PostgreSQL
pub struct PostgresStateStore {
    pool: PgPool,
}

/// Implementação com Redis
pub struct RedisStateStore {
    client: RedisClient,
    connection: Arc<RwLock<RedisConnection>>,
}

/// Implementação em memória (para testes)
pub struct MemoryStateStore {
    tasks: Arc<RwLock<HashMap<TaskId, Task>>>,
    task_status: Arc<RwLock<HashMap<TaskId, TaskStatus>>>,
    events: Arc<RwLock<Vec<SystemEvent>>>,
    metrics: Arc<RwLock<HashMap<TaskId, ExecutionMetrics>>>,
    checkpoints: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl SqliteStateStore {
    /// Cria uma nova instância SQLite
    pub async fn new(database_url: &str) -> TaskMeshResult<Self> {
        info!("Conectando ao SQLite: {}", database_url);
        
        let pool = SqlitePool::connect(database_url).await?;
        
        let store = Self { pool };
        store.initialize_schema().await?;
        
        Ok(store)
    }
    
    /// Inicializa schema do banco
    async fn initialize_schema(&self) -> TaskMeshResult<()> {
        debug!("Inicializando schema SQLite");
        
        // Tabela de tarefas
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                definition TEXT NOT NULL,
                dependencies TEXT NOT NULL,
                priority INTEGER NOT NULL,
                metadata TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                timeout_ms INTEGER,
                max_retries INTEGER NOT NULL,
                tags TEXT NOT NULL
            )
            "#
        ).execute(&self.pool).await?;
        
        // Tabela de status
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS task_status (
                task_id TEXT PRIMARY KEY,
                status_type TEXT NOT NULL,
                status_data TEXT NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks (id)
            )
            "#
        ).execute(&self.pool).await?;
        
        // Tabela de eventos
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                event_type TEXT NOT NULL,
                task_id TEXT,
                data TEXT NOT NULL
            )
            "#
        ).execute(&self.pool).await?;
        
        // Tabela de métricas
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                task_id TEXT PRIMARY KEY,
                execution_time_ms INTEGER NOT NULL,
                cpu_usage REAL NOT NULL,
                memory_usage INTEGER NOT NULL,
                network_io_read INTEGER NOT NULL,
                network_io_write INTEGER NOT NULL,
                disk_io_read INTEGER NOT NULL,
                disk_io_write INTEGER NOT NULL,
                recorded_at INTEGER NOT NULL,
                FOREIGN KEY (task_id) REFERENCES tasks (id)
            )
            "#
        ).execute(&self.pool).await?;
        
        // Tabela de checkpoints
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS checkpoints (
                id TEXT PRIMARY KEY,
                data BLOB NOT NULL,
                created_at INTEGER NOT NULL
            )
            "#
        ).execute(&self.pool).await?;
        
        info!("Schema SQLite inicializado");
        Ok(())
    }
}

#[async_trait]
impl StateStore for SqliteStateStore {
    #[instrument(skip(self, task))]
    async fn store_task(&self, task: &Task) -> TaskMeshResult<()> {
        debug!("Armazenando tarefa: {}", task.id);
        
        let definition = serde_json::to_string(&task.definition)?;
        let dependencies = serde_json::to_string(&task.dependencies)?;
        let metadata = serde_json::to_string(&task.metadata)?;
        let tags = serde_json::to_string(&task.tags)?;
        let created_at = task.created_at.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        let timeout_ms = task.timeout.map(|t| t.as_millis() as i64);
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO tasks 
            (id, name, definition, dependencies, priority, metadata, created_at, timeout_ms, max_retries, tags)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(task.id.to_string())
        .bind(&task.name)
        .bind(definition)
        .bind(dependencies)
        .bind(task.priority as i32)
        .bind(metadata)
        .bind(created_at)
        .bind(timeout_ms)
        .bind(task.max_retries as i32)
        .bind(tags)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_task(&self, task_id: &TaskId) -> TaskMeshResult<Option<Task>> {
        debug!("Recuperando tarefa: {}", task_id);
        
        let row = sqlx::query(
            "SELECT * FROM tasks WHERE id = ?"
        )
        .bind(task_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let task = self.row_to_task(row)?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
    
    async fn remove_task(&self, task_id: &TaskId) -> TaskMeshResult<()> {
        debug!("Removendo tarefa: {}", task_id);
        
        sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await?;
        
        sqlx::query("DELETE FROM task_status WHERE task_id = ?")
            .bind(task_id.to_string())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn update_task_status(&self, task_id: &TaskId, status: TaskStatus) -> TaskMeshResult<()> {
        debug!("Atualizando status da tarefa {}: {:?}", task_id, status);
        
        let status_type = self.status_to_type(&status);
        let status_data = serde_json::to_string(&status)?;
        let updated_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO task_status 
            (task_id, status_type, status_data, updated_at)
            VALUES (?, ?, ?, ?)
            "#
        )
        .bind(task_id.to_string())
        .bind(status_type)
        .bind(status_data)
        .bind(updated_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_task_status(&self, task_id: &TaskId) -> TaskMeshResult<TaskStatus> {
        debug!("Recuperando status da tarefa: {}", task_id);
        
        let row = sqlx::query(
            "SELECT status_data FROM task_status WHERE task_id = ?"
        )
        .bind(task_id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let status_data: String = row.try_get("status_data")?;
            let status: TaskStatus = serde_json::from_str(&status_data)?;
            Ok(status)
        } else {
            Ok(TaskStatus::Pending)
        }
    }
    
    async fn list_tasks(&self) -> TaskMeshResult<Vec<Task>> {
        debug!("Listando todas as tarefas");
        
        let rows = sqlx::query("SELECT * FROM tasks ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(self.row_to_task(row)?);
        }
        
        Ok(tasks)
    }
    
    async fn list_tasks_by_status(&self, status_filter: &[TaskStatus]) -> TaskMeshResult<Vec<Task>> {
        debug!("Listando tarefas por status: {:?}", status_filter);
        
        let status_types: Vec<String> = status_filter.iter()
            .map(|s| self.status_to_type(s))
            .collect();
        
        if status_types.is_empty() {
            return Ok(Vec::new());
        }
        
        let placeholders = status_types.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = format!(
            "SELECT t.* FROM tasks t JOIN task_status ts ON t.id = ts.task_id WHERE ts.status_type IN ({})",
            placeholders
        );
        
        let mut query_builder = sqlx::query(&query);
        for status_type in status_types {
            query_builder = query_builder.bind(status_type);
        }
        
        let rows = query_builder.fetch_all(&self.pool).await?;
        
        let mut tasks = Vec::new();
        for row in rows {
            tasks.push(self.row_to_task(row)?);
        }
        
        Ok(tasks)
    }
    
    async fn store_event(&self, event: &SystemEvent) -> TaskMeshResult<()> {
        debug!("Armazenando evento: {:?}", event.event_type);
        
        let timestamp = event.timestamp.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        let event_type = format!("{:?}", event.event_type);
        let task_id = event.task_id.map(|id| id.to_string());
        let data = serde_json::to_string(&event.data)?;
        
        sqlx::query(
            "INSERT INTO events (timestamp, event_type, task_id, data) VALUES (?, ?, ?, ?)"
        )
        .bind(timestamp)
        .bind(event_type)
        .bind(task_id)
        .bind(data)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_events(
        &self, 
        start_time: Option<SystemTime>, 
        end_time: Option<SystemTime>
    ) -> TaskMeshResult<Vec<SystemEvent>> {
        debug!("Recuperando eventos");
        
        let start_ts = start_time.map(|t| 
            t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs() as i64
        ).unwrap_or(0);
        
        let end_ts = end_time.map(|t| 
            t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs() as i64
        ).unwrap_or(i64::MAX);
        
        let rows = sqlx::query(
            "SELECT * FROM events WHERE timestamp >= ? AND timestamp <= ? ORDER BY timestamp DESC"
        )
        .bind(start_ts)
        .bind(end_ts)
        .fetch_all(&self.pool)
        .await?;
        
        let mut events = Vec::new();
        for row in rows {
            events.push(self.row_to_event(row)?);
        }
        
        Ok(events)
    }
    
    async fn store_metrics(&self, task_id: &TaskId, metrics: &ExecutionMetrics) -> TaskMeshResult<()> {
        debug!("Armazenando métricas da tarefa: {}", task_id);
        
        let recorded_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO metrics 
            (task_id, execution_time_ms, cpu_usage, memory_usage, 
             network_io_read, network_io_write, disk_io_read, disk_io_write, recorded_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(task_id.to_string())
        .bind(metrics.execution_time.as_millis() as i64)
        .bind(metrics.cpu_usage)
        .bind(metrics.memory_usage as i64)
        .bind(metrics.network_io.0 as i64)
        .bind(metrics.network_io.1 as i64)
        .bind(metrics.disk_io.0 as i64)
        .bind(metrics.disk_io.1 as i64)
        .bind(recorded_at)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    async fn get_metrics(&self, task_id: &TaskId) -> TaskMeshResult<Option<ExecutionMetrics>> {
        debug!("Recuperando métricas da tarefa: {}", task_id);
        
        let row = sqlx::query("SELECT * FROM metrics WHERE task_id = ?")
            .bind(task_id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            let metrics = self.row_to_metrics(row)?;
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }
    
    async fn create_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()> {
        debug!("Criando checkpoint: {}", checkpoint_id);
        
        // Serializar estado completo
        let tasks = self.list_tasks().await?;
        let checkpoint_data = CheckpointData {
            tasks,
            created_at: SystemTime::now(),
        };
        
        let data = bincode::serialize(&checkpoint_data)
            .map_err(|e| TaskMeshError::Internal(format!("Erro de serialização: {}", e)))?;
        
        let created_at = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        
        sqlx::query(
            "INSERT OR REPLACE INTO checkpoints (id, data, created_at) VALUES (?, ?, ?)"
        )
        .bind(checkpoint_id)
        .bind(data)
        .bind(created_at)
        .execute(&self.pool)
        .await?;
        
        info!("Checkpoint {} criado", checkpoint_id);
        Ok(())
    }
    
    async fn restore_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()> {
        debug!("Restaurando checkpoint: {}", checkpoint_id);
        
        let row = sqlx::query("SELECT data FROM checkpoints WHERE id = ?")
            .bind(checkpoint_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            let data: Vec<u8> = row.try_get("data")?;
            let checkpoint_data: CheckpointData = bincode::deserialize(&data)
                .map_err(|e| TaskMeshError::Internal(format!("Erro de desserialização: {}", e)))?;
            
            // Limpar estado atual
            sqlx::query("DELETE FROM tasks").execute(&self.pool).await?;
            sqlx::query("DELETE FROM task_status").execute(&self.pool).await?;
            
            // Restaurar tarefas
            for task in checkpoint_data.tasks {
                self.store_task(&task).await?;
            }
            
            info!("Checkpoint {} restaurado", checkpoint_id);
            Ok(())
        } else {
            Err(TaskMeshError::CheckpointNotFound(checkpoint_id.to_string()))
        }
    }
    
    async fn list_checkpoints(&self) -> TaskMeshResult<Vec<String>> {
        debug!("Listando checkpoints");
        
        let rows = sqlx::query("SELECT id FROM checkpoints ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut checkpoints = Vec::new();
        for row in rows {
            let id: String = row.try_get("id")?;
            checkpoints.push(id);
        }
        
        Ok(checkpoints)
    }
    
    async fn cleanup_old_data(&self, retention_days: u32) -> TaskMeshResult<()> {
        debug!("Limpando dados antigos (retenção: {} dias)", retention_days);
        
        let cutoff_time = SystemTime::now() - 
            std::time::Duration::from_secs(retention_days as u64 * 24 * 60 * 60);
        let cutoff_timestamp = cutoff_time.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_secs() as i64;
        
        // Limpar eventos antigos
        let deleted_events = sqlx::query("DELETE FROM events WHERE timestamp < ?")
            .bind(cutoff_timestamp)
            .execute(&self.pool)
            .await?
            .rows_affected();
        
        // Limpar checkpoints antigos (manter apenas os 10 mais recentes)
        sqlx::query(
            r#"
            DELETE FROM checkpoints 
            WHERE id NOT IN (
                SELECT id FROM checkpoints 
                ORDER BY created_at DESC 
                LIMIT 10
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        info!("Limpeza concluída: {} eventos removidos", deleted_events);
        Ok(())
    }
}

impl SqliteStateStore {
    /// Converte linha SQL para Task
    fn row_to_task(&self, row: sqlx::sqlite::SqliteRow) -> TaskMeshResult<Task> {
        use sqlx::Row;
        
        let id: String = row.try_get("id")?;
        let name: String = row.try_get("name")?;
        let definition_str: String = row.try_get("definition")?;
        let dependencies_str: String = row.try_get("dependencies")?;
        let priority: i32 = row.try_get("priority")?;
        let metadata_str: String = row.try_get("metadata")?;
        let created_at_secs: i64 = row.try_get("created_at")?;
        let timeout_ms: Option<i64> = row.try_get("timeout_ms")?;
        let max_retries: i32 = row.try_get("max_retries")?;
        let tags_str: String = row.try_get("tags")?;
        
        let task_id = uuid::Uuid::parse_str(&id)
            .map_err(|e| TaskMeshError::Internal(format!("UUID inválido: {}", e)))?;
        
        let definition: TaskDefinition = serde_json::from_str(&definition_str)?;
        let dependencies: Vec<TaskId> = serde_json::from_str(&dependencies_str)?;
        let metadata: HashMap<String, String> = serde_json::from_str(&metadata_str)?;
        let tags: Vec<String> = serde_json::from_str(&tags_str)?;
        
        let created_at = SystemTime::UNIX_EPOCH + 
            std::time::Duration::from_secs(created_at_secs as u64);
        
        let timeout = timeout_ms.map(|ms| std::time::Duration::from_millis(ms as u64));
        
        Ok(Task {
            id: task_id,
            name,
            definition,
            dependencies,
            priority: priority as u8,
            metadata,
            created_at,
            timeout,
            max_retries: max_retries as u32,
            tags,
        })
    }
    
    /// Converte linha SQL para SystemEvent
    fn row_to_event(&self, row: sqlx::sqlite::SqliteRow) -> TaskMeshResult<SystemEvent> {
        use sqlx::Row;
        
        let timestamp_secs: i64 = row.try_get("timestamp")?;
        let event_type_str: String = row.try_get("event_type")?;
        let task_id_str: Option<String> = row.try_get("task_id")?;
        let data_str: String = row.try_get("data")?;
        
        let timestamp = SystemTime::UNIX_EPOCH + 
            std::time::Duration::from_secs(timestamp_secs as u64);
        
        let event_type = match event_type_str.as_str() {
            "TaskSubmitted" => EventType::TaskSubmitted,
            "TaskScheduled" => EventType::TaskScheduled,
            "TaskStarted" => EventType::TaskStarted,
            "TaskCompleted" => EventType::TaskCompleted,
            "TaskFailed" => EventType::TaskFailed,
            "TaskCancelled" => EventType::TaskCancelled,
            _ => EventType::SystemStarted, // Fallback
        };
        
        let task_id = if let Some(id_str) = task_id_str {
            Some(uuid::Uuid::parse_str(&id_str)
                .map_err(|e| TaskMeshError::Internal(format!("UUID inválido: {}", e)))?)
        } else {
            None
        };
        
        let data: serde_json::Value = serde_json::from_str(&data_str)?;
        
        Ok(SystemEvent {
            timestamp,
            event_type,
            task_id,
            data,
        })
    }
    
    /// Converte linha SQL para ExecutionMetrics
    fn row_to_metrics(&self, row: sqlx::sqlite::SqliteRow) -> TaskMeshResult<ExecutionMetrics> {
        use sqlx::Row;
        
        let execution_time_ms: i64 = row.try_get("execution_time_ms")?;
        let cpu_usage: f64 = row.try_get("cpu_usage")?;
        let memory_usage: i64 = row.try_get("memory_usage")?;
        let network_io_read: i64 = row.try_get("network_io_read")?;
        let network_io_write: i64 = row.try_get("network_io_write")?;
        let disk_io_read: i64 = row.try_get("disk_io_read")?;
        let disk_io_write: i64 = row.try_get("disk_io_write")?;
        
        Ok(ExecutionMetrics {
            execution_time: std::time::Duration::from_millis(execution_time_ms as u64),
            cpu_usage,
            memory_usage: memory_usage as u64,
            network_io: (network_io_read as u64, network_io_write as u64),
            disk_io: (disk_io_read as u64, disk_io_write as u64),
        })
    }
    
    /// Converte TaskStatus para string
    fn status_to_type(&self, status: &TaskStatus) -> String {
        match status {
            TaskStatus::Pending => "Pending".to_string(),
            TaskStatus::Scheduled => "Scheduled".to_string(),
            TaskStatus::Running { .. } => "Running".to_string(),
            TaskStatus::Completed { .. } => "Completed".to_string(),
            TaskStatus::Failed { .. } => "Failed".to_string(),
            TaskStatus::Cancelled { .. } => "Cancelled".to_string(),
            TaskStatus::Paused { .. } => "Paused".to_string(),
        }
    }
}

/// Implementação PostgreSQL (similar ao SQLite, mas com sintaxe PostgreSQL)
impl PostgresStateStore {
    pub async fn new(database_url: &str) -> TaskMeshResult<Self> {
        info!("Conectando ao PostgreSQL: {}", database_url);
        
        let pool = PgPool::connect(database_url).await?;
        
        let store = Self { pool };
        store.initialize_schema().await?;
        
        Ok(store)
    }
    
    async fn initialize_schema(&self) -> TaskMeshResult<()> {
        debug!("Inicializando schema PostgreSQL");
        
        // Implementação similar ao SQLite, mas com sintaxe PostgreSQL
        // TODO: Implementar schema PostgreSQL completo
        
        Ok(())
    }
}

// Implementação StateStore para PostgreSQL seria similar ao SQLite
// Por brevidade, não implementando completa aqui

/// Implementação Redis
impl RedisStateStore {
    pub async fn new(redis_url: &str) -> TaskMeshResult<Self> {
        info!("Conectando ao Redis: {}", redis_url);
        
        let client = RedisClient::open(redis_url)
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        let connection = client.get_async_connection().await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(Self {
            client,
            connection: Arc::new(RwLock::new(connection)),
        })
    }
}

#[async_trait]
impl StateStore for RedisStateStore {
    async fn store_task(&self, task: &Task) -> TaskMeshResult<()> {
        debug!("Armazenando tarefa no Redis: {}", task.id);
        
        let mut conn = self.connection.write().await;
        let task_json = serde_json::to_string(task)?;
        let key = format!("task:{}", task.id);
        
        conn.set(&key, task_json).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        // Adicionar ao índice de tarefas
        conn.sadd("tasks:all", task.id.to_string()).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(())
    }
    
    async fn get_task(&self, task_id: &TaskId) -> TaskMeshResult<Option<Task>> {
        debug!("Recuperando tarefa do Redis: {}", task_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("task:{}", task_id);
        
        let task_json: Option<String> = conn.get(&key).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        if let Some(json) = task_json {
            let task: Task = serde_json::from_str(&json)?;
            Ok(Some(task))
        } else {
            Ok(None)
        }
    }
    
    async fn remove_task(&self, task_id: &TaskId) -> TaskMeshResult<()> {
        debug!("Removendo tarefa do Redis: {}", task_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("task:{}", task_id);
        let status_key = format!("status:{}", task_id);
        
        conn.del(&key).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        conn.del(&status_key).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        conn.srem("tasks:all", task_id.to_string()).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(())
    }
    
    async fn update_task_status(&self, task_id: &TaskId, status: TaskStatus) -> TaskMeshResult<()> {
        debug!("Atualizando status no Redis: {}", task_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("status:{}", task_id);
        let status_json = serde_json::to_string(&status)?;
        
        conn.set(&key, status_json).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(())
    }
    
    async fn get_task_status(&self, task_id: &TaskId) -> TaskMeshResult<TaskStatus> {
        debug!("Recuperando status do Redis: {}", task_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("status:{}", task_id);
        
        let status_json: Option<String> = conn.get(&key).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        if let Some(json) = status_json {
            let status: TaskStatus = serde_json::from_str(&json)?;
            Ok(status)
        } else {
            Ok(TaskStatus::Pending)
        }
    }
    
    async fn list_tasks(&self) -> TaskMeshResult<Vec<Task>> {
        debug!("Listando tarefas do Redis");
        
        let mut conn = self.connection.write().await;
        let task_ids: Vec<String> = conn.smembers("tasks:all").await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        let mut tasks = Vec::new();
        for task_id_str in task_ids {
            if let Ok(task_id) = uuid::Uuid::parse_str(&task_id_str) {
                if let Ok(Some(task)) = self.get_task(&task_id).await {
                    tasks.push(task);
                }
            }
        }
        
        Ok(tasks)
    }
    
    async fn list_tasks_by_status(&self, _status_filter: &[TaskStatus]) -> TaskMeshResult<Vec<Task>> {
        // TODO: Implementar filtragem por status no Redis
        self.list_tasks().await
    }
    
    async fn store_event(&self, event: &SystemEvent) -> TaskMeshResult<()> {
        debug!("Armazenando evento no Redis: {:?}", event.event_type);
        
        let mut conn = self.connection.write().await;
        let event_json = serde_json::to_string(event)?;
        let timestamp = event.timestamp.duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default().as_millis();
        
        conn.zadd("events", event_json, timestamp as f64).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(())
    }
    
    async fn get_events(
        &self, 
        start_time: Option<SystemTime>, 
        end_time: Option<SystemTime>
    ) -> TaskMeshResult<Vec<SystemEvent>> {
        debug!("Recuperando eventos do Redis");
        
        let mut conn = self.connection.write().await;
        
        let start_ts = start_time.map(|t| 
            t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as f64
        ).unwrap_or(0.0);
        
        let end_ts = end_time.map(|t| 
            t.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis() as f64
        ).unwrap_or(f64::MAX);
        
        let event_jsons: Vec<String> = conn.zrangebyscore("events", start_ts, end_ts).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        let mut events = Vec::new();
        for json in event_jsons {
            if let Ok(event) = serde_json::from_str::<SystemEvent>(&json) {
                events.push(event);
            }
        }
        
        Ok(events)
    }
    
    async fn store_metrics(&self, task_id: &TaskId, metrics: &ExecutionMetrics) -> TaskMeshResult<()> {
        debug!("Armazenando métricas no Redis: {}", task_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("metrics:{}", task_id);
        let metrics_json = serde_json::to_string(metrics)?;
        
        conn.set(&key, metrics_json).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(())
    }
    
    async fn get_metrics(&self, task_id: &TaskId) -> TaskMeshResult<Option<ExecutionMetrics>> {
        debug!("Recuperando métricas do Redis: {}", task_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("metrics:{}", task_id);
        
        let metrics_json: Option<String> = conn.get(&key).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        if let Some(json) = metrics_json {
            let metrics: ExecutionMetrics = serde_json::from_str(&json)?;
            Ok(Some(metrics))
        } else {
            Ok(None)
        }
    }
    
    async fn create_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()> {
        debug!("Criando checkpoint no Redis: {}", checkpoint_id);
        
        let tasks = self.list_tasks().await?;
        let checkpoint_data = CheckpointData {
            tasks,
            created_at: SystemTime::now(),
        };
        
        let mut conn = self.connection.write().await;
        let key = format!("checkpoint:{}", checkpoint_id);
        let data = serde_json::to_string(&checkpoint_data)?;
        
        conn.set(&key, data).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        conn.sadd("checkpoints:all", checkpoint_id).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(())
    }
    
    async fn restore_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()> {
        debug!("Restaurando checkpoint do Redis: {}", checkpoint_id);
        
        let mut conn = self.connection.write().await;
        let key = format!("checkpoint:{}", checkpoint_id);
        
        let data_json: Option<String> = conn.get(&key).await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        if let Some(json) = data_json {
            let checkpoint_data: CheckpointData = serde_json::from_str(&json)?;
            
            // Limpar estado atual
            let task_ids: Vec<String> = conn.smembers("tasks:all").await
                .map_err(|e| TaskMeshError::Redis(e))?;
            
            for task_id_str in task_ids {
                if let Ok(task_id) = uuid::Uuid::parse_str(&task_id_str) {
                    self.remove_task(&task_id).await?;
                }
            }
            
            // Restaurar tarefas
            for task in checkpoint_data.tasks {
                self.store_task(&task).await?;
            }
            
            Ok(())
        } else {
            Err(TaskMeshError::CheckpointNotFound(checkpoint_id.to_string()))
        }
    }
    
    async fn list_checkpoints(&self) -> TaskMeshResult<Vec<String>> {
        debug!("Listando checkpoints do Redis");
        
        let mut conn = self.connection.write().await;
        let checkpoints: Vec<String> = conn.smembers("checkpoints:all").await
            .map_err(|e| TaskMeshError::Redis(e))?;
        
        Ok(checkpoints)
    }
    
    async fn cleanup_old_data(&self, _retention_days: u32) -> TaskMeshResult<()> {
        debug!("Limpeza de dados do Redis não implementada");
        // TODO: Implementar limpeza de dados antigos no Redis
        Ok(())
    }
}

/// Implementação em memória
impl MemoryStateStore {
    pub async fn new() -> TaskMeshResult<Self> {
        Ok(Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_status: Arc::new(RwLock::new(HashMap::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            checkpoints: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

#[async_trait]
impl StateStore for MemoryStateStore {
    async fn store_task(&self, task: &Task) -> TaskMeshResult<()> {
        self.tasks.write().await.insert(task.id, task.clone());
        Ok(())
    }
    
    async fn get_task(&self, task_id: &TaskId) -> TaskMeshResult<Option<Task>> {
        Ok(self.tasks.read().await.get(task_id).cloned())
    }
    
    async fn remove_task(&self, task_id: &TaskId) -> TaskMeshResult<()> {
        self.tasks.write().await.remove(task_id);
        self.task_status.write().await.remove(task_id);
        Ok(())
    }
    
    async fn update_task_status(&self, task_id: &TaskId, status: TaskStatus) -> TaskMeshResult<()> {
        self.task_status.write().await.insert(*task_id, status);
        Ok(())
    }
    
    async fn get_task_status(&self, task_id: &TaskId) -> TaskMeshResult<TaskStatus> {
        Ok(self.task_status.read().await.get(task_id).cloned().unwrap_or(TaskStatus::Pending))
    }
    
    async fn list_tasks(&self) -> TaskMeshResult<Vec<Task>> {
        Ok(self.tasks.read().await.values().cloned().collect())
    }
    
    async fn list_tasks_by_status(&self, status_filter: &[TaskStatus]) -> TaskMeshResult<Vec<Task>> {
        let tasks = self.tasks.read().await;
        let status_map = self.task_status.read().await;
        
        let filtered_tasks: Vec<Task> = tasks.values()
            .filter(|task| {
                let status = status_map.get(&task.id).unwrap_or(&TaskStatus::Pending);
                status_filter.contains(status)
            })
            .cloned()
            .collect();
        
        Ok(filtered_tasks)
    }
    
    async fn store_event(&self, event: &SystemEvent) -> TaskMeshResult<()> {
        self.events.write().await.push(event.clone());
        Ok(())
    }
    
    async fn get_events(
        &self, 
        start_time: Option<SystemTime>, 
        end_time: Option<SystemTime>
    ) -> TaskMeshResult<Vec<SystemEvent>> {
        let events = self.events.read().await;
        
        let filtered_events: Vec<SystemEvent> = events.iter()
            .filter(|event| {
                if let Some(start) = start_time {
                    if event.timestamp < start {
                        return false;
                    }
                }
                if let Some(end) = end_time {
                    if event.timestamp > end {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();
        
        Ok(filtered_events)
    }
    
    async fn store_metrics(&self, task_id: &TaskId, metrics: &ExecutionMetrics) -> TaskMeshResult<()> {
        self.metrics.write().await.insert(*task_id, metrics.clone());
        Ok(())
    }
    
    async fn get_metrics(&self, task_id: &TaskId) -> TaskMeshResult<Option<ExecutionMetrics>> {
        Ok(self.metrics.read().await.get(task_id).cloned())
    }
    
    async fn create_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()> {
        let tasks = self.list_tasks().await?;
        let checkpoint_data = CheckpointData {
            tasks,
            created_at: SystemTime::now(),
        };
        
        let data = bincode::serialize(&checkpoint_data)
            .map_err(|e| TaskMeshError::Internal(format!("Erro de serialização: {}", e)))?;
        
        self.checkpoints.write().await.insert(checkpoint_id.to_string(), data);
        Ok(())
    }
    
    async fn restore_checkpoint(&self, checkpoint_id: &str) -> TaskMeshResult<()> {
        let checkpoints = self.checkpoints.read().await;
        
        if let Some(data) = checkpoints.get(checkpoint_id) {
            let checkpoint_data: CheckpointData = bincode::deserialize(data)
                .map_err(|e| TaskMeshError::Internal(format!("Erro de desserialização: {}", e)))?;
            
            // Limpar estado atual
            self.tasks.write().await.clear();
            self.task_status.write().await.clear();
            
            // Restaurar tarefas
            for task in checkpoint_data.tasks {
                self.store_task(&task).await?;
            }
            
            Ok(())
        } else {
            Err(TaskMeshError::CheckpointNotFound(checkpoint_id.to_string()))
        }
    }
    
    async fn list_checkpoints(&self) -> TaskMeshResult<Vec<String>> {
        Ok(self.checkpoints.read().await.keys().cloned().collect())
    }
    
    async fn cleanup_old_data(&self, _retention_days: u32) -> TaskMeshResult<()> {
        // Para implementação em memória, não há necessidade de limpeza
        Ok(())
    }
}

/// Dados de checkpoint
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct CheckpointData {
    tasks: Vec<Task>,
    created_at: SystemTime,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_state_store() {
        let store = MemoryStateStore::new().await.unwrap();
        
        let task = Task::new(
            "test_task".to_string(),
            TaskDefinition::Command("echo hello".to_string()),
            vec![],
        );
        let task_id = task.id;
        
        // Armazenar tarefa
        store.store_task(&task).await.unwrap();
        
        // Recuperar tarefa
        let retrieved_task = store.get_task(&task_id).await.unwrap();
        assert!(retrieved_task.is_some());
        
        // Atualizar status
        store.update_task_status(&task_id, TaskStatus::Running {
            started_at: SystemTime::now(),
            worker_id: "worker_1".to_string(),
        }).await.unwrap();
        
        // Verificar status
        let status = store.get_task_status(&task_id).await.unwrap();
        assert!(status.is_active());
        
        // Remover tarefa
        store.remove_task(&task_id).await.unwrap();
        
        let removed_task = store.get_task(&task_id).await.unwrap();
        assert!(removed_task.is_none());
    }
    
    #[tokio::test]
    async fn test_checkpoint_creation() {
        let store = MemoryStateStore::new().await.unwrap();
        
        let task = Task::new(
            "test_task".to_string(),
            TaskDefinition::Command("echo hello".to_string()),
            vec![],
        );
        
        store.store_task(&task).await.unwrap();
        
        // Criar checkpoint
        store.create_checkpoint("test_checkpoint").await.unwrap();
        
        // Listar checkpoints
        let checkpoints = store.list_checkpoints().await.unwrap();
        assert!(checkpoints.contains(&"test_checkpoint".to_string()));
        
        // Remover tarefa
        store.remove_task(&task.id).await.unwrap();
        
        // Restaurar checkpoint
        store.restore_checkpoint("test_checkpoint").await.unwrap();
        
        // Verificar se tarefa foi restaurada
        let restored_task = store.get_task(&task.id).await.unwrap();
        assert!(restored_task.is_some());
    }
}


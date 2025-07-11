//! # Demonstração do Sistema de Backup e Checkpoint
//!
//! Este exemplo demonstra como usar o sistema completo de backup e checkpoint:
//! - Configuração do sistema
//! - Criação de snapshots
//! - Checkpoints automáticos
//! - Restauração de dados

use orchestrator_core::{
    backup::{
        BackupSystem, BackupConfig, MinioConfig, SqliteConfig, 
        SnapshotConfig, CheckpointConfig, SystemState
    },
    graph::{TaskMesh, TaskNode, TaskId, TaskStatus, TaskPriority},
    metrics::SystemMetrics,
    errors::Result,
};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{info, error};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Inicializar logging
    tracing_subscriber::fmt::init();
    
    info!("=== Demonstração do Sistema de Backup e Checkpoint ===");
    
    // 1. Configurar sistema de backup
    let backup_config = create_backup_config();
    let backup_system = BackupSystem::new(backup_config).await?;
    
    // 2. Criar TaskGraph de exemplo
    let task_graph = create_sample_task_graph();
    let system_metrics = create_sample_metrics();
    
    // 3. Demonstrar snapshot
    info!("\n--- Criando snapshot do TaskGraph ---");
    let snapshot = backup_system.create_snapshot(&task_graph, &system_metrics).await?;
    info!("Snapshot criado: ID={}", snapshot.id);
    
    // 4. Demonstrar checkpoints automáticos
    info!("\n--- Simulando conclusão de tarefas para checkpoint ---");
    for i in 1..=15 {
        let task_id = Uuid::new_v4();
        if let Some(checkpoint) = backup_system.on_task_completed(task_id).await? {
            info!("Checkpoint criado automaticamente: ID={}, task_count={}", 
                  checkpoint.id, checkpoint.task_count);
        }
        info!("Tarefa {} concluída", i);
    }
    
    // 5. Demonstrar checkpoint manual
    info!("\n--- Criando checkpoint manual ---");
    let system_state = SystemState {
        active_tasks: vec![Uuid::new_v4(), Uuid::new_v4()],
        pending_tasks: vec![Uuid::new_v4()],
        failed_tasks: vec![],
        resource_usage: HashMap::from([
            ("cpu".to_string(), 45.5),
            ("memory".to_string(), 67.8),
            ("disk".to_string(), 23.1),
        ]),
        configuration_hash: "abc123def456".to_string(),
    };
    
    let mut recovery_data = HashMap::new();
    recovery_data.insert("manual_trigger".to_string(), serde_json::json!(true));
    recovery_data.insert("user".to_string(), serde_json::json!("admin"));
    
    let manual_checkpoint = backup_system.create_checkpoint(
        20,
        Some(Uuid::new_v4()),
        system_state,
        recovery_data,
    ).await?;
    info!("Checkpoint manual criado: ID={}", manual_checkpoint.id);
    
    // 6. Demonstrar restauração
    info!("\n--- Testando restauração ---");
    
    // Restaurar snapshot mais recente
    if let Some(restored_snapshot) = backup_system.restore_latest_snapshot().await? {
        info!("Snapshot restaurado: ID={}, {} tarefas totais", 
              restored_snapshot.id, restored_snapshot.metadata.total_tasks);
    }
    
    // Restaurar checkpoint mais recente
    if let Some(restored_checkpoint) = backup_system.restore_latest_checkpoint().await? {
        info!("Checkpoint restaurado: ID={}, {} tarefas completadas", 
              restored_checkpoint.id, restored_checkpoint.task_count);
    }
    
    // 7. Simular restauração no boot
    info!("\n--- Simulando restauração no boot ---");
    let (boot_snapshot, boot_checkpoint) = backup_system.auto_restore_on_boot().await?;
    
    match (boot_snapshot, boot_checkpoint) {
        (Some(s), Some(c)) => {
            info!("Boot: Restaurados snapshot {} e checkpoint {}", s.id, c.id);
        }
        (Some(s), None) => {
            info!("Boot: Restaurado apenas snapshot {}", s.id);
        }
        (None, Some(c)) => {
            info!("Boot: Restaurado apenas checkpoint {}", c.id);
        }
        (None, None) => {
            info!("Boot: Nenhum backup encontrado");
        }
    }
    
    // 8. Exibir estatísticas
    info!("\n--- Estatísticas do sistema de backup ---");
    let stats = backup_system.get_backup_stats().await?;
    info!("Snapshots: {}", stats.snapshot_count);
    info!("Checkpoints: {}", stats.checkpoint_count);
    info!("Tamanho total: {} bytes", stats.total_size_bytes);
    info!("Tarefas completadas: {}", stats.completed_tasks_count);
    
    if let Some(last_snapshot) = stats.last_snapshot_time {
        info!("Último snapshot: {}", last_snapshot);
    }
    
    if let Some(last_checkpoint) = stats.last_checkpoint_time {
        info!("Último checkpoint: {}", last_checkpoint);
    }
    
    info!("\n=== Demonstração concluída com sucesso! ===");
    Ok(())
}

/// Cria configuração do sistema de backup
fn create_backup_config() -> BackupConfig {
    BackupConfig {
        minio_config: MinioConfig {
            endpoint: "http://localhost:9000".to_string(),
            bucket_name: "arkitect-backups".to_string(),
            access_key: "minioadmin".to_string(),
            secret_key: "minioadmin".to_string(),
            region: "us-east-1".to_string(),
        },
        sqlite_config: SqliteConfig {
            database_path: PathBuf::from("./data/backup.db"),
            max_connections: 10,
            connection_timeout_seconds: 30,
        },
        snapshot_config: SnapshotConfig {
            interval_seconds: 300, // 5 minutos
            max_snapshots: 10,
            compression_enabled: true,
            snapshot_prefix: "taskgraph".to_string(),
        },
        checkpoint_config: CheckpointConfig {
            tasks_per_checkpoint: 10, // Checkpoint a cada 10 tarefas
            retention_days: 30,
            auto_cleanup: true,
        },
    }
}

/// Cria TaskGraph de exemplo para demonstração
fn create_sample_task_graph() -> TaskMesh {
    let mut graph = TaskMesh::new();
    
    // Adicionar algumas tarefas de exemplo
    let task1 = TaskNode {
        id: Uuid::new_v4(),
        name: "Processar dados".to_string(),
        description: Some("Processa dados de entrada".to_string()),
        status: TaskStatus::Completed,
        priority: TaskPriority::High,
        estimated_duration: Some(std::time::Duration::from_secs(300)),
        actual_duration: Some(std::time::Duration::from_secs(280)),
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: Some(chrono::Utc::now()),
        retry_count: 0,
        max_retries: 3,
        layer_preference: None,
        metadata: HashMap::new(),
    };
    
    let task2 = TaskNode {
        id: Uuid::new_v4(),
        name: "Validar resultados".to_string(),
        description: Some("Valida os dados processados".to_string()),
        status: TaskStatus::Running,
        priority: TaskPriority::Medium,
        estimated_duration: Some(std::time::Duration::from_secs(180)),
        actual_duration: None,
        created_at: chrono::Utc::now(),
        started_at: Some(chrono::Utc::now()),
        completed_at: None,
        retry_count: 0,
        max_retries: 2,
        layer_preference: None,
        metadata: HashMap::new(),
    };
    
    let task3 = TaskNode {
        id: Uuid::new_v4(),
        name: "Gerar relatório".to_string(),
        description: Some("Gera relatório final".to_string()),
        status: TaskStatus::Pending,
        priority: TaskPriority::Low,
        estimated_duration: Some(std::time::Duration::from_secs(120)),
        actual_duration: None,
        created_at: chrono::Utc::now(),
        started_at: None,
        completed_at: None,
        retry_count: 0,
        max_retries: 1,
        layer_preference: None,
        metadata: HashMap::new(),
    };
    
    // Adicionar nós ao grafo
    let node1 = graph.add_node(task1);
    let node2 = graph.add_node(task2);
    let node3 = graph.add_node(task3);
    
    // Adicionar dependências (task2 depende de task1, task3 depende de task2)
    let _ = graph.add_edge(node1, node2, orchestrator_core::graph::DependencyEdge {
        id: Uuid::new_v4(),
        source_task: graph[node1].id,
        target_task: graph[node2].id,
        dependency_type: orchestrator_core::graph::DependencyType::Sequential,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    });
    
    let _ = graph.add_edge(node2, node3, orchestrator_core::graph::DependencyEdge {
        id: Uuid::new_v4(),
        source_task: graph[node2].id,
        target_task: graph[node3].id,
        dependency_type: orchestrator_core::graph::DependencyType::Sequential,
        created_at: chrono::Utc::now(),
        metadata: HashMap::new(),
    });
    
    graph
}

/// Cria métricas de exemplo
fn create_sample_metrics() -> SystemMetrics {
    SystemMetrics {
        timestamp: chrono::Utc::now(),
        total_tasks: 25,
        completed_tasks: 18,
        failed_tasks: 2,
        running_tasks: 3,
        pending_tasks: 2,
        cpu_usage: 42.5,
        memory_usage: 68.3,
        disk_usage: 34.7,
        network_io: 1024.0,
        active_connections: 5,
        average_task_duration: std::time::Duration::from_secs(240),
        throughput_per_minute: 2.3,
        error_rate: 0.08,
        system_health_score: 0.92,
        custom_metrics: HashMap::from([
            ("cache_hit_rate".to_string(), 0.85),
            ("queue_length".to_string(), 12.0),
        ]),
    }
}


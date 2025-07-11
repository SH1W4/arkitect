//! # TaskMesh Core
//!
//! Sistema de orquestração de tarefas simbiótico com capacidades avançadas de
//! scheduling, execução assíncrona, persistência e recuperação de estado.
//!
//! ## Características Principais
//!
//! - **TaskRegistry**: Registro centralizado de tarefas com metadados
//! - **Scheduler**: Agendamento topológico com heurísticas avançadas
//! - **Executor**: Execução assíncrona usando Tokio e Rayon
//! - **StateStore**: Persistência em SQLite/Redis com sincronização
//! - **CheckpointEngine**: Sistema de checkpoints para recuperação
//! - **ErrorHandler**: Tratamento robusto de erros com retry patterns
//! - **FFI**: Interface Python via maturin/PyO3

use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};

// Módulos públicos
pub mod task_registry;
pub mod scheduler;
pub mod executor;
pub mod state_store;
pub mod checkpoint;
pub mod error_handler;
pub mod types;
pub mod metrics;

// FFI Python (opcional)
#[cfg(feature = "python")]
pub mod python_bindings;

// Re-exports públicos
pub use task_registry::TaskRegistry;
pub use scheduler::{Scheduler, SchedulingHeuristic};
pub use executor::{TaskExecutor, ExecutionContext};
pub use state_store::{StateStore, StorageBackend};
pub use checkpoint::{CheckpointEngine, CheckpointStrategy};
pub use error_handler::{ErrorHandler, RetryPolicy};
pub use types::*;

/// Configuração principal do TaskMesh Core
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskMeshConfig {
    /// Configuração do banco de dados
    pub database_url: String,
    /// Configuração do Redis (opcional)
    pub redis_url: Option<String>,
    /// Número máximo de workers
    pub max_workers: usize,
    /// Intervalo de checkpoint em segundos
    pub checkpoint_interval: u64,
    /// Estratégia de retry padrão
    pub retry_policy: RetryPolicy,
    /// Habilitar métricas
    pub enable_metrics: bool,
}

impl Default for TaskMeshConfig {
    fn default() -> Self {
        Self {
            database_url: "sqlite::memory:".to_string(),
            redis_url: None,
            max_workers: num_cpus::get(),
            checkpoint_interval: 30,
            retry_policy: RetryPolicy::default(),
            enable_metrics: false,
        }
    }
}

/// Core principal do TaskMesh
///
/// Integra todos os componentes em uma interface unificada
pub struct TaskMeshCore {
    /// Registro de tarefas
    pub registry: Arc<RwLock<TaskRegistry>>,
    /// Scheduler
    pub scheduler: Arc<Scheduler>,
    /// Executor
    pub executor: Arc<TaskExecutor>,
    /// Armazenamento de estado
    pub state_store: Arc<dyn StateStore>,
    /// Engine de checkpoint
    pub checkpoint_engine: Arc<CheckpointEngine>,
    /// Handler de erros
    pub error_handler: Arc<ErrorHandler>,
    /// Configuração
    config: TaskMeshConfig,
}

impl TaskMeshCore {
    /// Cria uma nova instância do TaskMesh Core
    pub async fn new(config: TaskMeshConfig) -> Result<Self, TaskMeshError> {
        info!("Inicializando TaskMesh Core");

        // Inicializar componentes
        let registry = Arc::new(RwLock::new(TaskRegistry::new()));
        let state_store = Self::create_state_store(&config).await?;
        let error_handler = Arc::new(ErrorHandler::new(config.retry_policy.clone()));
        let checkpoint_engine = Arc::new(CheckpointEngine::new(
            state_store.clone(),
            config.checkpoint_interval,
        ));
        let scheduler = Arc::new(Scheduler::new(SchedulingHeuristic::default()));
        let executor = Arc::new(TaskExecutor::new(
            config.max_workers,
            state_store.clone(),
            error_handler.clone(),
        ).await?);

        let core = Self {
            registry,
            scheduler,
            executor,
            state_store,
            checkpoint_engine,
            error_handler,
            config,
        };

        // Inicializar métricas se habilitado
        #[cfg(feature = "metrics")]
        if config.enable_metrics {
            metrics::init_metrics();
        }

        info!("TaskMesh Core inicializado com sucesso");
        Ok(core)
    }

    /// Cria o armazenamento de estado baseado na configuração
    async fn create_state_store(
        config: &TaskMeshConfig,
    ) -> Result<Arc<dyn StateStore>, TaskMeshError> {
        use state_store::*;

        if config.database_url.starts_with("sqlite") {
            let store = SqliteStateStore::new(&config.database_url).await?;
            Ok(Arc::new(store))
        } else if config.database_url.starts_with("postgres") {
            let store = PostgresStateStore::new(&config.database_url).await?;
            Ok(Arc::new(store))
        } else if let Some(redis_url) = &config.redis_url {
            let store = RedisStateStore::new(redis_url).await?;
            Ok(Arc::new(store))
        } else {
            Err(TaskMeshError::Configuration(
                "URL de banco de dados inválida".to_string(),
            ))
        }
    }

    /// Inicia o TaskMesh Core
    pub async fn start(&self) -> Result<(), TaskMeshError> {
        info!("Iniciando TaskMesh Core");

        // Iniciar checkpoint engine
        self.checkpoint_engine.start().await?;

        // Iniciar executor
        self.executor.start().await?;

        info!("TaskMesh Core iniciado");
        Ok(())
    }

    /// Para o TaskMesh Core graciosamente
    pub async fn shutdown(&self) -> Result<(), TaskMeshError> {
        info!("Parando TaskMesh Core");

        // Parar executor
        self.executor.shutdown().await?;

        // Parar checkpoint engine
        self.checkpoint_engine.stop().await?;

        // Criar checkpoint final
        self.checkpoint_engine.create_checkpoint().await?;

        info!("TaskMesh Core parado");
        Ok(())
    }

    /// Submete uma nova tarefa
    pub async fn submit_task(&self, task: Task) -> Result<TaskId, TaskMeshError> {
        let task_id = task.id;

        // Registrar tarefa
        self.registry.write().await.register_task(task.clone())?;

        // Agendar execução
        self.scheduler.schedule_task(task).await?;

        info!("Tarefa {} submetida", task_id);
        Ok(task_id)
    }

    /// Obtém o status de uma tarefa
    pub async fn get_task_status(&self, task_id: &TaskId) -> Result<TaskStatus, TaskMeshError> {
        self.state_store.get_task_status(task_id).await
    }

    /// Lista todas as tarefas
    pub async fn list_tasks(&self) -> Result<Vec<Task>, TaskMeshError> {
        self.registry.read().await.list_tasks()
    }

    /// Cancela uma tarefa
    pub async fn cancel_task(&self, task_id: &TaskId) -> Result<(), TaskMeshError> {
        self.executor.cancel_task(task_id).await
    }

    /// Obtém métricas do sistema
    #[cfg(feature = "metrics")]
    pub async fn get_metrics(&self) -> Result<metrics::SystemMetrics, TaskMeshError> {
        metrics::collect_metrics().await
    }

    /// Força criação de checkpoint
    pub async fn create_checkpoint(&self) -> Result<(), TaskMeshError> {
        self.checkpoint_engine.create_checkpoint().await
    }

    /// Restaura estado a partir de checkpoint
    pub async fn restore_from_checkpoint(
        &self,
        checkpoint_id: &str,
    ) -> Result<(), TaskMeshError> {
        self.checkpoint_engine.restore_checkpoint(checkpoint_id).await
    }
}

// Helper para inicializar logging
pub fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_task_mesh_core_creation() {
        let config = TaskMeshConfig::default();
        let core = TaskMeshCore::new(config).await;
        assert!(core.is_ok());
    }

    #[tokio::test]
    async fn test_submit_and_get_task() {
        let config = TaskMeshConfig::default();
        let core = TaskMeshCore::new(config).await.unwrap();
        
        let task = Task::new(
            "test_task".to_string(),
            TaskDefinition::Command("echo hello".to_string()),
            vec![],
        );
        let task_id = task.id;
        
        let result = core.submit_task(task).await;
        assert!(result.is_ok());
        
        let status = core.get_task_status(&task_id).await;
        assert!(status.is_ok());
    }
}


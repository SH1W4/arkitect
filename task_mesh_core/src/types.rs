//! Definições de tipos fundamentais do TaskMesh Core

use std::collections::HashMap;
use std::fmt;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identificador único de tarefa
pub type TaskId = Uuid;

/// Identificador de dependência
pub type DependencyId = Uuid;

/// Prioridade de tarefa (0-100, onde 100 é maior prioridade)
pub type Priority = u8;

/// Definição de uma tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Identificador único
    pub id: TaskId,
    /// Nome da tarefa
    pub name: String,
    /// Definição da tarefa
    pub definition: TaskDefinition,
    /// Dependências
    pub dependencies: Vec<TaskId>,
    /// Prioridade (0-100)
    pub priority: Priority,
    /// Metadados adicionais
    pub metadata: HashMap<String, String>,
    /// Timestamp de criação
    pub created_at: SystemTime,
    /// Timeout máximo de execução
    pub timeout: Option<Duration>,
    /// Número máximo de tentativas
    pub max_retries: u32,
    /// Tags para organização
    pub tags: Vec<String>,
}

impl Task {
    /// Cria uma nova tarefa
    pub fn new(
        name: String,
        definition: TaskDefinition,
        dependencies: Vec<TaskId>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            definition,
            dependencies,
            priority: 50, // Prioridade média por padrão
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            timeout: None,
            max_retries: 3,
            tags: Vec::new(),
        }
    }

    /// Define a prioridade da tarefa
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = priority.min(100);
        self
    }

    /// Define o timeout da tarefa
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Define o número máximo de tentativas
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Adiciona metadados
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Adiciona tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags.extend(tags);
        self
    }

    /// Verifica se a tarefa tem dependências não resolvidas
    pub fn has_unresolved_dependencies(&self, resolved_tasks: &[TaskId]) -> bool {
        self.dependencies
            .iter()
            .any(|dep| !resolved_tasks.contains(dep))
    }
}

/// Tipos de definição de tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskDefinition {
    /// Comando shell
    Command(String),
    /// Script Python
    PythonScript {
        script: String,
        args: Vec<String>,
        env: HashMap<String, String>,
    },
    /// Função Rust
    RustFunction {
        function_name: String,
        args: serde_json::Value,
    },
    /// HTTP Request
    HttpRequest {
        method: String,
        url: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    },
    /// Workflow composto
    Workflow {
        tasks: Vec<Task>,
        execution_strategy: WorkflowStrategy,
    },
}

/// Estratégias de execução de workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStrategy {
    /// Execução sequencial
    Sequential,
    /// Execução paralela
    Parallel,
    /// Execução baseada em DAG
    DAG,
}

/// Status de execução de uma tarefa
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TaskStatus {
    /// Tarefa criada, aguardando agendamento
    Pending,
    /// Tarefa agendada, aguardando dependências
    Scheduled,
    /// Tarefa em execução
    Running {
        started_at: SystemTime,
        worker_id: String,
    },
    /// Tarefa concluída com sucesso
    Completed {
        started_at: SystemTime,
        completed_at: SystemTime,
        result: TaskResult,
    },
    /// Tarefa falhou
    Failed {
        started_at: SystemTime,
        failed_at: SystemTime,
        error: String,
        retry_count: u32,
    },
    /// Tarefa cancelada
    Cancelled {
        cancelled_at: SystemTime,
        reason: String,
    },
    /// Tarefa pausada
    Paused {
        paused_at: SystemTime,
        reason: String,
    },
}

impl TaskStatus {
    /// Verifica se a tarefa está em estado final
    pub fn is_final(&self) -> bool {
        matches!(
            self,
            TaskStatus::Completed { .. }
                | TaskStatus::Failed { .. }
                | TaskStatus::Cancelled { .. }
        )
    }

    /// Verifica se a tarefa está ativa
    pub fn is_active(&self) -> bool {
        matches!(self, TaskStatus::Running { .. })
    }

    /// Verifica se a tarefa pode ser executada
    pub fn can_execute(&self) -> bool {
        matches!(self, TaskStatus::Scheduled | TaskStatus::Paused { .. })
    }
}

/// Resultado da execução de uma tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Status de saída
    pub exit_code: i32,
    /// Saída padrão
    pub stdout: String,
    /// Saída de erro
    pub stderr: String,
    /// Dados de saída estruturados
    pub output_data: Option<serde_json::Value>,
    /// Métricas de execução
    pub metrics: ExecutionMetrics,
}

/// Métricas de execução
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Tempo de execução
    pub execution_time: Duration,
    /// Uso de CPU (porcentagem)
    pub cpu_usage: f64,
    /// Uso de memória (bytes)
    pub memory_usage: u64,
    /// I/O de rede (bytes lidos/escritos)
    pub network_io: (u64, u64),
    /// I/O de disco (bytes lidos/escritos)
    pub disk_io: (u64, u64),
}

impl Default for ExecutionMetrics {
    fn default() -> Self {
        Self {
            execution_time: Duration::from_secs(0),
            cpu_usage: 0.0,
            memory_usage: 0,
            network_io: (0, 0),
            disk_io: (0, 0),
        }
    }
}

/// Contexto de execução para uma tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// ID do worker executando a tarefa
    pub worker_id: String,
    /// Diretório de trabalho
    pub working_directory: String,
    /// Variáveis de ambiente
    pub environment: HashMap<String, String>,
    /// Recursos alocados
    pub allocated_resources: ResourceAllocation,
    /// Checkpoint ativo
    pub checkpoint_id: Option<String>,
}

/// Alocação de recursos
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Número de CPUs alocadas
    pub cpu_cores: f64,
    /// Memória alocada (bytes)
    pub memory_bytes: u64,
    /// Limite de tempo
    pub time_limit: Option<Duration>,
    /// Prioridade de agendamento
    pub scheduling_priority: Priority,
}

impl Default for ResourceAllocation {
    fn default() -> Self {
        Self {
            cpu_cores: 1.0,
            memory_bytes: 1024 * 1024 * 1024, // 1GB
            time_limit: Some(Duration::from_secs(3600)), // 1 hora
            scheduling_priority: 50,
        }
    }
}

/// Política de retry para tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Número máximo de tentativas
    pub max_attempts: u32,
    /// Estratégia de backoff
    pub backoff_strategy: BackoffStrategy,
    /// Condições para retry
    pub retry_conditions: Vec<RetryCondition>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_strategy: BackoffStrategy::Exponential {
                initial_delay: Duration::from_secs(1),
                max_delay: Duration::from_secs(300),
                multiplier: 2.0,
            },
            retry_conditions: vec![
                RetryCondition::ExitCode(vec![1, 2]),
                RetryCondition::Timeout,
                RetryCondition::ResourceUnavailable,
            ],
        }
    }
}

/// Estratégias de backoff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackoffStrategy {
    /// Delay fixo
    Fixed {
        delay: Duration,
    },
    /// Backoff linear
    Linear {
        initial_delay: Duration,
        increment: Duration,
        max_delay: Duration,
    },
    /// Backoff exponencial
    Exponential {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
    },
}

/// Condições para retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryCondition {
    /// Exit codes específicos
    ExitCode(Vec<i32>),
    /// Timeout
    Timeout,
    /// Recurso indisponível
    ResourceUnavailable,
    /// Erro de rede
    NetworkError,
    /// Palavras-chave no stderr
    StderrContains(Vec<String>),
}

/// Erros do TaskMesh
#[derive(Debug, thiserror::Error)]
pub enum TaskMeshError {
    #[error("Erro de configuração: {0}")]
    Configuration(String),

    #[error("Erro de banco de dados: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Erro de Redis: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),

    #[error("Erro de serialização: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Tarefa não encontrada: {0}")]
    TaskNotFound(TaskId),

    #[error("Dependência circular detectada: {0:?}")]
    CircularDependency(Vec<TaskId>),

    #[error("Recurso indisponível: {0}")]
    ResourceUnavailable(String),

    #[error("Timeout na execução da tarefa: {0}")]
    ExecutionTimeout(TaskId),

    #[error("Erro na execução da tarefa: {0}")]
    ExecutionError(String),

    #[error("Checkpoint não encontrado: {0}")]
    CheckpointNotFound(String),

    #[error("Erro interno: {0}")]
    Internal(String),
}

/// Resultado padrão do TaskMesh
pub type TaskMeshResult<T> = Result<T, TaskMeshError>;

/// Evento do sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEvent {
    /// Timestamp do evento
    pub timestamp: SystemTime,
    /// Tipo do evento
    pub event_type: EventType,
    /// ID da tarefa relacionada (se aplicável)
    pub task_id: Option<TaskId>,
    /// Dados do evento
    pub data: serde_json::Value,
}

/// Tipos de eventos do sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    TaskSubmitted,
    TaskScheduled,
    TaskStarted,
    TaskCompleted,
    TaskFailed,
    TaskCancelled,
    CheckpointCreated,
    CheckpointRestored,
    WorkerStarted,
    WorkerStopped,
    SystemStarted,
    SystemStopped,
}

/// Informações de um worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerInfo {
    /// ID único do worker
    pub id: String,
    /// Status do worker
    pub status: WorkerStatus,
    /// Recursos disponíveis
    pub available_resources: ResourceAllocation,
    /// Tarefa atual (se houver)
    pub current_task: Option<TaskId>,
    /// Estatísticas do worker
    pub stats: WorkerStats,
    /// Última atualização
    pub last_heartbeat: SystemTime,
}

/// Status do worker
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkerStatus {
    /// Worker disponível
    Idle,
    /// Worker ocupado
    Busy,
    /// Worker indisponível
    Unavailable,
    /// Worker parado
    Stopped,
}

/// Estatísticas do worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
    /// Total de tarefas executadas
    pub tasks_completed: u64,
    /// Total de tarefas que falharam
    pub tasks_failed: u64,
    /// Tempo total de execução
    pub total_execution_time: Duration,
    /// Tempo médio por tarefa
    pub average_task_time: Duration,
    /// Último erro (se houver)
    pub last_error: Option<String>,
}

impl Default for WorkerStats {
    fn default() -> Self {
        Self {
            tasks_completed: 0,
            tasks_failed: 0,
            total_execution_time: Duration::from_secs(0),
            average_task_time: Duration::from_secs(0),
            last_error: None,
        }
    }
}

// Implementações de Display para melhor debug
impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Scheduled => write!(f, "Scheduled"),
            TaskStatus::Running { started_at, worker_id } => {
                write!(f, "Running on {} since {:?}", worker_id, started_at)
            }
            TaskStatus::Completed { completed_at, .. } => {
                write!(f, "Completed at {:?}", completed_at)
            }
            TaskStatus::Failed { error, retry_count, .. } => {
                write!(f, "Failed ({} retries): {}", retry_count, error)
            }
            TaskStatus::Cancelled { reason, .. } => {
                write!(f, "Cancelled: {}", reason)
            }
            TaskStatus::Paused { reason, .. } => {
                write!(f, "Paused: {}", reason)
            }
        }
    }
}

impl fmt::Display for WorkerStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WorkerStatus::Idle => write!(f, "Idle"),
            WorkerStatus::Busy => write!(f, "Busy"),
            WorkerStatus::Unavailable => write!(f, "Unavailable"),
            WorkerStatus::Stopped => write!(f, "Stopped"),
        }
    }
}


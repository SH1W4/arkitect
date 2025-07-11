//! # Error Types
//!
//! Framework robusto de tratamento de erros para o SYMBIOTIC_METHOD.
//! Implementa ErrorKind enum, retry com backoff, circuit-breaker e logging contextual.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, warn, info, debug, instrument};
use uuid::Uuid;

/// Resultado padrão para operações do orchestrator
pub type Result<T> = std::result::Result<T, OrchestratorError>;

/// Erros do orchestrator
#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Validation error in field {field}: {message}")]
    ValidationError {
        field: String,
        message: String,
        kind: ErrorKind,
        context: ErrorContext,
    },
    #[error("Runtime error in component {component}: {message}")]
    RuntimeError {
        component: String,
        message: String,
        kind: ErrorKind,
        context: ErrorContext,
        retry_info: Option<RetryInfo>,
    },
    #[error("External service error in {service}: {message}")]
    ExternalServiceError {
        service: String,
        message: String,
        kind: ErrorKind,
        context: ErrorContext,
        circuit_breaker_state: CircuitBreakerState,
    },
    #[error("Panic error: {reason}")]
    PanicError {
        reason: String,
        kind: ErrorKind,
        context: ErrorContext,
        recovery_strategy: RecoveryStrategy,
    },
    /// Tarefa não encontrada
    #[error("Task not found: {0}")]
    TaskNotFound(Uuid),
    
    /// Dependência cíclica detectada
    #[error("Cyclic dependency detected in task graph")]
    CyclicDependency,
    
    /// Limite de recursos excedido
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    /// Nenhum nó ativo no cluster
    #[error("No active nodes available in cluster")]
    NoActiveNodes,
    
    /// Camada de execução não disponível
    #[error("Execution layer not available: {0:?}")]
    LayerNotAvailable(crate::layers::ExecutionLayer),
    
    /// Modelo não encontrado
    #[error("Learning model not found: {0}")]
    ModelNotFound(String),
    
    /// Dados insuficientes para treinamento
    #[error("Insufficient training data")]
    InsufficientData,
    
    /// Erro de configuração
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    /// Erro de serialização/deserialização
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    /// Erro de IO
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    /// Erro de rede
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),
    
    /// Erro de database
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    /// Erro de autenticação
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    
    /// Erro de autorização
    #[error("Authorization error: {0}")]
    AuthorizationError(String),
    
    /// Timeout
    #[error("Operation timeout: {0}")]
    Timeout(String),
    
    /// Estado inválido
    #[error("Invalid state: {0}")]
    InvalidState(String),
    
    /// Operação não suportada
    #[error("Unsupported operation: {0}")]
    UnsupportedOperation(String),
    
    /// Erro de consciência simbiótica
    #[error("Symbiotic consciousness error: {0}")]
    ConsciousnessError(String),
    
    /// Erro quântico
    #[error("Quantum simulation error: {0}")]
    QuantumError(String),
    
    /// Erro interno genérico
    #[error("Internal error: {0}")]
    InternalError(String),
    
    /// Erro externo
    #[error("External error: {0}")]
    ExternalError(#[from] anyhow::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
    Validation {
        field: String,
        rule: String,
        value: String,
    },
    Runtime {
        component: String,
        operation: String,
        cause: String,
    },
    External {
        service: String,
        endpoint: String,
        status_code: Option<u16>,
    },
    Panic {
        reason: String,
        stack_trace: String,
        recovery_suggestion: String,
    },
}

impl ErrorKind {
    pub fn is_recoverable(&self) -> bool {
        match self {
            ErrorKind::Validation { .. } => false,
            ErrorKind::Runtime { .. } => true,
            ErrorKind::External { .. } => true,
            ErrorKind::Panic { .. } => false,
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            ErrorKind::Validation { .. } => ErrorSeverity::Warning,
            ErrorKind::Runtime { .. } => ErrorSeverity::Error,
            ErrorKind::External { .. } => ErrorSeverity::Warning,
            ErrorKind::Panic { .. } => ErrorSeverity::Critical,
        }
    }

    pub fn telemetry_category(&self) -> &'static str {
        match self {
            ErrorKind::Validation { .. } => "symbiotic.validation",
            ErrorKind::Runtime { .. } => "symbiotic.runtime",
            ErrorKind::External { .. } => "symbiotic.external",
            ErrorKind::Panic { .. } => "symbiotic.panic",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Info = 1,
    Warning = 2,
    Error = 3,
    Critical = 4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryInfo {
    pub attempt: u32,
    pub max_attempts: u32,
    pub last_attempt_at: DateTime<Utc>,
    pub next_retry_at: DateTime<Utc>,
    pub backoff_duration: Duration,
    pub total_duration: Duration,
    pub exponential_base: f64,
    pub jitter_factor: f64,
}

impl RetryInfo {
    pub fn new(max_attempts: u32) -> Self {
        let now = Utc::now();
        Self {
            attempt: 0,
            max_attempts,
            last_attempt_at: now,
            next_retry_at: now,
            backoff_duration: Duration::from_millis(100),
            total_duration: Duration::ZERO,
            exponential_base: 2.0,
            jitter_factor: 0.1,
        }
    }

    pub fn should_retry(&self) -> bool {
        self.attempt < self.max_attempts && Utc::now() >= self.next_retry_at
    }

    pub fn record_attempt(&mut self) {
        self.attempt += 1;
        self.last_attempt_at = Utc::now();
        let base_delay = Duration::from_millis((100.0 * self.exponential_base.powi(self.attempt as i32 - 1)) as u64);
        let jitter = fastrand::f64() * self.jitter_factor;
        let jitter_factor = 1.0 + (jitter - self.jitter_factor / 2.0);
        self.backoff_duration = Duration::from_millis((base_delay.as_millis() as f64 * jitter_factor) as u64);
        self.next_retry_at = self.last_attempt_at + chrono::Duration::from_std(self.backoff_duration).unwrap();
        self.total_duration += base_delay;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open {
        opened_at: DateTime<Utc>,
        failure_count: u32,
    },
    HalfOpen {
        opened_at: DateTime<Utc>,
        test_request_sent: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    Restart {
        component: String,
        graceful: bool,
    },
    Fallback {
        primary_system: String,
        fallback_system: String,
    },
    Isolate {
        component: String,
        reason: String,
    },
    Escalate {
        priority: String,
        contact: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub trace_id: String,
    pub span_id: Option<String>,
    pub user_id: Option<String>,
    pub request_id: Option<String>,
}

impl ErrorContext {
    pub fn new(operation: &str, component: &str) -> Self {
        Self {
            operation: operation.to_string(),
            component: component.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            trace_id: Uuid::new_v4().to_string(),
            span_id: None,
            user_id: None,
            request_id: None,
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_trace(mut self, trace_id: String, span_id: Option<String>) -> Self {
        self.trace_id = trace_id;
        self.span_id = span_id;
        self
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn with_request(mut self, request_id: String) -> Self {
        self.request_id = Some(request_id);
        self
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub type SymbioticResult<T> = std::result::Result<T, OrchestratorError>;

impl OrchestratorError {
    /// Verifica se o erro é recuperável
    pub fn is_recoverable(&self) -> bool {
        match self {
            OrchestratorError::TaskNotFound(_) => false,
            OrchestratorError::CyclicDependency => false,
            OrchestratorError::ResourceLimitExceeded(_) => true,
            OrchestratorError::NoActiveNodes => true,
            OrchestratorError::LayerNotAvailable(_) => true,
            OrchestratorError::ModelNotFound(_) => false,
            OrchestratorError::InsufficientData => true,
            OrchestratorError::ConfigurationError(_) => false,
            OrchestratorError::SerializationError(_) => false,
            OrchestratorError::IoError(_) => true,
            OrchestratorError::NetworkError(_) => true,
            OrchestratorError::DatabaseError(_) => true,
            OrchestratorError::AuthenticationError(_) => false,
            OrchestratorError::AuthorizationError(_) => false,
            OrchestratorError::Timeout(_) => true,
            OrchestratorError::InvalidState(_) => false,
            OrchestratorError::UnsupportedOperation(_) => false,
            OrchestratorError::ConsciousnessError(_) => true,
            OrchestratorError::QuantumError(_) => true,
            OrchestratorError::InternalError(_) => false,
            OrchestratorError::ExternalError(_) => true,
            OrchestratorError::ValidationError { kind, .. } => kind.is_recoverable(),
            OrchestratorError::RuntimeError { kind, .. } => kind.is_recoverable(),
            OrchestratorError::ExternalServiceError { kind, .. } => kind.is_recoverable(),
            OrchestratorError::PanicError { kind, .. } => kind.is_recoverable(),
        }
    }
    
    /// Obtém código de erro
    pub fn error_code(&self) -> &'static str {
        match self {
            OrchestratorError::TaskNotFound(_) => "TASK_NOT_FOUND",
            OrchestratorError::CyclicDependency => "CYCLIC_DEPENDENCY",
            OrchestratorError::ResourceLimitExceeded(_) => "RESOURCE_LIMIT_EXCEEDED",
            OrchestratorError::NoActiveNodes => "NO_ACTIVE_NODES",
            OrchestratorError::LayerNotAvailable(_) => "LAYER_NOT_AVAILABLE",
            OrchestratorError::ModelNotFound(_) => "MODEL_NOT_FOUND",
            OrchestratorError::InsufficientData => "INSUFFICIENT_DATA",
            OrchestratorError::ConfigurationError(_) => "CONFIGURATION_ERROR",
            OrchestratorError::SerializationError(_) => "SERIALIZATION_ERROR",
            OrchestratorError::IoError(_) => "IO_ERROR",
            OrchestratorError::NetworkError(_) => "NETWORK_ERROR",
            OrchestratorError::DatabaseError(_) => "DATABASE_ERROR",
            OrchestratorError::AuthenticationError(_) => "AUTHENTICATION_ERROR",
            OrchestratorError::AuthorizationError(_) => "AUTHORIZATION_ERROR",
            OrchestratorError::Timeout(_) => "TIMEOUT",
            OrchestratorError::InvalidState(_) => "INVALID_STATE",
            OrchestratorError::UnsupportedOperation(_) => "UNSUPPORTED_OPERATION",
            OrchestratorError::ConsciousnessError(_) => "CONSCIOUSNESS_ERROR",
            OrchestratorError::QuantumError(_) => "QUANTUM_ERROR",
            OrchestratorError::InternalError(_) => "INTERNAL_ERROR",
            OrchestratorError::ExternalError(_) => "EXTERNAL_ERROR",
            OrchestratorError::ValidationError { .. } => "VALIDATION_ERROR",
            OrchestratorError::RuntimeError { .. } => "RUNTIME_ERROR",
            OrchestratorError::ExternalServiceError { .. } => "EXTERNAL_SERVICE_ERROR",
            OrchestratorError::PanicError { .. } => "PANIC_ERROR",
        }
    }
    
    /// Obtém categoria do erro
    pub fn category(&self) -> ErrorCategory {
        match self {
            OrchestratorError::TaskNotFound(_) => ErrorCategory::NotFound,
            OrchestratorError::CyclicDependency => ErrorCategory::Logic,
            OrchestratorError::ResourceLimitExceeded(_) => ErrorCategory::Resource,
            OrchestratorError::NoActiveNodes => ErrorCategory::Infrastructure,
            OrchestratorError::LayerNotAvailable(_) => ErrorCategory::Infrastructure,
            OrchestratorError::ModelNotFound(_) => ErrorCategory::NotFound,
            OrchestratorError::InsufficientData => ErrorCategory::Data,
            OrchestratorError::ConfigurationError(_) => ErrorCategory::Configuration,
            OrchestratorError::SerializationError(_) => ErrorCategory::Data,
            OrchestratorError::IoError(_) => ErrorCategory::System,
            OrchestratorError::NetworkError(_) => ErrorCategory::Network,
            OrchestratorError::DatabaseError(_) => ErrorCategory::Database,
            OrchestratorError::AuthenticationError(_) => ErrorCategory::Security,
            OrchestratorError::AuthorizationError(_) => ErrorCategory::Security,
            OrchestratorError::Timeout(_) => ErrorCategory::Performance,
            OrchestratorError::InvalidState(_) => ErrorCategory::Logic,
            OrchestratorError::UnsupportedOperation(_) => ErrorCategory::Logic,
            OrchestratorError::ConsciousnessError(_) => ErrorCategory::AI,
            OrchestratorError::QuantumError(_) => ErrorCategory::Quantum,
            OrchestratorError::InternalError(_) => ErrorCategory::System,
            OrchestratorError::ExternalError(_) => ErrorCategory::External,
            OrchestratorError::ValidationError { .. } => ErrorCategory::Logic,
            OrchestratorError::RuntimeError { .. } => ErrorCategory::System,
            OrchestratorError::ExternalServiceError { .. } => ErrorCategory::External,
            OrchestratorError::PanicError { .. } => ErrorCategory::System,
        }
    }
}

/// Categorias de erro
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Recurso não encontrado
    NotFound,
    /// Erro de lógica
    Logic,
    /// Limite de recursos
    Resource,
    /// Problema de infraestrutura
    Infrastructure,
    /// Problema com dados
    Data,
    /// Erro de configuração
    Configuration,
    /// Erro de sistema
    System,
    /// Erro de rede
    Network,
    /// Erro de banco de dados
    Database,
    /// Erro de segurança
    Security,
    /// Problema de performance
    Performance,
    /// Erro de IA/ML
    AI,
    /// Erro quântico
    Quantum,
    /// Erro externo
    External,
}

/// Trait para adicionar contexto a erros
pub trait WithErrorContext<T> {
    fn with_error_context(self, context: ErrorContext) -> SymbioticResult<T>;
}

impl<T> WithErrorContext<T> for SymbioticResult<T> {
    fn with_error_context(self, _context: ErrorContext) -> SymbioticResult<T> {
        self // For now, just pass through. Can be enhanced to wrap with context
    }
}

/// Implementação de WithContext para Result padrão
pub trait WithContext<T> {
    fn with_context(self, context: ErrorContext) -> Result<T>;
}

impl<T> WithContext<T> for Result<T> {
    fn with_context(self, _context: ErrorContext) -> Result<T> {
        self // For now, just pass through. Can be enhanced to wrap with context
    }
}

/// Sistema de Retry com Backoff Exponencial
#[derive(Debug)]
pub struct RetryManager {
    default_max_attempts: u32,
    default_exponential_base: f64,
    default_jitter_factor: f64,
    metrics: Arc<RwLock<RetryMetrics>>,
}

#[derive(Debug, Default, Clone)]
struct RetryMetrics {
    total_attempts: u64,
    successful_retries: u64,
    failed_retries: u64,
    total_backoff_time: Duration,
}

impl RetryManager {
    pub fn new(max_attempts: u32) -> Self {
        Self {
            default_max_attempts: max_attempts,
            default_exponential_base: 2.0,
            default_jitter_factor: 0.1,
            metrics: Arc::new(RwLock::new(RetryMetrics::default())),
        }
    }
    
    #[instrument(skip(self, operation))]
    pub async fn retry_with_backoff<T, F, Fut>(
        &self,
        operation: F,
        context: ErrorContext,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut retry_info = RetryInfo::new(self.default_max_attempts);
        
        loop {
            retry_info.record_attempt();
            
            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.total_attempts += 1;
                metrics.total_backoff_time += retry_info.backoff_duration;
            }
            
            info!(
                attempt = retry_info.attempt,
                max_attempts = retry_info.max_attempts,
                backoff_duration = ?retry_info.backoff_duration,
                trace_id = context.trace_id,
                "Attempting operation"
            );
            
            match operation().await {
                Ok(result) => {
                    if retry_info.attempt > 1 {
                        info!(
                            attempt = retry_info.attempt,
                            trace_id = context.trace_id,
                            "Operation succeeded after retry"
                        );
                        let mut metrics = self.metrics.write().await;
                        metrics.successful_retries += 1;
                    }
                    return Ok(result);
                }
                Err(err) => {
                    if !err.is_recoverable() || !retry_info.should_retry() {
                        error!(
                            attempt = retry_info.attempt,
                            max_attempts = retry_info.max_attempts,
                            recoverable = err.is_recoverable(),
                            trace_id = context.trace_id,
                            "Operation failed permanently"
                        );
                        let mut metrics = self.metrics.write().await;
                        metrics.failed_retries += 1;
                        return Err(OrchestratorError::RuntimeError {
                            component: context.component.clone(),
                            message: format!("Operation failed after {} attempts: {}", retry_info.attempt, err),
                            kind: ErrorKind::Runtime {
                                component: context.component.clone(),
                                operation: context.operation.clone(),
                                cause: err.to_string(),
                            },
                            context: context.clone(),
                            retry_info: Some(retry_info),
                        });
                    }
                    
                    warn!(
                        attempt = retry_info.attempt,
                        max_attempts = retry_info.max_attempts,
                        next_retry_in = ?retry_info.backoff_duration,
                        error = %err,
                        trace_id = context.trace_id,
                        "Operation failed, will retry"
                    );
                    
                    // Wait for backoff period
                    tokio::time::sleep(retry_info.backoff_duration).await;
                }
            }
        }
    }
    
    pub async fn get_metrics(&self) -> RetryMetrics {
        self.metrics.read().await.clone()
    }
}

/// Circuit Breaker para dependências externas
#[derive(Debug)]
pub struct CircuitBreaker {
    name: String,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_timeout: Duration,
    metrics: Arc<RwLock<CircuitBreakerMetrics>>,
}

#[derive(Debug, Default, Clone)]
struct CircuitBreakerMetrics {
    total_calls: u64,
    successful_calls: u64,
    failed_calls: u64,
    circuit_opens: u64,
    circuit_closes: u64,
}

impl CircuitBreaker {
    pub fn new(name: String, failure_threshold: u32, timeout_duration: Duration) -> Self {
        Self {
            name,
            state: Arc::new(RwLock::new(CircuitBreakerState::Closed)),
            failure_threshold,
            timeout_duration,
            half_open_timeout: Duration::from_secs(30),
            metrics: Arc::new(RwLock::new(CircuitBreakerMetrics::default())),
        }
    }
    
    #[instrument(skip(self, operation))]
    pub async fn call<T, F, Fut>(
        &self,
        operation: F,
        context: ErrorContext,
    ) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        // Check circuit state
        let current_state = {
            let mut state = self.state.write().await;
            match *state {
                CircuitBreakerState::Open { opened_at, failure_count } => {
                    if Utc::now().signed_duration_since(opened_at).to_std().unwrap() > self.timeout_duration {
                        *state = CircuitBreakerState::HalfOpen {
                            opened_at,
                            test_request_sent: false,
                        };
                        info!(
                            name = self.name,
                            trace_id = context.trace_id,
                            "Circuit breaker transitioning to half-open"
                        );
                    } else {
                        return Err(OrchestratorError::ExternalServiceError {
                            service: self.name.clone(),
                            message: "Circuit breaker is open".to_string(),
                            kind: ErrorKind::External {
                                service: self.name.clone(),
                                endpoint: context.operation.clone(),
                                status_code: None,
                            },
                            context,
                            circuit_breaker_state: state.clone(),
                        });
                    }
                }
                CircuitBreakerState::HalfOpen { test_request_sent: true, .. } => {
                    return Err(OrchestratorError::ExternalServiceError {
                        service: self.name.clone(),
                        message: "Circuit breaker is half-open with test request pending".to_string(),
                        kind: ErrorKind::External {
                            service: self.name.clone(),
                            endpoint: context.operation.clone(),
                            status_code: None,
                        },
                        context,
                        circuit_breaker_state: state.clone(),
                    });
                }
                CircuitBreakerState::HalfOpen { opened_at, .. } => {
                    *state = CircuitBreakerState::HalfOpen {
                        opened_at,
                        test_request_sent: true,
                    };
                }
                _ => {}
            }
            state.clone()
        };
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_calls += 1;
        }
        
        // Execute operation
        match operation().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(err) => {
                self.record_failure().await;
                Err(err)
            }
        }
    }
    
    async fn record_success(&self) {
        let mut state = self.state.write().await;
        let mut metrics = self.metrics.write().await;
        
        metrics.successful_calls += 1;
        
        if let CircuitBreakerState::HalfOpen { .. } = *state {
            *state = CircuitBreakerState::Closed;
            metrics.circuit_closes += 1;
            info!(
                name = self.name,
                "Circuit breaker closing after successful test"
            );
        }
    }
    
    async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let mut metrics = self.metrics.write().await;
        
        metrics.failed_calls += 1;
        
        match *state {
            CircuitBreakerState::Closed => {
                if metrics.failed_calls >= self.failure_threshold as u64 {
                    *state = CircuitBreakerState::Open {
                        opened_at: Utc::now(),
                        failure_count: metrics.failed_calls as u32,
                    };
                    metrics.circuit_opens += 1;
                    warn!(
                        name = self.name,
                        failure_threshold = self.failure_threshold,
                        failed_calls = metrics.failed_calls,
                        "Circuit breaker opening due to failures"
                    );
                }
            }
            CircuitBreakerState::HalfOpen { opened_at, .. } => {
                *state = CircuitBreakerState::Open {
                    opened_at,
                    failure_count: metrics.failed_calls as u32,
                };
                warn!(
                    name = self.name,
                    "Circuit breaker reopening after failed test"
                );
            }
            _ => {}
        }
    }
    
    pub async fn get_state(&self) -> CircuitBreakerState {
        self.state.read().await.clone()
    }
    
    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        self.metrics.read().await.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_properties() {
        let error = OrchestratorError::TaskNotFound(Uuid::new_v4());
        
        assert!(!error.is_recoverable());
        assert_eq!(error.error_code(), "TASK_NOT_FOUND");
        assert_eq!(error.category(), ErrorCategory::NotFound);
    }
    
    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation", "test_component")
            .with_metadata("key", "value");
        
        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.component, "test_component");
        assert!(context.metadata.contains_key("key"));
    }
    
    #[test]
    fn test_with_context() {
        let result: Result<i32> = Err(OrchestratorError::InternalError("test".to_string()));
        let context = ErrorContext::new("test", "test");
        
        let context_result = result.with_context(context);
        assert!(context_result.is_err());
    }
}

    #[test]
    fn test_retry_info() {
        let mut retry_info = RetryInfo::new(3);
        
        assert_eq!(retry_info.attempt, 0);
        assert!(retry_info.should_retry());
        
        retry_info.record_attempt();
        assert_eq!(retry_info.attempt, 1);
        assert!(retry_info.should_retry());
        
        retry_info.record_attempt();
        retry_info.record_attempt();
        assert_eq!(retry_info.attempt, 3);
        assert!(!retry_info.should_retry());
    }


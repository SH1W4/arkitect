//! # Task Mesh IA Orchestrator Core
//!
//! Sistema de orquestração simbiótica para grafos de tarefas (DAG) com múltiplas camadas:
//! - Local: Execução em máquina local
//! - Cluster: Distribuição em cluster
//! - Quantum-Sim: Simulação quântica
//!
//! Inclui módulos de consciência simbiótica e aprendizado contínuo.

pub mod core;
pub mod graph;
pub mod layers;
pub mod symbiotic;
pub mod learning;
pub mod errors;
pub mod config;
pub mod metrics;
pub mod backup;

// Re-exports principais
pub use crate::core::{OrchestratorCore, TaskExecutionResult};
pub use crate::graph::{TaskMesh, TaskNode, DependencyEdge};
pub use crate::layers::{ExecutionLayer, LocalLayer, ClusterLayer, QuantumSimLayer};
pub use crate::symbiotic::{SymbioticConsciousness, ConsciousnessState};
pub use crate::learning::{ContinuousLearning, LearningMetrics};
pub use crate::errors::{OrchestratorError, Result};
pub use crate::config::OrchestratorConfig;
pub use crate::metrics::SystemMetrics;

/// Resultado padrão para operações do orchestrator
pub type OrchestratorResult<T> = std::result::Result<T, OrchestratorError>;

/// Versão da API do orchestrator
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_initialization() {
        let config = OrchestratorConfig::default();
        let orchestrator = OrchestratorCore::new(config).await;
        assert!(orchestrator.is_ok());
    }
}
    
    #[tokio::test]
    async fn test_error_handling_integration() {
        use crate::errors::{ErrorContext, RetryManager, WithContext};
        
        let retry_manager = RetryManager::new(2);
        let context = ErrorContext::new("test_operation", "test_component");
        
        let mut attempts = 0;
        let result = retry_manager.retry_with_backoff(
            || {
                attempts += 1;
                async move {
                    if attempts == 1 {
                        Err(OrchestratorError::InternalError("test failure".to_string()))
                    } else {
                        Ok("success".to_string())
                    }
                }
            },
            context,
        ).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert_eq!(attempts, 2);
    }

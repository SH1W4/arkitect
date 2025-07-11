//! # Demo do Sistema de Tratamento de Erros Robusto
//!
//! Este exemplo demonstra o uso do framework de tratamento de erros com:
//! - ErrorKind enum para categorização
//! - Retry com backoff exponencial
//! - Circuit breaker para dependências externas
//! - Logging contextual com tracing + JSON

use orchestrator_core::{
    OrchestratorCore, OrchestratorConfig, ErrorContext, RetryManager, CircuitBreaker,
    OrchestratorError, ErrorKind, WithContext,
};
use std::time::Duration;
use tracing::{info, warn, error};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configurar logging estruturado
    tracing_subscriber::registry()
        .with(fmt::layer().json())
        .init();

    info!("Iniciando demo do sistema de tratamento de erros");

    // Demo 1: Validação de Erro
    demo_validation_error().await;

    // Demo 2: Retry com Backoff
    demo_retry_with_backoff().await;

    // Demo 3: Circuit Breaker
    demo_circuit_breaker().await;

    // Demo 4: Logging Contextual
    demo_contextual_logging().await;

    // Demo 5: Sistema Integrado
    demo_integrated_system().await;

    Ok(())
}

/// Demonstra validação de erro com ErrorKind
async fn demo_validation_error() {
    info!("=== Demo 1: Validação de Erro ===");
    
    let context = ErrorContext::new("validate_input", "demo_component")
        .with_metadata("user_id", "12345")
        .with_metadata("request_id", "req-001");

    let validation_error = OrchestratorError::ValidationError {
        field: "email".to_string(),
        message: "Invalid email format".to_string(),
        kind: ErrorKind::Validation {
            field: "email".to_string(),
            rule: "email_format".to_string(),
            value: "invalid-email".to_string(),
        },
        context: context.clone(),
    };

    validation_error.log_error();
    
    info!(
        error_recoverable = validation_error.is_recoverable(),
        error_code = validation_error.error_code(),
        error_category = ?validation_error.category(),
        "Validation error processed"
    );
}

/// Demonstra retry com backoff exponencial
async fn demo_retry_with_backoff() {
    info!("=== Demo 2: Retry com Backoff ===");
    
    let retry_manager = RetryManager::new(3);
    let context = ErrorContext::new("external_api_call", "retry_demo")
        .with_metadata("api_endpoint", "https://api.example.com/data");

    let mut attempt_count = 0;
    let result = retry_manager.retry_with_backoff(
        || {
            attempt_count += 1;
            async move {
                info!(attempt = attempt_count, "Attempting API call");
                
                if attempt_count < 3 {
                    Err(OrchestratorError::NetworkError(
                        reqwest::Error::from(std::io::Error::new(
                            std::io::ErrorKind::ConnectionRefused,
                            "Connection refused"
                        ))
                    ))
                } else {
                    Ok("API call successful".to_string())
                }
            }
        },
        context.clone(),
    ).await;

    match result {
        Ok(success_msg) => info!(message = success_msg, "Retry succeeded"),
        Err(err) => {
            err.log_error();
            error!("Retry failed permanently");
        }
    }

    let metrics = retry_manager.get_metrics().await;
    info!(
        total_attempts = metrics.total_attempts,
        successful_retries = metrics.successful_retries,
        failed_retries = metrics.failed_retries,
        "Retry metrics"
    );
}

/// Demonstra circuit breaker para dependências
async fn demo_circuit_breaker() {
    info!("=== Demo 3: Circuit Breaker ===");
    
    let circuit_breaker = CircuitBreaker::new(
        "external_service".to_string(),
        2, // failure threshold
        Duration::from_secs(60), // timeout
    );

    // Simula múltiplas chamadas falhando
    for i in 1..=5 {
        let context = ErrorContext::new("service_call", "circuit_breaker_demo")
            .with_metadata("call_number", &i.to_string());

        let result = circuit_breaker.call(
            || async {
                if i <= 3 {
                    Err(OrchestratorError::ExternalError(
                        anyhow::anyhow!("Service unavailable")
                    ))
                } else {
                    Ok(format!("Success on call {}", i))
                }
            },
            context.clone(),
        ).await;

        match result {
            Ok(msg) => info!(call = i, message = msg, "Call succeeded"),
            Err(err) => {
                err.log_error();
                warn!(call = i, "Call failed");
            }
        }

        let state = circuit_breaker.get_state().await;
        info!(call = i, circuit_state = ?state, "Circuit breaker state");

        // Pequena pausa entre chamadas
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    let metrics = circuit_breaker.get_metrics().await;
    info!(
        total_calls = metrics.total_calls,
        successful_calls = metrics.successful_calls,
        failed_calls = metrics.failed_calls,
        circuit_opens = metrics.circuit_opens,
        "Circuit breaker metrics"
    );
}

/// Demonstra logging contextual estruturado
async fn demo_contextual_logging() {
    info!("=== Demo 4: Logging Contextual ===");
    
    let context = ErrorContext::new("process_transaction", "payment_service")
        .with_metadata("transaction_id", "txn-789")
        .with_metadata("amount", "100.50")
        .with_metadata("currency", "USD")
        .with_user("user-456".to_string())
        .with_request("req-999".to_string());

    // Log do contexto em JSON
    let context_json = context.to_json().unwrap();
    info!(context = context_json, "Transaction context");

    // Diferentes tipos de erro com contexto
    let runtime_error = OrchestratorError::RuntimeError {
        component: "payment_processor".to_string(),
        message: "Database connection timeout".to_string(),
        kind: ErrorKind::Runtime {
            component: "payment_processor".to_string(),
            operation: "save_transaction".to_string(),
            cause: "Connection timeout after 30 seconds".to_string(),
        },
        context: context.clone(),
        retry_info: None,
    };

    runtime_error.log_error();

    let external_error = OrchestratorError::ExternalServiceError {
        service: "fraud_detection".to_string(),
        message: "Service temporarily unavailable".to_string(),
        kind: ErrorKind::External {
            service: "fraud_detection".to_string(),
            endpoint: "/api/v1/check".to_string(),
            status_code: Some(503),
        },
        context: context.clone(),
        circuit_breaker_state: orchestrator_core::CircuitBreakerState::Open {
            opened_at: chrono::Utc::now(),
            failure_count: 5,
        },
    };

    external_error.log_error();
}

/// Demonstra sistema integrado com todos os componentes
async fn demo_integrated_system() {
    info!("=== Demo 5: Sistema Integrado ===");
    
    let config = OrchestratorConfig::default();
    let mut orchestrator = OrchestratorCore::new(config).await
        .expect("Failed to create orchestrator");

    // Registra circuit breakers para serviços externos
    orchestrator.register_circuit_breaker(
        "database".to_string(),
        3,
        Duration::from_secs(30),
    );
    
    orchestrator.register_circuit_breaker(
        "external_api".to_string(),
        2,
        Duration::from_secs(60),
    );

    // Simula operação complexa com múltiplos pontos de falha
    let context = ErrorContext::new("complex_operation", "integrated_demo")
        .with_metadata("operation_id", "op-123")
        .with_user("admin".to_string());

    // Operação com database usando circuit breaker
    let db_result = orchestrator.execute_with_circuit_breaker(
        "database",
        || async {
            info!("Executing database operation");
            // Simula operação de database
            Ok("Database operation successful".to_string())
        },
        context.clone(),
    ).await;

    match db_result {
        Ok(msg) => info!(message = msg, "Database operation completed"),
        Err(err) => {
            err.log_error();
            error!("Database operation failed");
        }
    }

    // Operação com API externa usando circuit breaker
    let api_result = orchestrator.execute_with_circuit_breaker(
        "external_api",
        || async {
            info!("Executing external API call");
            // Simula chamada para API externa
            Err(OrchestratorError::NetworkError(
                reqwest::Error::from(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Request timeout"
                ))
            ))
        },
        context.clone(),
    ).await;

    match api_result {
        Ok(msg) => info!(message = msg, "API call completed"),
        Err(err) => {
            err.log_error();
            error!("API call failed");
        }
    }

    info!("Demo do sistema integrado concluído");
}


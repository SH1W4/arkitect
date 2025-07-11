# Exemplos do Sistema de Tratamento de Erros Simbiótico

## Visão Geral

Este diretório contém exemplos práticos do framework robusto de tratamento de erros implementado no SYMBIOTIC_METHOD. O sistema oferece:

### 🎯 Características Principais

1. **ErrorKind Enum Estruturado**
   - `Validation`: Erros de validação de entrada
   - `Runtime`: Erros de tempo de execução
   - `External`: Erros de serviços externos
   - `Panic`: Erros críticos do sistema

2. **Sistema de Retry com Backoff Exponencial**
   - Retry automático para erros recuperáveis
   - Backoff exponencial com jitter
   - Métricas de tentativas e sucesso
   - Configuração flexível de tentativas

3. **Circuit Breaker para Dependências**
   - Proteção contra cascata de falhas
   - Estados: Closed, Open, Half-Open
   - Métricas de saúde dos serviços
   - Configuração de thresholds e timeouts

4. **Logging Contextual Estruturado**
   - Integração com tracing
   - Logs em formato JSON
   - Contexto rico com metadados
   - Trace IDs para correlação

## 🚀 Executando os Exemplos

### Demo Completo

```bash
cd orchestrator_core
cargo run --example error_handling_demo
```

### Configuração de Logging

Para ver logs detalhados:

```bash
RUST_LOG=debug cargo run --example error_handling_demo
```

Para logs em formato JSON:

```bash
RUST_LOG=info cargo run --example error_handling_demo 2>&1 | jq
```

## 📋 Estrutura dos Exemplos

### 1. Demo de Validação

Monstra como criar e usar erros de validação:

```rust
let validation_error = OrchestratorError::ValidationError {
    field: "email".to_string(),
    message: "Invalid email format".to_string(),
    kind: ErrorKind::Validation {
        field: "email".to_string(),
        rule: "email_format".to_string(),
        value: "invalid-email".to_string(),
    },
    context: error_context,
};
```

### 2. Demo de Retry

Demonstra retry automático com backoff:

```rust
let retry_manager = RetryManager::new(3);
let result = retry_manager.retry_with_backoff(
    || async { /* operação que pode falhar */ },
    context,
).await;
```

### 3. Demo de Circuit Breaker

Mostra proteção contra falhas em cascata:

```rust
let circuit_breaker = CircuitBreaker::new(
    "external_service".to_string(),
    2, // failure threshold
    Duration::from_secs(60), // timeout
);

let result = circuit_breaker.call(
    || async { /* chamada para serviço externo */ },
    context,
).await;
```

### 4. Demo de Logging Contextual

Ilustra logging estruturado com contexto rico:

```rust
let context = ErrorContext::new("operation", "component")
    .with_metadata("transaction_id", "txn-123")
    .with_user("user-456".to_string())
    .with_request("req-789".to_string());

error.log_error(); // Log estruturado automático
```

## 🔧 Configuração Avançada

### Retry Manager Personalizado

```rust
let retry_manager = RetryManager::new(5) // 5 tentativas máximas
    .with_exponential_base(1.5) // Base menor para backoff
    .with_jitter_factor(0.2); // Mais jitter
```

### Circuit Breaker Avançado

```rust
let circuit_breaker = CircuitBreaker::new(
    "critical_service".to_string(),
    1, // Falha uma vez = abre circuito
    Duration::from_secs(30), // Timeout curto
)
.with_half_open_timeout(Duration::from_secs(10));
```

### Contexto Rico

```rust
let context = ErrorContext::new("payment_processing", "payment_service")
    .with_metadata("amount", "100.50")
    .with_metadata("currency", "USD")
    .with_metadata("merchant_id", "merchant-123")
    .with_user(user_id)
    .with_request(request_id)
    .with_trace(trace_id, Some(span_id));
```

## 📊 Métricas e Monitoramento

### Métricas de Retry

```rust
let metrics = retry_manager.get_metrics().await;
println!("Total attempts: {}", metrics.total_attempts);
println!("Successful retries: {}", metrics.successful_retries);
println!("Failed retries: {}", metrics.failed_retries);
```

### Métricas de Circuit Breaker

```rust
let metrics = circuit_breaker.get_metrics().await;
println!("Total calls: {}", metrics.total_calls);
println!("Success rate: {:.2}%", 
    (metrics.successful_calls as f64 / metrics.total_calls as f64) * 100.0);
```

## 🎨 Patterns de Uso

### Pattern 1: Operação com Fallback

```rust
let result = retry_manager.retry_with_backoff(
    || circuit_breaker.call(
        || primary_operation(),
        context.clone()
    ),
    context.clone()
).await;

let final_result = match result {
    Ok(value) => value,
    Err(_) => fallback_operation().await?,
};
```

### Pattern 2: Agregação de Erros

```rust
let mut errors = Vec::new();

for service in services {
    match call_service(service).await {
        Ok(_) => {},
        Err(e) => {
            e.log_error();
            errors.push(e);
        }
    }
}

if !errors.is_empty() {
    return Err(OrchestratorError::RuntimeError {
        component: "service_aggregator".to_string(),
        message: format!("Failed to call {} services", errors.len()),
        // ... outros campos
    });
}
```

### Pattern 3: Timeout com Context

```rust
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(30),
    circuit_breaker.call(
        || external_api_call(),
        context.clone()
    )
).await;

match result {
    Ok(Ok(value)) => Ok(value),
    Ok(Err(e)) => Err(e),
    Err(_) => Err(OrchestratorError::Timeout(
        "External API call timed out".to_string()
    )),
}
```

## 🧪 Testes

Para executar testes dos componentes de error handling:

```bash
cargo test errors::
```

Para testes com output detalhado:

```bash
cargo test errors:: -- --nocapture
```

## 📝 Logs de Exemplo

### Log de Erro de Validação

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "WARN",
  "fields": {
    "error_code": "VALIDATION_ERROR",
    "category": "Logic",
    "field": "email",
    "message": "Invalid email format",
    "trace_id": "a1b2c3d4-e5f6-7890-1234-567890abcdef",
    "component": "demo_component",
    "operation": "validate_input"
  },
  "target": "orchestrator_core::errors"
}
```

### Log de Retry

```json
{
  "timestamp": "2024-01-15T10:30:46.456Z",
  "level": "INFO",
  "fields": {
    "attempt": 2,
    "max_attempts": 3,
    "backoff_duration": "400ms",
    "trace_id": "a1b2c3d4-e5f6-7890-1234-567890abcdef"
  },
  "target": "orchestrator_core::errors",
  "message": "Attempting operation"
}
```

### Log de Circuit Breaker

```json
{
  "timestamp": "2024-01-15T10:30:47.789Z",
  "level": "WARN",
  "fields": {
    "name": "external_service",
    "failure_threshold": 2,
    "failed_calls": 2
  },
  "target": "orchestrator_core::errors",
  "message": "Circuit breaker opening due to failures"
}
```

## 🔗 Integração com Monitoramento

O sistema pode ser integrado com ferramentas de monitoramento:

- **Prometheus**: Métricas de retry e circuit breaker
- **Grafana**: Dashboards de saúde dos serviços
- **Jaeger**: Tracing distribuído
- **ELK Stack**: Logs estruturados

### Exemplo de Métricas Prometheus

```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref RETRY_ATTEMPTS: Counter = register_counter!(
        "symbiotic_retry_attempts_total",
        "Total number of retry attempts"
    ).unwrap();
    
    static ref CIRCUIT_BREAKER_STATE: Histogram = register_histogram!(
        "symbiotic_circuit_breaker_state_duration_seconds",
        "Time spent in each circuit breaker state"
    ).unwrap();
}
```

Este sistema de tratamento de erros fornece uma base sólida para construir aplicações resilientes e observáveis no ecossistema SYMBIOTIC_METHOD.


# 🛡️ Framework Robusto de Tratamento de Erros - SYMBIOTIC_METHOD

## 📋 Visão Geral

Este documento descreve a implementação completa do framework robusto de tratamento de erros para o projeto SYMBIOTIC_METHOD, conforme especificado no Step 7 do plano de desenvolvimento.

### ✅ Componentes Implementados

1. **ErrorKind Enum** - Categorização estruturada de erros
2. **Estratégia Retry + Backoff** - Retry automático com backoff exponencial
3. **Circuit-Breaker** - Proteção para dependências externas
4. **Logging Contextual** - Tracing + JSON estruturado

## 🎯 ErrorKind Enum

### Estrutura

```rust
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
```

### Funcionalidades

- **Recuperabilidade**: Método `is_recoverable()` para determinar se o erro pode ser recuperado
- **Severidade**: Método `severity()` retorna nível de severidade (Info, Warning, Error, Critical)
- **Telemetria**: Método `telemetry_category()` para categorização em sistemas de monitoramento

### Exemplo de Uso

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

// Verifica se é recuperável
if !validation_error.is_recoverable() {
    // Erro não recuperável, falha imediata
    return Err(validation_error);
}
```

## 🔄 Sistema de Retry com Backoff Exponencial

### Características

- **Backoff Exponencial**: Base configurável (padrão: 2.0)
- **Jitter**: Redução de efeito "thundering herd" (padrão: 10%)
- **Métricas**: Rastreamento de tentativas, sucessos e falhas
- **Configuração Flexível**: Número de tentativas e parâmetros ajustáveis

### Implementação

```rust
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

pub struct RetryManager {
    default_max_attempts: u32,
    default_exponential_base: f64,
    default_jitter_factor: f64,
    metrics: Arc<RwLock<RetryMetrics>>,
}
```

### Uso Prático

```rust
let retry_manager = RetryManager::new(3);
let context = ErrorContext::new("api_call", "service_client");

let result = retry_manager.retry_with_backoff(
    || async {
        // Operação que pode falhar
        external_api_call().await
    },
    context,
).await;

match result {
    Ok(data) => println!("Success: {:?}", data),
    Err(err) => {
        err.log_error();
        println!("Failed after retries: {}", err);
    }
}
```

### Fórmula de Backoff

```
delay = base_delay * (exponential_base ^ (attempt - 1)) * jitter_factor
```

Onde:
- `base_delay`: 100ms (inicial)
- `exponential_base`: 2.0 (padrão)
- `jitter_factor`: 0.9 a 1.1 (10% de variação)

## ⚡ Circuit Breaker

### Estados

1. **Closed**: Operação normal, todas as requisições passam
2. **Open**: Falhas detectadas, requisições bloqueadas
3. **Half-Open**: Teste de recuperação, uma requisição de teste permitida

### Implementação

```rust
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

pub struct CircuitBreaker {
    name: String,
    state: Arc<RwLock<CircuitBreakerState>>,
    failure_threshold: u32,
    timeout_duration: Duration,
    half_open_timeout: Duration,
    metrics: Arc<RwLock<CircuitBreakerMetrics>>,
}
```

### Configuração

```rust
let circuit_breaker = CircuitBreaker::new(
    "payment_service".to_string(),
    5, // failure_threshold: abre após 5 falhas
    Duration::from_secs(60), // timeout: tenta reabrir após 60s
);
```

### Uso Integrado

```rust
// Registro no orchestrator
orchestrator.register_circuit_breaker(
    "external_api".to_string(),
    3,
    Duration::from_secs(30),
);

// Uso
let result = orchestrator.execute_with_circuit_breaker(
    "external_api",
    || external_service_call(),
    error_context,
).await;
```

### Transições de Estado

```
CLOSED --[failures >= threshold]--> OPEN
OPEN --[timeout elapsed]--> HALF_OPEN
HALF_OPEN --[success]--> CLOSED
HALF_OPEN --[failure]--> OPEN
```

## 📊 Logging Contextual (Tracing + JSON)

### ErrorContext

```rust
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
```

### Criação de Contexto

```rust
let context = ErrorContext::new("payment_processing", "payment_service")
    .with_metadata("transaction_id", "txn-123")
    .with_metadata("amount", "100.50")
    .with_metadata("currency", "USD")
    .with_user("user-456".to_string())
    .with_request("req-789".to_string())
    .with_trace(trace_id, Some(span_id));
```

### Logging Automático

```rust
// Log estruturado automático
error.log_error();
```

### Exemplo de Log JSON

```json
{
  "timestamp": "2024-01-15T10:30:45.123Z",
  "level": "ERROR",
  "fields": {
    "error_code": "RUNTIME_ERROR",
    "category": "System",
    "component": "payment_processor",
    "message": "Database connection timeout",
    "trace_id": "a1b2c3d4-e5f6-7890-1234-567890abcdef",
    "user_id": "user-456",
    "request_id": "req-789",
    "transaction_id": "txn-123",
    "retry_attempt": 2
  },
  "target": "orchestrator_core::errors",
  "message": "Runtime error occurred"
}
```

### Configuração de Tracing

```rust
// Desenvolvimento - logs legíveis
tracing_subscriber::fmt::init();

// Produção - logs JSON estruturados
tracing_subscriber::registry()
    .with(fmt::layer().json())
    .init();
```

## 🔧 Integração no OrchestratorCore

### Adições na Estrutura

```rust
pub struct OrchestratorCore {
    // ... campos existentes ...
    retry_manager: RetryManager,
    circuit_breakers: HashMap<String, CircuitBreaker>,
}
```

### Métodos Adicionados

```rust
impl OrchestratorCore {
    // Registra circuit breaker
    pub fn register_circuit_breaker(
        &mut self, 
        service_name: String, 
        failure_threshold: u32, 
        timeout: Duration
    ) { ... }
    
    // Executa com circuit breaker
    pub async fn execute_with_circuit_breaker<T, F, Fut>(
        &self,
        service_name: &str,
        operation: F,
        context: ErrorContext,
    ) -> Result<T> { ... }
    
    // Métodos existentes agora usam contexto
    pub async fn execute_task(
        &self,
        task_id: TaskId,
        layer: ExecutionLayer,
    ) -> Result<TaskExecutionResult> {
        let context = ErrorContext::new("execute_task", "orchestrator_core")
            .with_metadata("task_id", &task_id.to_string())
            .with_metadata("layer", &format!("{:?}", layer));
        
        // Usa retry manager automaticamente
        self.retry_manager.retry_with_backoff(
            || execution_layer.execute_task(&task),
            context,
        ).await
    }
}
```

## 📈 Métricas e Monitoramento

### Métricas de Retry

```rust
#[derive(Debug, Default, Clone)]
struct RetryMetrics {
    total_attempts: u64,
    successful_retries: u64,
    failed_retries: u64,
    total_backoff_time: Duration,
}
```

### Métricas de Circuit Breaker

```rust
#[derive(Debug, Default, Clone)]
struct CircuitBreakerMetrics {
    total_calls: u64,
    successful_calls: u64,
    failed_calls: u64,
    circuit_opens: u64,
    circuit_closes: u64,
}
```

### Exportação para Prometheus

```rust
use prometheus::{Counter, Histogram, register_counter, register_histogram};

lazy_static! {
    static ref RETRY_ATTEMPTS: Counter = register_counter!(
        "symbiotic_retry_attempts_total",
        "Total number of retry attempts"
    ).unwrap();
    
    static ref CIRCUIT_BREAKER_STATE_DURATION: Histogram = register_histogram!(
        "symbiotic_circuit_breaker_state_duration_seconds",
        "Time spent in each circuit breaker state"
    ).unwrap();
}
```

## 🧪 Testes

### Testes Unitários

```bash
# Executar todos os testes de error handling
cargo test errors::

# Testes específicos
cargo test test_retry_info
cargo test test_circuit_breaker_states
cargo test test_error_context
```

### Testes de Integração

```bash
# Demo completo
cargo run --example error_handling_demo

# Com logs JSON
RUST_LOG=info cargo run --example error_handling_demo 2>&1 | jq
```

### Coverage de Testes

- ✅ ErrorKind enum e métodos
- ✅ RetryInfo e RetryManager
- ✅ CircuitBreaker estados e transições
- ✅ ErrorContext criação e serialização
- ✅ Integração com OrchestratorCore
- ✅ Logging estruturado

## 🚀 Exemplos de Uso

### Pattern 1: Operação Simples com Retry

```rust
let retry_manager = RetryManager::new(3);
let result = retry_manager.retry_with_backoff(
    || database_operation(),
    ErrorContext::new("db_query", "data_service"),
).await?;
```

### Pattern 2: Serviço Externo com Circuit Breaker

```rust
let circuit_breaker = CircuitBreaker::new(
    "payment_gateway".to_string(),
    5,
    Duration::from_secs(30),
);

let result = circuit_breaker.call(
    || payment_api_call(),
    ErrorContext::new("payment", "payment_service"),
).await?;
```

### Pattern 3: Operação Crítica com Fallback

```rust
let primary_result = retry_manager.retry_with_backoff(
    || circuit_breaker.call(
        || primary_service_call(),
        context.clone()
    ),
    context.clone()
).await;

let result = match primary_result {
    Ok(data) => data,
    Err(err) => {
        err.log_error();
        warn!("Primary service failed, using fallback");
        fallback_service_call().await?
    }
};
```

### Pattern 4: Agregação de Múltiplos Serviços

```rust
let mut results = Vec::new();
let mut errors = Vec::new();

for service in services {
    let context = ErrorContext::new("service_call", "aggregator")
        .with_metadata("service_name", &service.name);
    
    match orchestrator.execute_with_circuit_breaker(
        &service.name,
        || service.call(),
        context
    ).await {
        Ok(result) => results.push(result),
        Err(err) => {
            err.log_error();
            errors.push(err);
        }
    }
}

if !errors.is_empty() {
    warn!("Some services failed: {}/{}", errors.len(), services.len());
}
```

## 📋 Checklist de Implementação

### ✅ Concluído

- [x] ErrorKind enum com 4 categorias (Validation, Runtime, External, Panic)
- [x] Sistema de retry com backoff exponencial e jitter
- [x] Circuit breaker com 3 estados (Closed, Open, Half-Open)
- [x] Logging contextual com tracing e JSON
- [x] Integração no OrchestratorCore
- [x] Métricas de retry e circuit breaker
- [x] Testes unitários e de integração
- [x] Exemplo funcional completo
- [x] Documentação detalhada

### 🔄 Melhorias Futuras

- [ ] Métricas Prometheus automáticas
- [ ] Dashboard Grafana para monitoramento
- [ ] Configuração via arquivo externo
- [ ] Rate limiting integrado
- [ ] Bulkhead pattern para isolamento
- [ ] Timeout configurável por operação
- [ ] Alertas automáticos baseados em thresholds

## 🎯 Benefícios Alcançados

1. **Resiliência**: Sistema resistente a falhas temporárias
2. **Observabilidade**: Logs estruturados e métricas detalhadas
3. **Manutenibilidade**: Código organizado e bem documentado
4. **Performance**: Evita sobrecarga em serviços com falha
5. **Debugabilidade**: Contexto rico para investigação
6. **Escalabilidade**: Patterns proven para sistemas distribuídos

Este framework fornece uma base sólida para construir aplicações resilientes e observáveis no ecossistema SYMBIOTIC_METHOD, alinhado com as melhores práticas da indústria para tratamento robusto de erros.


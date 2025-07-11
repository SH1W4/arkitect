# 🧠 Orchestrator Core - SYMBIOTIC_METHOD

## 📋 Visão Geral

O Orchestrator Core é o núcleo do sistema de orquestração simbiótica do projeto SYMBIOTIC_METHOD. Ele implementa um framework robusto para execução de tarefas em grafos direcionados acíclicos (DAG) com capacidades de consciência simbiótica e tratamento avançado de erros.

## 🎯 Funcionalidades Principais

### 🛡️ Framework de Tratamento de Erros

- **ErrorKind Enum**: Categorização estruturada (Validation, Runtime, External, Panic)
- **Retry com Backoff**: Sistema automático com backoff exponencial e jitter
- **Circuit Breaker**: Proteção contra cascata de falhas
- **Logging Contextual**: Tracing estruturado com JSON

### 🌐 Camadas de Execução

- **Local**: Execução em máquina local
- **Cluster**: Distribuição em cluster
- **Quantum-Sim**: Simulação quântica (experimental)

### 🧠 Consciência Simbiótica

- **Níveis de Consciência**: Basic, Cognitive, Metacognitive, Quantum, Transcendent
- **Aprendizado Contínuo**: Reconhecimento de padrões e evolução
- **Memória Episódica**: Histórico de decisões e outcomes

## 🚀 Instalação e Uso

### Dependências

```toml
[dependencies]
orchestrator_core = { path = "path/to/orchestrator_core" }
tokio = { version = "1.0", features = ["full"] }
tracing = "0.1"
tracing-subscriber = "0.3"
```

### Exemplo Básico

```rust
use orchestrator_core::{
    OrchestratorCore, OrchestratorConfig, ErrorContext,
    TaskNode, ExecutionLayer
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Inicializar logging
    tracing_subscriber::fmt::init();
    
    // Criar orchestrator
    let config = OrchestratorConfig::default();
    let mut orchestrator = OrchestratorCore::new(config).await?;
    
    // Registrar circuit breakers
    orchestrator.register_circuit_breaker(
        "external_api".to_string(),
        3, // failure threshold
        Duration::from_secs(60), // timeout
    );
    
    // Criar e executar tarefa
    let task = TaskNode::new("example_task".to_string())
        .with_description("Example task execution".to_string());
    
    let task_id = orchestrator.add_task(task).await?;
    let result = orchestrator.execute_task(task_id, ExecutionLayer::Local).await?;
    
    println!("Task result: {:?}", result);
    Ok(())
}
```

### Tratamento de Erros Avançado

```rust
use orchestrator_core::{
    ErrorContext, RetryManager, CircuitBreaker, WithContext
};

// Criar contexto de erro
let context = ErrorContext::new("payment_processing", "payment_service")
    .with_metadata("transaction_id", "txn-123")
    .with_user("user-456".to_string());

// Retry automático
let retry_manager = RetryManager::new(3);
let result = retry_manager.retry_with_backoff(
    || async { external_api_call().await },
    context.clone(),
).await?;

// Circuit breaker
let circuit_breaker = CircuitBreaker::new(
    "payment_gateway".to_string(),
    5, // threshold
    Duration::from_secs(30), // timeout
);

let protected_result = circuit_breaker.call(
    || async { payment_gateway_call().await },
    context,
).await?;
```

## 📊 Monitoramento e Métricas

### Métricas Disponíveis

- **Retry Metrics**: Total de tentativas, sucessos, falhas
- **Circuit Breaker Metrics**: Chamadas, aberturas, fechamentos
- **Task Execution Metrics**: Tempo de execução, sucessos, falhas
- **Consciousness Metrics**: Nível de consciência, insights gerados

### Exemplo de Coleta

```rust
// Métricas de retry
let retry_metrics = retry_manager.get_metrics().await;
println!("Retry success rate: {:.2}%", 
    (retry_metrics.successful_retries as f64 / retry_metrics.total_attempts as f64) * 100.0
);

// Métricas de circuit breaker
let cb_metrics = circuit_breaker.get_metrics().await;
println!("Circuit breaker success rate: {:.2}%",
    (cb_metrics.successful_calls as f64 / cb_metrics.total_calls as f64) * 100.0
);
```

## 🧪 Testes

### Executar Testes

```bash
# Todos os testes
cargo test

# Testes de error handling
cargo test errors::

# Testes de consciência simbiótica
cargo test symbiotic::

# Testes com output detalhado
cargo test -- --nocapture
```

### Demo Completo

```bash
# Executar demo de error handling
cargo run --example error_handling_demo

# Com logs JSON estruturados
RUST_LOG=info cargo run --example error_handling_demo 2>&1 | jq
```

## 📁 Estrutura do Projeto

```
orchestrator_core/
├── src/
│   ├── lib.rs              # Exports principais
│   ├── core.rs             # OrchestratorCore
│   ├── errors.rs           # Framework de erros
│   ├── graph.rs            # TaskMesh e DAG
│   ├── layers.rs           # Camadas de execução
│   ├── symbiotic.rs        # Consciência simbiótica
│   ├── learning.rs         # Aprendizado contínuo
│   ├── config.rs           # Configuração
│   └── metrics.rs          # Sistema de métricas
├── examples/
│   ├── error_handling_demo.rs    # Demo completo
│   └── README.md                  # Documentação dos exemplos
├── Cargo.toml
└── README.md
```

## ⚙️ Configuração

### Arquivo de Configuração

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub max_concurrent_tasks: usize,
    pub default_timeout: Duration,
    pub retry_attempts: u32,
    pub circuit_breaker_threshold: u32,
    pub consciousness_level: AwarenessLevel,
    pub enable_quantum_simulation: bool,
    pub cluster_nodes: Vec<String>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 100,
            default_timeout: Duration::from_secs(30),
            retry_attempts: 3,
            circuit_breaker_threshold: 5,
            consciousness_level: AwarenessLevel::Basic,
            enable_quantum_simulation: false,
            cluster_nodes: vec![],
        }
    }
}
```

### Variáveis de Ambiente

```bash
# Nível de log
RUST_LOG=info

# Configuração de retry
SYMBIOTIC_RETRY_ATTEMPTS=5
SYMBIOTIC_BACKOFF_BASE=1.5

# Circuit breaker
SYMBIOTIC_CB_THRESHOLD=3
SYMBIOTIC_CB_TIMEOUT=30
```

## 🔗 Integração com Ecossistema

### Quantum Bridge (Opcional)

```rust
[features]
default = ["symbiotic-consciousness"]
symbiotic-consciousness = ["quantum-bridge", "vireon-neural"]
```

### Prometheus Metrics

```rust
use prometheus::{Counter, Histogram};

// Métricas automáticas exportadas
lazy_static! {
    static ref TASK_EXECUTIONS: Counter = register_counter!(
        "symbiotic_task_executions_total",
        "Total number of task executions"
    ).unwrap();
    
    static ref RETRY_DURATION: Histogram = register_histogram!(
        "symbiotic_retry_duration_seconds",
        "Duration of retry operations"
    ).unwrap();
}
```

## 🎨 Patterns Recomendados

### 1. Operação Crítica com Fallback

```rust
let result = retry_manager.retry_with_backoff(
    || circuit_breaker.call(
        || primary_operation(),
        context.clone()
    ),
    context.clone()
).await
.or_else(|_| fallback_operation())
.with_context(context)?;
```

### 2. Agregação de Serviços

```rust
let futures: Vec<_> = services.into_iter()
    .map(|service| {
        let context = ErrorContext::new("service_call", "aggregator")
            .with_metadata("service", &service.name);
        
        orchestrator.execute_with_circuit_breaker(
            &service.name,
            || service.call(),
            context
        )
    })
    .collect();

let results = futures::future::join_all(futures).await;
```

### 3. Pipeline de Processamento

```rust
let pipeline_result = data
    .validate()
    .with_context(ErrorContext::new("validation", "pipeline"))?
    .transform()
    .with_context(ErrorContext::new("transformation", "pipeline"))?
    .persist()
    .with_context(ErrorContext::new("persistence", "pipeline"))?;
```

## 📚 Documentação Adicional

- [Framework de Tratamento de Erros](../ERROR_HANDLING_FRAMEWORK.md)
- [Exemplos Práticos](examples/README.md)
- [API Reference](https://docs.rs/orchestrator_core)
- [Troubleshooting Guide](docs/troubleshooting.md)

## 🤝 Contribuição

1. Fork o projeto
2. Crie uma branch para sua feature (`git checkout -b feature/nova-funcionalidade`)
3. Commit suas mudanças (`git commit -am 'Adiciona nova funcionalidade'`)
4. Push para a branch (`git push origin feature/nova-funcionalidade`)
5. Abra um Pull Request

## 📄 Licença

Este projeto está licenciado sob a MIT License - veja o arquivo [LICENSE](LICENSE) para detalhes.

## 🌟 Status do Projeto

- ✅ Framework de Error Handling
- ✅ Sistema de Retry/Backoff
- ✅ Circuit Breaker
- ✅ Logging Contextual
- ✅ Consciência Simbiótica Básica
- 🔄 Simulação Quântica (Beta)
- 🔄 Modo Cluster (Beta)
- 📋 Dashboard de Monitoramento (Planejado)

---

**SYMBIOTIC_METHOD** - Construindo o futuro da orquestração inteligente


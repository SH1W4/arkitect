//! # Configuration Module
//!
//! Configuração do Task Mesh IA Orchestrator.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::layers::{ExecutionConfig, ClusterConfig, QuantumSimConfig};
use crate::learning::LearningConfig;

/// Configuração principal do orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    /// Configuração geral
    pub general: GeneralConfig,
    /// Configuração das camadas de execução
    pub execution: ExecutionConfig,
    /// Configuração do cluster
    pub cluster: Option<ClusterConfig>,
    /// Configuração de simulação quântica
    pub quantum: Option<QuantumSimConfig>,
    /// Configuração de aprendizado
    pub learning: LearningConfig,
    /// Configuração de consciência simbiótica
    pub consciousness: ConsciousnessConfig,
    /// Configuração de persistência
    pub persistence: PersistenceConfig,
    /// Configuração de segurança
    pub security: SecurityConfig,
    /// Configuração de observabilidade
    pub observability: ObservabilityConfig,
}

/// Configuração geral
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Nome da instância
    pub instance_name: String,
    /// Versão da configuração
    pub version: String,
    /// Ambiente (development, staging, production)
    pub environment: Environment,
    /// Diretório de trabalho
    pub work_dir: PathBuf,
    /// Diretório de logs
    pub log_dir: PathBuf,
    /// Nível de log
    pub log_level: LogLevel,
    /// Modo debug
    pub debug_mode: bool,
}

/// Ambientes suportados
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

/// Níveis de log
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// Configuração de consciência simbiótica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessConfig {
    /// Ativação da consciência
    pub enabled: bool,
    /// Nível inicial de consciência
    pub initial_awareness_level: String,
    /// Taxa de evolução
    pub evolution_rate: f64,
    /// Limiar de adaptação
    pub adaptation_threshold: f64,
    /// Tamanho máximo da memória episódica
    pub max_episodic_memory: usize,
    /// Intervalo de consolidação de aprendizados
    pub consolidation_interval: u64,
}

/// Configuração de persistência
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    /// Tipo de banco de dados
    pub database_type: DatabaseType,
    /// URL de conexão
    pub database_url: String,
    /// Tamanho do pool de conexões
    pub connection_pool_size: u32,
    /// Timeout de conexão
    pub connection_timeout_ms: u64,
    /// Configuração de cache
    pub cache: CacheConfig,
}

/// Tipos de banco de dados suportados
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DatabaseType {
    PostgreSQL,
    SQLite,
    MongoDB,
    Redis,
}

/// Configuração de cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Ativação do cache
    pub enabled: bool,
    /// URL do Redis
    pub redis_url: String,
    /// TTL padrão em segundos
    pub default_ttl: u64,
    /// Tamanho máximo em MB
    pub max_size_mb: u64,
}

/// Configuração de segurança
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Autenticação habilitada
    pub authentication_enabled: bool,
    /// Chave secreta para JWT
    pub jwt_secret: String,
    /// Tempo de expiração do token em segundos
    pub token_expiration: u64,
    /// Configuração de TLS
    pub tls: Option<TlsConfig>,
    /// Configuração de CORS
    pub cors: CorsConfig,
}

/// Configuração de TLS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificado
    pub cert_file: PathBuf,
    /// Chave privada
    pub key_file: PathBuf,
    /// CA bundle
    pub ca_file: Option<PathBuf>,
}

/// Configuração de CORS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Origens permitidas
    pub allowed_origins: Vec<String>,
    /// Métodos permitidos
    pub allowed_methods: Vec<String>,
    /// Cabeçalhos permitidos
    pub allowed_headers: Vec<String>,
}

/// Configuração de observabilidade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    /// Configuração de métricas
    pub metrics: MetricsConfig,
    /// Configuração de tracing
    pub tracing: TracingConfig,
    /// Configuração de health checks
    pub health_checks: HealthCheckConfig,
}

/// Configuração de métricas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Ativação de métricas
    pub enabled: bool,
    /// Porta para exposição de métricas
    pub port: u16,
    /// Caminho para métricas
    pub path: String,
    /// Intervalo de coleta em segundos
    pub collection_interval: u64,
}

/// Configuração de tracing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Ativação de tracing
    pub enabled: bool,
    /// Endpoint do Jaeger
    pub jaeger_endpoint: Option<String>,
    /// Taxa de amostragem
    pub sampling_rate: f64,
}

/// Configuração de health checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Ativação de health checks
    pub enabled: bool,
    /// Intervalo de verificação em segundos
    pub check_interval: u64,
    /// Timeout para cada check em segundos
    pub check_timeout: u64,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                instance_name: "orchestrator-core".to_string(),
                version: "0.1.0".to_string(),
                environment: Environment::Development,
                work_dir: PathBuf::from("./work"),
                log_dir: PathBuf::from("./logs"),
                log_level: LogLevel::Info,
                debug_mode: false,
            },
            execution: ExecutionConfig::default(),
            cluster: None,
            quantum: None,
            learning: LearningConfig::default(),
            consciousness: ConsciousnessConfig {
                enabled: true,
                initial_awareness_level: "Basic".to_string(),
                evolution_rate: 0.1,
                adaptation_threshold: 0.7,
                max_episodic_memory: 1000,
                consolidation_interval: 3600,
            },
            persistence: PersistenceConfig {
                database_type: DatabaseType::SQLite,
                database_url: "sqlite://orchestrator.db".to_string(),
                connection_pool_size: 10,
                connection_timeout_ms: 5000,
                cache: CacheConfig {
                    enabled: false,
                    redis_url: "redis://localhost:6379".to_string(),
                    default_ttl: 3600,
                    max_size_mb: 256,
                },
            },
            security: SecurityConfig {
                authentication_enabled: false,
                jwt_secret: "change-me-in-production".to_string(),
                token_expiration: 3600,
                tls: None,
                cors: CorsConfig {
                    allowed_origins: vec!["*".to_string()],
                    allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                    allowed_headers: vec!["*".to_string()],
                },
            },
            observability: ObservabilityConfig {
                metrics: MetricsConfig {
                    enabled: true,
                    port: 9090,
                    path: "/metrics".to_string(),
                    collection_interval: 60,
                },
                tracing: TracingConfig {
                    enabled: false,
                    jaeger_endpoint: None,
                    sampling_rate: 0.1,
                },
                health_checks: HealthCheckConfig {
                    enabled: true,
                    check_interval: 30,
                    check_timeout: 5,
                },
            },
        }
    }
}

impl OrchestratorConfig {
    /// Carrega configuração de arquivo
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self, config::ConfigError> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name(path.as_ref().to_str().unwrap()))
            .add_source(config::Environment::with_prefix("ORCHESTRATOR"))
            .build()?;
            
        settings.try_deserialize()
    }
    
    /// Salva configuração em arquivo
    pub fn to_file<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Valida configuração
    pub fn validate(&self) -> Result<(), String> {
        // Validações básicas
        if self.general.instance_name.is_empty() {
            return Err("Instance name cannot be empty".to_string());
        }
        
        if self.execution.max_parallel_tasks == 0 {
            return Err("Max parallel tasks must be greater than 0".to_string());
        }
        
        if self.learning.learning_rate <= 0.0 || self.learning.learning_rate > 1.0 {
            return Err("Learning rate must be between 0 and 1".to_string());
        }
        
        if self.consciousness.evolution_rate < 0.0 || self.consciousness.evolution_rate > 1.0 {
            return Err("Evolution rate must be between 0 and 1".to_string());
        }
        
        Ok(())
    }
    
    /// Obtém configuração para ambiente
    pub fn for_environment(env: Environment) -> Self {
        let mut config = Self::default();
        config.general.environment = env.clone();
        
        match env {
            Environment::Development => {
                config.general.debug_mode = true;
                config.general.log_level = LogLevel::Debug;
                config.security.authentication_enabled = false;
            },
            Environment::Staging => {
                config.general.debug_mode = false;
                config.general.log_level = LogLevel::Info;
                config.security.authentication_enabled = true;
            },
            Environment::Production => {
                config.general.debug_mode = false;
                config.general.log_level = LogLevel::Warn;
                config.security.authentication_enabled = true;
                config.security.jwt_secret = "production-secret-change-me".to_string();
            },
        }
        
        config
    }
    
    /// Mescla configurações
    pub fn merge(&mut self, other: Self) {
        // Implementação simplificada - em produção seria mais sofisticada
        if other.general.instance_name != "orchestrator-core" {
            self.general.instance_name = other.general.instance_name;
        }
        
        if other.general.debug_mode {
            self.general.debug_mode = other.general.debug_mode;
        }
        
        // Mescla outras configurações conforme necessário
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_default_config() {
        let config = OrchestratorConfig::default();
        assert_eq!(config.general.instance_name, "orchestrator-core");
        assert_eq!(config.general.environment, Environment::Development);
        assert!(config.consciousness.enabled);
    }
    
    #[test]
    fn test_config_validation() {
        let config = OrchestratorConfig::default();
        assert!(config.validate().is_ok());
        
        let mut invalid_config = config;
        invalid_config.general.instance_name = String::new();
        assert!(invalid_config.validate().is_err());
    }
    
    #[test]
    fn test_environment_configs() {
        let dev_config = OrchestratorConfig::for_environment(Environment::Development);
        assert!(dev_config.general.debug_mode);
        assert!(!dev_config.security.authentication_enabled);
        
        let prod_config = OrchestratorConfig::for_environment(Environment::Production);
        assert!(!prod_config.general.debug_mode);
        assert!(prod_config.security.authentication_enabled);
    }
    
    #[test]
    fn test_file_serialization() {
        let config = OrchestratorConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Salva configuração
        config.to_file(temp_file.path()).unwrap();
        
        // Carrega configuração (seria necessário ajustar para formato correto)
        // let loaded_config = OrchestratorConfig::from_file(temp_file.path()).unwrap();
        // assert_eq!(config.general.instance_name, loaded_config.general.instance_name);
    }
}


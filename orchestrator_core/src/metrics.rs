//! # System Metrics Module
//!
//! Sistema de métricas e observabilidade do Task Mesh IA Orchestrator.

use chrono::{DateTime, Utc};
use prometheus::{
    Counter, Gauge, Histogram, IntCounter, IntGauge, Registry,
    opts, register_counter, register_gauge, register_histogram,
    register_int_counter, register_int_gauge
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::{OrchestratorError, Result};
use crate::graph::{TaskId, TaskStatus};
use crate::layers::ExecutionLayer;

/// Métricas do sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub timestamp: DateTime<Utc>,
    pub orchestrator: OrchestratorMetrics,
    pub tasks: TaskMetrics,
    pub layers: LayerMetrics,
    pub consciousness: ConsciousnessMetrics,
    pub learning: LearningMetrics,
    pub system: SystemResourceMetrics,
}

/// Métricas do orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorMetrics {
    pub uptime_seconds: u64,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub active_connections: u64,
}

/// Métricas de tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub total_tasks: u64,
    pub pending_tasks: u64,
    pub running_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time_ms: f64,
    pub throughput_per_minute: f64,
    pub queue_depth: u64,
}

/// Métricas por camada de execução
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetrics {
    pub local: LayerStatistics,
    pub cluster: LayerStatistics,
    pub quantum_sim: LayerStatistics,
}

/// Estatísticas de uma camada
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerStatistics {
    pub tasks_executed: u64,
    pub success_rate: f64,
    pub average_execution_time_ms: f64,
    pub resource_utilization: f64,
    pub availability: f64,
    pub error_count: u64,
}

/// Métricas de consciência simbiótica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessMetrics {
    pub awareness_level: String,
    pub synchronization_level: f64,
    pub coherence_index: f64,
    pub patterns_recognized: u64,
    pub insights_generated: u64,
    pub decisions_made: u64,
    pub evolution_events: u64,
}

/// Métricas de aprendizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub models_trained: u64,
    pub training_iterations: u64,
    pub average_accuracy: f64,
    pub improvement_rate: f64,
    pub predictions_made: u64,
    pub prediction_accuracy: f64,
}

/// Métricas de recursos do sistema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub memory_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub disk_usage_percent: f64,
    pub network_rx_mb: f64,
    pub network_tx_mb: f64,
    pub open_file_descriptors: u64,
}

/// Coletor de métricas
#[derive(Debug)]
pub struct MetricsCollector {
    registry: Registry,
    metrics: Arc<RwLock<SystemMetrics>>,
    start_time: DateTime<Utc>,
    
    // Contadores Prometheus
    task_counter: IntCounter,
    task_success_counter: IntCounter,
    task_failure_counter: IntCounter,
    
    // Gauges Prometheus
    active_tasks_gauge: IntGauge,
    consciousness_level_gauge: Gauge,
    resource_usage_gauge: Gauge,
    
    // Histogramas Prometheus
    task_execution_histogram: Histogram,
    response_time_histogram: Histogram,
}

impl MetricsCollector {
    /// Cria novo coletor de métricas
    pub fn new() -> Result<Self> {
        let registry = Registry::new();
        let start_time = Utc::now();
        
        // Inicializa métricas Prometheus
        let task_counter = register_int_counter!(
            opts!("orchestrator_tasks_total", "Total number of tasks processed")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let task_success_counter = register_int_counter!(
            opts!("orchestrator_tasks_success_total", "Total number of successful tasks")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let task_failure_counter = register_int_counter!(
            opts!("orchestrator_tasks_failure_total", "Total number of failed tasks")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let active_tasks_gauge = register_int_gauge!(
            opts!("orchestrator_active_tasks", "Number of currently active tasks")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let consciousness_level_gauge = register_gauge!(
            opts!("orchestrator_consciousness_level", "Current consciousness level")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let resource_usage_gauge = register_gauge!(
            opts!("orchestrator_resource_usage", "Resource usage percentage")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let task_execution_histogram = register_histogram!(
            opts!("orchestrator_task_execution_duration_seconds", "Task execution duration")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let response_time_histogram = register_histogram!(
            opts!("orchestrator_response_time_seconds", "API response time")
        ).map_err(|e| OrchestratorError::InternalError(e.to_string()))?;
        
        let initial_metrics = SystemMetrics {
            timestamp: start_time,
            orchestrator: OrchestratorMetrics {
                uptime_seconds: 0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                active_connections: 0,
            },
            tasks: TaskMetrics {
                total_tasks: 0,
                pending_tasks: 0,
                running_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_execution_time_ms: 0.0,
                throughput_per_minute: 0.0,
                queue_depth: 0,
            },
            layers: LayerMetrics {
                local: LayerStatistics {
                    tasks_executed: 0,
                    success_rate: 0.0,
                    average_execution_time_ms: 0.0,
                    resource_utilization: 0.0,
                    availability: 1.0,
                    error_count: 0,
                },
                cluster: LayerStatistics {
                    tasks_executed: 0,
                    success_rate: 0.0,
                    average_execution_time_ms: 0.0,
                    resource_utilization: 0.0,
                    availability: 0.0,
                    error_count: 0,
                },
                quantum_sim: LayerStatistics {
                    tasks_executed: 0,
                    success_rate: 0.0,
                    average_execution_time_ms: 0.0,
                    resource_utilization: 0.0,
                    availability: 0.0,
                    error_count: 0,
                },
            },
            consciousness: ConsciousnessMetrics {
                awareness_level: "Basic".to_string(),
                synchronization_level: 0.5,
                coherence_index: 0.5,
                patterns_recognized: 0,
                insights_generated: 0,
                decisions_made: 0,
                evolution_events: 0,
            },
            learning: LearningMetrics {
                models_trained: 0,
                training_iterations: 0,
                average_accuracy: 0.0,
                improvement_rate: 0.0,
                predictions_made: 0,
                prediction_accuracy: 0.0,
            },
            system: SystemResourceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
                memory_usage_percent: 0.0,
                disk_usage_mb: 0.0,
                disk_usage_percent: 0.0,
                network_rx_mb: 0.0,
                network_tx_mb: 0.0,
                open_file_descriptors: 0,
            },
        };
        
        Ok(Self {
            registry,
            metrics: Arc::new(RwLock::new(initial_metrics)),
            start_time,
            task_counter,
            task_success_counter,
            task_failure_counter,
            active_tasks_gauge,
            consciousness_level_gauge,
            resource_usage_gauge,
            task_execution_histogram,
            response_time_histogram,
        })
    }
    
    /// Incrementa contador de tarefas
    pub async fn increment_task_counter(&self) {
        self.task_counter.inc();
        
        let mut metrics = self.metrics.write().await;
        metrics.tasks.total_tasks += 1;
        metrics.timestamp = Utc::now();
    }
    
    /// Registra sucesso de tarefa
    pub async fn record_task_success(&self, execution_time_ms: f64) {
        self.task_success_counter.inc();
        self.task_execution_histogram.observe(execution_time_ms / 1000.0);
        
        let mut metrics = self.metrics.write().await;
        metrics.tasks.completed_tasks += 1;
        
        // Atualiza média de tempo de execução
        let total_completed = metrics.tasks.completed_tasks;
        let current_avg = metrics.tasks.average_execution_time_ms;
        metrics.tasks.average_execution_time_ms = 
            (current_avg * (total_completed - 1) as f64 + execution_time_ms) / total_completed as f64;
            
        metrics.timestamp = Utc::now();
    }
    
    /// Registra falha de tarefa
    pub async fn record_task_failure(&self) {
        self.task_failure_counter.inc();
        
        let mut metrics = self.metrics.write().await;
        metrics.tasks.failed_tasks += 1;
        metrics.timestamp = Utc::now();
    }
    
    /// Atualiza gauge de tarefas ativas
    pub async fn set_active_tasks(&self, count: i64) {
        self.active_tasks_gauge.set(count);
        
        let mut metrics = self.metrics.write().await;
        metrics.tasks.running_tasks = count as u64;
        metrics.timestamp = Utc::now();
    }
    
    /// Atualiza métricas de consciência
    pub async fn update_consciousness_metrics(&self, consciousness_metrics: ConsciousnessMetrics) {
        // Mapeia nível de consciência para valor numérico
        let level_value = match consciousness_metrics.awareness_level.as_str() {
            "Basic" => 1.0,
            "Cognitive" => 2.0,
            "Metacognitive" => 3.0,
            "Quantum" => 4.0,
            "Transcendent" => 5.0,
            _ => 1.0,
        };
        
        self.consciousness_level_gauge.set(level_value);
        
        let mut metrics = self.metrics.write().await;
        metrics.consciousness = consciousness_metrics;
        metrics.timestamp = Utc::now();
    }
    
    /// Atualiza métricas de recursos do sistema
    pub async fn update_system_resources(&self, system_metrics: SystemResourceMetrics) {
        self.resource_usage_gauge.set(system_metrics.cpu_usage_percent);
        
        let mut metrics = self.metrics.write().await;
        metrics.system = system_metrics;
        metrics.timestamp = Utc::now();
    }
    
    /// Atualiza métricas de camada
    pub async fn update_layer_metrics(&self, layer: ExecutionLayer, stats: LayerStatistics) {
        let mut metrics = self.metrics.write().await;
        
        match layer {
            ExecutionLayer::Local => metrics.layers.local = stats,
            ExecutionLayer::Cluster => metrics.layers.cluster = stats,
            ExecutionLayer::QuantumSim => metrics.layers.quantum_sim = stats,
        }
        
        metrics.timestamp = Utc::now();
    }
    
    /// Registra tempo de resposta da API
    pub async fn record_api_response_time(&self, duration_ms: f64) {
        self.response_time_histogram.observe(duration_ms / 1000.0);
        
        let mut metrics = self.metrics.write().await;
        let total_requests = metrics.orchestrator.total_requests + 1;
        let current_avg = metrics.orchestrator.average_response_time_ms;
        
        metrics.orchestrator.total_requests = total_requests;
        metrics.orchestrator.average_response_time_ms = 
            (current_avg * (total_requests - 1) as f64 + duration_ms) / total_requests as f64;
            
        metrics.timestamp = Utc::now();
    }
    
    /// Registra requisição bem-sucedida
    pub async fn record_successful_request(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.orchestrator.successful_requests += 1;
        metrics.timestamp = Utc::now();
    }
    
    /// Registra requisição falhada
    pub async fn record_failed_request(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.orchestrator.failed_requests += 1;
        metrics.timestamp = Utc::now();
    }
    
    /// Obtém métricas atuais
    pub async fn get_metrics(&self) -> SystemMetrics {
        let mut metrics = self.metrics.read().await.clone();
        
        // Atualiza uptime
        metrics.orchestrator.uptime_seconds = 
            (Utc::now() - self.start_time).num_seconds() as u64;
            
        // Calcula throughput por minuto
        if metrics.orchestrator.uptime_seconds > 0 {
            metrics.tasks.throughput_per_minute = 
                (metrics.tasks.completed_tasks as f64 * 60.0) / metrics.orchestrator.uptime_seconds as f64;
        }
        
        metrics
    }
    
    /// Coleta métricas do sistema operacional
    pub async fn collect_system_metrics(&self) -> SystemResourceMetrics {
        // Implementação simplificada - em produção usaria bibliotecas como sysinfo
        SystemResourceMetrics {
            cpu_usage_percent: self.get_cpu_usage().await,
            memory_usage_mb: self.get_memory_usage_mb().await,
            memory_usage_percent: self.get_memory_usage_percent().await,
            disk_usage_mb: self.get_disk_usage_mb().await,
            disk_usage_percent: self.get_disk_usage_percent().await,
            network_rx_mb: self.get_network_rx_mb().await,
            network_tx_mb: self.get_network_tx_mb().await,
            open_file_descriptors: self.get_open_file_descriptors().await,
        }
    }
    
    /// Exporta métricas no formato Prometheus
    pub fn export_prometheus_metrics(&self) -> String {
        prometheus::gather().into_iter()
            .map(|mf| prometheus::TextEncoder::new().encode_to_string(&[mf]).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("")
    }
    
    /// Reset de métricas (para testes)
    pub async fn reset_metrics(&self) {
        let mut metrics = self.metrics.write().await;
        *metrics = SystemMetrics {
            timestamp: Utc::now(),
            orchestrator: OrchestratorMetrics {
                uptime_seconds: 0,
                total_requests: 0,
                successful_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
                active_connections: 0,
            },
            tasks: TaskMetrics {
                total_tasks: 0,
                pending_tasks: 0,
                running_tasks: 0,
                completed_tasks: 0,
                failed_tasks: 0,
                average_execution_time_ms: 0.0,
                throughput_per_minute: 0.0,
                queue_depth: 0,
            },
            layers: LayerMetrics {
                local: LayerStatistics {
                    tasks_executed: 0,
                    success_rate: 0.0,
                    average_execution_time_ms: 0.0,
                    resource_utilization: 0.0,
                    availability: 1.0,
                    error_count: 0,
                },
                cluster: LayerStatistics {
                    tasks_executed: 0,
                    success_rate: 0.0,
                    average_execution_time_ms: 0.0,
                    resource_utilization: 0.0,
                    availability: 0.0,
                    error_count: 0,
                },
                quantum_sim: LayerStatistics {
                    tasks_executed: 0,
                    success_rate: 0.0,
                    average_execution_time_ms: 0.0,
                    resource_utilization: 0.0,
                    availability: 0.0,
                    error_count: 0,
                },
            },
            consciousness: ConsciousnessMetrics {
                awareness_level: "Basic".to_string(),
                synchronization_level: 0.5,
                coherence_index: 0.5,
                patterns_recognized: 0,
                insights_generated: 0,
                decisions_made: 0,
                evolution_events: 0,
            },
            learning: LearningMetrics {
                models_trained: 0,
                training_iterations: 0,
                average_accuracy: 0.0,
                improvement_rate: 0.0,
                predictions_made: 0,
                prediction_accuracy: 0.0,
            },
            system: SystemResourceMetrics {
                cpu_usage_percent: 0.0,
                memory_usage_mb: 0.0,
                memory_usage_percent: 0.0,
                disk_usage_mb: 0.0,
                disk_usage_percent: 0.0,
                network_rx_mb: 0.0,
                network_tx_mb: 0.0,
                open_file_descriptors: 0,
            },
        };
    }
    
    // Métodos auxiliares para coleta de métricas do sistema
    async fn get_cpu_usage(&self) -> f64 {
        // Simulação - em produção usaria biblioteca apropriada
        25.0 + (rand::random::<f64>() * 50.0)
    }
    
    async fn get_memory_usage_mb(&self) -> f64 {
        512.0 + (rand::random::<f64>() * 1024.0)
    }
    
    async fn get_memory_usage_percent(&self) -> f64 {
        30.0 + (rand::random::<f64>() * 40.0)
    }
    
    async fn get_disk_usage_mb(&self) -> f64 {
        10240.0 + (rand::random::<f64>() * 5120.0)
    }
    
    async fn get_disk_usage_percent(&self) -> f64 {
        40.0 + (rand::random::<f64>() * 30.0)
    }
    
    async fn get_network_rx_mb(&self) -> f64 {
        rand::random::<f64>() * 100.0
    }
    
    async fn get_network_tx_mb(&self) -> f64 {
        rand::random::<f64>() * 50.0
    }
    
    async fn get_open_file_descriptors(&self) -> u64 {
        100 + (rand::random::<u64>() % 500)
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new().expect("Failed to create metrics collector")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let collector = MetricsCollector::new();
        assert!(collector.is_ok());
    }
    
    #[tokio::test]
    async fn test_task_metrics() {
        let collector = MetricsCollector::new().unwrap();
        
        collector.increment_task_counter().await;
        collector.record_task_success(1500.0).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.tasks.total_tasks, 1);
        assert_eq!(metrics.tasks.completed_tasks, 1);
        assert_eq!(metrics.tasks.average_execution_time_ms, 1500.0);
    }
    
    #[tokio::test]
    async fn test_consciousness_metrics() {
        let collector = MetricsCollector::new().unwrap();
        
        let consciousness_metrics = ConsciousnessMetrics {
            awareness_level: "Cognitive".to_string(),
            synchronization_level: 0.8,
            coherence_index: 0.9,
            patterns_recognized: 5,
            insights_generated: 3,
            decisions_made: 10,
            evolution_events: 2,
        };
        
        collector.update_consciousness_metrics(consciousness_metrics.clone()).await;
        
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.consciousness.awareness_level, "Cognitive");
        assert_eq!(metrics.consciousness.patterns_recognized, 5);
    }
    
    #[tokio::test]
    async fn test_prometheus_export() {
        let collector = MetricsCollector::new().unwrap();
        
        collector.increment_task_counter().await;
        collector.record_task_success(1000.0).await;
        
        let prometheus_output = collector.export_prometheus_metrics();
        assert!(!prometheus_output.is_empty());
    }
}


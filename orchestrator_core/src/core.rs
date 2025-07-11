//! # Orchestrator Core Module
//!
//! Módulo principal do Task Mesh IA Orchestrator Core.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, Mutex};
use chrono::{DateTime, Utc};
use tracing::{info, warn, error, debug};

use crate::config::OrchestratorConfig;
use crate::errors::{OrchestratorError, Result};
use crate::graph::{TaskMesh, TaskNode, TaskId, TaskStatus};
use crate::layers::{LayerManager, ExecutionLayer, TaskExecutionResult, ExecutionLayerTrait};
use crate::symbiotic::{SymbioticConsciousness, SystemEvent, EventSeverity};
use crate::learning::ContinuousLearning;
use crate::metrics::MetricsCollector;

/// Resultado de execução de tarefa (re-export)
pub use crate::layers::TaskExecutionResult;

/// Estado do orchestrator
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrchestratorStatus {
    /// Inicializando
    Initializing,
    /// Operacional
    Running,
    /// Pausado
    Paused,
    /// Finalizando
    Shutting,
    /// Parado
    Stopped,
    /// Estado de erro
    Error,
}

/// Core principal do orchestrator
#[derive(Debug)]
pub struct OrchestratorCore {
    /// Configuração
    config: OrchestratorConfig,
    /// Status atual
    status: Arc<RwLock<OrchestratorStatus>>,
    /// Grafo de tarefas
    task_mesh: Arc<RwLock<TaskMesh>>,
    /// Gerenciador de camadas de execução
    layer_manager: Arc<LayerManager>,
    /// Sistema de consciência simbiótica
    consciousness: Arc<SymbioticConsciousness>,
    /// Sistema de aprendizado contínuo
    learning: Arc<ContinuousLearning>,
    /// Coletor de métricas
    metrics: Arc<MetricsCollector>,
    /// Fila de execução
    execution_queue: Arc<Mutex<Vec<TaskId>>>,
    /// Tarefas em execução
    running_tasks: Arc<RwLock<HashMap<TaskId, tokio::task::JoinHandle<()>>>>,
    /// Timestamp de inicialização
    started_at: DateTime<Utc>,
}

impl OrchestratorCore {
    /// Cria nova instância do orchestrator
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        info!("Initializing Orchestrator Core with config: {:?}", config.general.instance_name);
        
        // Valida configuração
        config.validate().map_err(|e| OrchestratorError::ConfigurationError(e))?;
        
        // Inicializa componentes
        let task_mesh = Arc::new(RwLock::new(TaskMesh::new()));
        let layer_manager = Arc::new(LayerManager::new());
        let consciousness = Arc::new(SymbioticConsciousness::new());
        let learning = Arc::new(ContinuousLearning::new(config.learning.clone()));
        let metrics = Arc::new(MetricsCollector::new()?);
        
        let orchestrator = Self {
            config,
            status: Arc::new(RwLock::new(OrchestratorStatus::Initializing)),
            task_mesh,
            layer_manager,
            consciousness,
            learning,
            metrics,
            execution_queue: Arc::new(Mutex::new(Vec::new())),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            started_at: Utc::now(),
        };
        
        info!("Orchestrator Core initialized successfully");
        Ok(orchestrator)
    }
    
    /// Inicia o orchestrator
    pub async fn start(&self) -> Result<()> {
        info!("Starting Orchestrator Core");
        
        {
            let mut status = self.status.write().await;
            *status = OrchestratorStatus::Running;
        }
        
        // Inicializa loops de execução
        self.start_execution_loop().await;
        self.start_metrics_collection_loop().await;
        self.start_consciousness_loop().await;
        
        // Emite evento de inicialização
        let start_event = SystemEvent {
            event_type: "orchestrator_started".to_string(),
            data: HashMap::new(),
            timestamp: Utc::now(),
            source: "orchestrator_core".to_string(),
            severity: EventSeverity::Medium,
        };
        
        let _ = self.consciousness.process_event(start_event).await;
        
        info!("Orchestrator Core started successfully");
        Ok(())
    }
    
    /// Para o orchestrator
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Orchestrator Core");
        
        {
            let mut status = self.status.write().await;
            *status = OrchestratorStatus::Shutting;
        }
        
        // Cancela tarefas em execução
        let running_tasks = self.running_tasks.read().await;
        for handle in running_tasks.values() {
            handle.abort();
        }
        
        {
            let mut status = self.status.write().await;
            *status = OrchestratorStatus::Stopped;
        }
        
        info!("Orchestrator Core stopped");
        Ok(())
    }
    
    /// Adiciona tarefa ao grafo
    pub async fn add_task(&self, mut task: TaskNode) -> Result<TaskId> {
        let task_id = task.id;
        
        debug!("Adding task: {} ({})", task.name, task_id);
        
        // Adiciona ao grafo
        {
            let mut mesh = self.task_mesh.write().await;
            mesh.add_task(task.clone())?;
        }
        
        // Enfileira para execução se não tiver dependências
        let ready = self.is_task_ready(&task_id).await?;
        if ready {
            let mut queue = self.execution_queue.lock().await;
            queue.push(task_id);
        }
        
        // Atualiza métricas
        self.metrics.increment_task_counter().await;
        
        // Emite evento para consciência
        let task_event = SystemEvent {
            event_type: "task_added".to_string(),
            data: HashMap::from([
                ("task_id".to_string(), serde_json::Value::String(task_id.to_string())),
                ("task_name".to_string(), serde_json::Value::String(task.name.clone())),
            ]),
            timestamp: Utc::now(),
            source: "orchestrator_core".to_string(),
            severity: EventSeverity::Low,
        };
        
        let _ = self.consciousness.process_event(task_event).await;
        
        info!("Task added: {} ({})", task.name, task_id);
        Ok(task_id)
    }
    
    /// Remove tarefa do grafo
    pub async fn remove_task(&self, task_id: TaskId) -> Result<()> {
        debug!("Removing task: {}", task_id);
        
        // Remove da fila de execução
        {
            let mut queue = self.execution_queue.lock().await;
            queue.retain(|&id| id != task_id);
        }
        
        // Cancela se estiver em execução
        {
            let mut running = self.running_tasks.write().await;
            if let Some(handle) = running.remove(&task_id) {
                handle.abort();
            }
        }
        
        info!("Task removed: {}", task_id);
        Ok(())
    }
    
    /// Executa uma tarefa
    pub async fn execute_task(&self, task_id: TaskId) -> Result<TaskExecutionResult> {
        debug!("Executing task: {}", task_id);
        
        let task = {
            let mesh = self.task_mesh.read().await;
            mesh.get_task(&task_id)
                .ok_or_else(|| OrchestratorError::TaskNotFound(task_id))?
                .clone()
        };
        
        // Verifica se pode executar
        if !self.is_task_ready(&task_id).await? {
            return Err(OrchestratorError::InvalidState(
                "Task dependencies not satisfied".to_string()
            ));
        }
        
        // Atualiza status da tarefa
        {
            let mut mesh = self.task_mesh.write().await;
            if let Some(task_mut) = mesh.get_task_mut(&task_id) {
                task_mut.update_status(TaskStatus::Running);
            }
        }
        
        // Seleciona camada de execução
        let layer = self.select_execution_layer(&task).await?;
        
        // Obtém executor da camada
        let executor = self.layer_manager.get_layer(&layer)
            .ok_or_else(|| OrchestratorError::LayerNotAvailable(layer.clone()))?;
        
        // Executa tarefa
        let start_time = Utc::now();
        let result = executor.execute_task(&task, &self.config.execution).await;
        
        let execution_result = match result {
            Ok(mut exec_result) => {
                // Atualiza status da tarefa
                {
                    let mut mesh = self.task_mesh.write().await;
                    if let Some(task_mut) = mesh.get_task_mut(&task_id) {
                        task_mut.update_status(TaskStatus::Completed);
                        task_mut.metrics.start_time = Some(start_time);
                        task_mut.metrics.end_time = exec_result.end_time;
                    }
                }
                
                // Registra sucesso nas métricas
                let duration = (Utc::now() - start_time).num_milliseconds() as f64;
                self.metrics.record_task_success(duration).await;
                
                // Adiciona dados ao aprendizado
                let _ = self.learning.add_execution_data(&task, &exec_result).await;
                
                exec_result
            },
            Err(e) => {
                // Atualiza status da tarefa como falha
                {
                    let mut mesh = self.task_mesh.write().await;
                    if let Some(task_mut) = mesh.get_task_mut(&task_id) {
                        task_mut.update_status(TaskStatus::Failed);
                    }
                }
                
                // Registra falha nas métricas
                self.metrics.record_task_failure().await;
                
                warn!("Task execution failed: {} - {}", task_id, e);
                return Err(e);
            }
        };
        
        // Enfileira tarefas dependentes
        self.enqueue_dependent_tasks(&task_id).await?;
        
        // Emite evento de conclusão
        let completion_event = SystemEvent {
            event_type: "task_completed".to_string(),
            data: HashMap::from([
                ("task_id".to_string(), serde_json::Value::String(task_id.to_string())),
                ("execution_time_ms".to_string(), serde_json::Value::Number(
                    serde_json::Number::from(execution_result.resource_usage.execution_time_ms)
                )),
            ]),
            timestamp: Utc::now(),
            source: "orchestrator_core".to_string(),
            severity: EventSeverity::Low,
        };
        
        let _ = self.consciousness.process_event(completion_event).await;
        
        info!("Task completed: {}", task_id);
        Ok(execution_result)
    }
    
    /// Verifica se uma tarefa está pronta para execução
    async fn is_task_ready(&self, task_id: &TaskId) -> Result<bool> {
        let mesh = self.task_mesh.read().await;
        mesh.can_execute_task(task_id)
    }
    
    /// Seleciona camada de execução para uma tarefa
    async fn select_execution_layer(&self, task: &TaskNode) -> Result<ExecutionLayer> {
        // Tenta usar aprendizado para recomendar camada
        if let Ok(recommended_layer) = self.learning.recommend_execution_layer(task).await {
            debug!("Learning recommended layer: {:?} for task: {}", recommended_layer, task.id);
            return Ok(recommended_layer);
        }
        
        // Fallback para seleção baseada em heurísticas
        match task.priority {
            crate::graph::TaskPriority::Critical => Ok(ExecutionLayer::Local),
            crate::graph::TaskPriority::High => {
                if task.task_type == crate::graph::TaskType::ExtraLarge {
                    Ok(ExecutionLayer::QuantumSim)
                } else {
                    Ok(ExecutionLayer::Cluster)
                }
            },
            _ => Ok(ExecutionLayer::Local),
        }
    }
    
    /// Enfileira tarefas dependentes que ficaram prontas
    async fn enqueue_dependent_tasks(&self, completed_task_id: &TaskId) -> Result<()> {
        let mesh = self.task_mesh.read().await;
        let dependents = mesh.get_dependents(completed_task_id)?;
        
        let mut queue = self.execution_queue.lock().await;
        
        for dependent in dependents {
            if mesh.can_execute_task(&dependent.id)? {
                queue.push(dependent.id);
                debug!("Enqueued dependent task: {}", dependent.id);
            }
        }
        
        Ok(())
    }
    
    /// Inicia loop de execução
    async fn start_execution_loop(&self) {
        let queue = Arc::clone(&self.execution_queue);
        let running_tasks = Arc::clone(&self.running_tasks);
        let orchestrator = self.clone_for_tasks();
        
        tokio::spawn(async move {
            loop {
                // Processa fila de execução
                let task_id = {
                    let mut q = queue.lock().await;
                    q.pop()
                };
                
                if let Some(task_id) = task_id {
                    let orch_clone = orchestrator.clone();
                    let handle = tokio::spawn(async move {
                        if let Err(e) = orch_clone.execute_task(task_id).await {
                            error!("Task execution error: {}", e);
                        }
                    });
                    
                    running_tasks.write().await.insert(task_id, handle);
                }
                
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
    }
    
    /// Inicia loop de coleta de métricas
    async fn start_metrics_collection_loop(&self) {
        let metrics = Arc::clone(&self.metrics);
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(config.observability.metrics.collection_interval)
            );
            
            loop {
                interval.tick().await;
                
                // Coleta métricas do sistema
                let system_metrics = metrics.collect_system_metrics().await;
                metrics.update_system_resources(system_metrics).await;
            }
        });
    }
    
    /// Inicia loop de consciência
    async fn start_consciousness_loop(&self) {
        let consciousness = Arc::clone(&self.consciousness);
        let metrics = Arc::clone(&self.metrics);
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(60) // Processa consciência a cada minuto
            );
            
            loop {
                interval.tick().await;
                
                // Força evolução periódica da consciência
                if let Err(e) = consciousness.evolve().await {
                    error!("Consciousness evolution error: {}", e);
                }
                
                // Atualiza métricas de consciência
                let state = consciousness.get_state().await;
                let consciousness_metrics = crate::metrics::ConsciousnessMetrics {
                    awareness_level: format!("{:?}", state.awareness_level),
                    synchronization_level: state.collective_state.synchronization_level,
                    coherence_index: state.collective_state.coherence_index,
                    patterns_recognized: state.recognized_patterns.len() as u64,
                    insights_generated: state.collective_state.shared_insights.len() as u64,
                    decisions_made: 0, // TODO: Rastrear decisões
                    evolution_events: 0, // TODO: Rastrear eventos de evolução
                };
                
                metrics.update_consciousness_metrics(consciousness_metrics).await;
            }
        });
    }
    
    /// Clone simplificado para uso em tasks
    fn clone_for_tasks(&self) -> OrchestratorCoreRef {
        OrchestratorCoreRef {
            task_mesh: Arc::clone(&self.task_mesh),
            layer_manager: Arc::clone(&self.layer_manager),
            consciousness: Arc::clone(&self.consciousness),
            learning: Arc::clone(&self.learning),
            metrics: Arc::clone(&self.metrics),
            execution_queue: Arc::clone(&self.execution_queue),
            running_tasks: Arc::clone(&self.running_tasks),
            config: self.config.clone(),
        }
    }
    
    /// Obtém status atual
    pub async fn get_status(&self) -> OrchestratorStatus {
        self.status.read().await.clone()
    }
    
    /// Obtém métricas atuais
    pub async fn get_metrics(&self) -> crate::metrics::SystemMetrics {
        self.metrics.get_metrics().await
    }
    
    /// Obtém estado da consciência
    pub async fn get_consciousness_state(&self) -> crate::symbiotic::ConsciousnessState {
        self.consciousness.get_state().await
    }
    
    /// Obtém estatísticas do grafo de tarefas
    pub async fn get_task_statistics(&self) -> crate::graph::TaskMeshStatistics {
        self.task_mesh.read().await.statistics()
    }
    
    /// Lista tarefas prontas para execução
    pub async fn get_ready_tasks(&self) -> Result<Vec<&crate::graph::TaskNode>> {
        self.task_mesh.read().await.get_ready_tasks()
    }
}

/// Referência simplificada para uso em tasks
#[derive(Clone)]
struct OrchestratorCoreRef {
    task_mesh: Arc<RwLock<TaskMesh>>,
    layer_manager: Arc<LayerManager>,
    consciousness: Arc<SymbioticConsciousness>,
    learning: Arc<ContinuousLearning>,
    metrics: Arc<MetricsCollector>,
    execution_queue: Arc<Mutex<Vec<TaskId>>>,
    running_tasks: Arc<RwLock<HashMap<TaskId, tokio::task::JoinHandle<()>>>>,
    config: OrchestratorConfig,
}

impl OrchestratorCoreRef {
    async fn execute_task(&self, task_id: TaskId) -> Result<TaskExecutionResult> {
        // Implementação simplificada para evitar recursão
        debug!("Executing task in ref: {}", task_id);
        
        let task = {
            let mesh = self.task_mesh.read().await;
            mesh.get_task(&task_id)
                .ok_or_else(|| OrchestratorError::TaskNotFound(task_id))?
                .clone()
        };
        
        // Seleciona camada local por simplicidade
        let layer = ExecutionLayer::Local;
        let executor = self.layer_manager.get_layer(&layer)
            .ok_or_else(|| OrchestratorError::LayerNotAvailable(layer))?;
        
        executor.execute_task(&task, &self.config.execution).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::OrchestratorConfig;
    use crate::graph::TaskNode;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let config = OrchestratorConfig::default();
        let orchestrator = OrchestratorCore::new(config).await;
        
        assert!(orchestrator.is_ok());
        let orch = orchestrator.unwrap();
        assert_eq!(orch.get_status().await, OrchestratorStatus::Initializing);
    }
    
    #[tokio::test]
    async fn test_add_task() {
        let config = OrchestratorConfig::default();
        let orchestrator = OrchestratorCore::new(config).await.unwrap();
        
        let task = TaskNode::new("Test Task".to_string(), Some("Test Description".to_string()));
        let task_id = task.id;
        
        let result = orchestrator.add_task(task).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), task_id);
    }
    
    #[tokio::test]
    async fn test_orchestrator_lifecycle() {
        let config = OrchestratorConfig::default();
        let orchestrator = OrchestratorCore::new(config).await.unwrap();
        
        // Inicia
        orchestrator.start().await.unwrap();
        assert_eq!(orchestrator.get_status().await, OrchestratorStatus::Running);
        
        // Para
        orchestrator.stop().await.unwrap();
        assert_eq!(orchestrator.get_status().await, OrchestratorStatus::Stopped);
    }
}


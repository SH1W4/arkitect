//! Scheduler inteligente com algoritmos topológicos e heurísticas avançadas

use std::collections::{HashMap, HashSet, VecDeque, BinaryHeap};
use std::cmp::{Ordering, Reverse};
use std::time::{Duration, SystemTime};
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use petgraph::prelude::*;
use petgraph::algo::toposort;

use crate::types::*;
use crate::TaskMeshResult;

/// Heurísticas de agendamento
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SchedulingHeuristic {
    /// Primeiro a entrar, primeiro a sair
    FIFO,
    /// Último a entrar, primeiro a sair
    LIFO,
    /// Prioridade mais alta primeiro
    Priority,
    /// Menor tempo estimado primeiro
    ShortestJobFirst,
    /// Deadline mais próximo primeiro
    EarliestDeadlineFirst,
    /// Menor razão crítica primeiro (deadline/tempo_estimado)
    CriticalRatio,
    /// Algoritmo genético adaptativo
    Genetic {
        population_size: usize,
        mutation_rate: f64,
        crossover_rate: f64,
    },
    /// Heurística híbrida personalizada
    Hybrid {
        primary: Box<SchedulingHeuristic>,
        secondary: Box<SchedulingHeuristic>,
        threshold: f64,
    },
}

impl Default for SchedulingHeuristic {
    fn default() -> Self {
        SchedulingHeuristic::Hybrid {
            primary: Box::new(SchedulingHeuristic::CriticalRatio),
            secondary: Box::new(SchedulingHeuristic::Priority),
            threshold: 0.7,
        }
    }
}

/// Estimativa de custo de execução
#[derive(Debug, Clone)]
pub struct ExecutionEstimate {
    /// Tempo estimado de execução
    pub estimated_duration: Duration,
    /// Recursos necessários
    pub resource_requirements: ResourceAllocation,
    /// Confiança na estimativa (0.0-1.0)
    pub confidence: f64,
    /// Histórico de execuções similares
    pub historical_data: Vec<ExecutionMetrics>,
}

/// Plano de execução
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    /// Ordem das tarefas
    pub execution_order: Vec<TaskId>,
    /// Estimativa total de tempo
    pub total_estimated_time: Duration,
    /// Agrupamentos paralelos
    pub parallel_groups: Vec<Vec<TaskId>>,
    /// Pontos de sincronização
    pub sync_points: Vec<usize>,
    /// Métricas do plano
    pub plan_metrics: PlanMetrics,
}

/// Métricas do plano de execução
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanMetrics {
    /// Paralelismo médio
    pub avg_parallelism: f64,
    /// Eficiência de recursos
    pub resource_efficiency: f64,
    /// Fator de carga
    pub load_factor: f64,
    /// Caminho crítico
    pub critical_path_length: Duration,
}

/// Item da fila de agendamento
#[derive(Debug, Clone)]
struct ScheduleItem {
    task_id: TaskId,
    priority_score: f64,
    estimated_duration: Duration,
    deadline: Option<SystemTime>,
    resource_requirements: ResourceAllocation,
}

impl PartialEq for ScheduleItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score.partial_cmp(&other.priority_score) == Some(Ordering::Equal)
    }
}

impl Eq for ScheduleItem {}

impl PartialOrd for ScheduleItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority_score.partial_cmp(&other.priority_score)
    }
}

impl Ord for ScheduleItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority_score.partial_cmp(&other.priority_score)
            .unwrap_or(Ordering::Equal)
    }
}

/// Scheduler principal
pub struct Scheduler {
    /// Heurística ativa
    heuristic: SchedulingHeuristic,
    
    /// Fila de agendamento
    schedule_queue: Arc<RwLock<BinaryHeap<ScheduleItem>>>,
    
    /// Grafo de dependências
    dependency_graph: Arc<RwLock<DiGraph<TaskId, ()>>>,
    
    /// Mapeamento de nós
    node_map: Arc<RwLock<HashMap<TaskId, NodeIndex>>>,
    
    /// Estimativas de execução
    execution_estimates: Arc<RwLock<HashMap<TaskId, ExecutionEstimate>>>,
    
    /// Histórico de performance
    performance_history: Arc<RwLock<HashMap<String, Vec<ExecutionMetrics>>>>,
    
    /// Canal de comunicação
    command_tx: mpsc::UnboundedSender<SchedulerCommand>,
    command_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<SchedulerCommand>>>>,
    
    /// Configuração
    config: SchedulerConfig,
}

/// Comandos do scheduler
#[derive(Debug)]
enum SchedulerCommand {
    ScheduleTask(Task),
    UpdateHeuristic(SchedulingHeuristic),
    RecalculatePlan,
    UpdateEstimate(TaskId, ExecutionEstimate),
    TaskCompleted(TaskId, ExecutionMetrics),
    TaskFailed(TaskId, String),
}

/// Configuração do scheduler
#[derive(Debug, Clone)]
pub struct SchedulerConfig {
    /// Intervalo de replanejamento
    pub replan_interval: Duration,
    /// Habilitar otimização de recursos
    pub enable_resource_optimization: bool,
    /// Fator de segurança para estimativas
    pub safety_factor: f64,
    /// Máximo de tarefas em paralelo
    pub max_parallel_tasks: usize,
    /// Habilitar aprendizado adaptativo
    pub enable_adaptive_learning: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            replan_interval: Duration::from_secs(60),
            enable_resource_optimization: true,
            safety_factor: 1.2,
            max_parallel_tasks: num_cpus::get(),
            enable_adaptive_learning: true,
        }
    }
}

impl Scheduler {
    /// Cria um novo scheduler
    pub fn new(heuristic: SchedulingHeuristic) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        
        info!("Inicializando Scheduler com heurística: {:?}", heuristic);
        
        Self {
            heuristic,
            schedule_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            dependency_graph: Arc::new(RwLock::new(DiGraph::new())),
            node_map: Arc::new(RwLock::new(HashMap::new())),
            execution_estimates: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(HashMap::new())),
            command_tx,
            command_rx: Arc::new(RwLock::new(Some(command_rx))),
            config: SchedulerConfig::default(),
        }
    }

    /// Cria scheduler com configuração personalizada
    pub fn with_config(heuristic: SchedulingHeuristic, config: SchedulerConfig) -> Self {
        let mut scheduler = Self::new(heuristic);
        scheduler.config = config;
        scheduler
    }

    /// Agenda uma nova tarefa
    pub async fn schedule_task(&self, task: Task) -> TaskMeshResult<()> {
        debug!("Agendando tarefa: {} ({})", task.name, task.id);
        
        // Adicionar ao grafo de dependências
        self.add_to_dependency_graph(&task).await?;
        
        // Calcular estimativa de execução
        let estimate = self.estimate_execution(&task).await;
        self.execution_estimates.write().await.insert(task.id, estimate.clone());
        
        // Calcular score de prioridade
        let priority_score = self.calculate_priority_score(&task, &estimate).await;
        
        // Criar item de agendamento
        let schedule_item = ScheduleItem {
            task_id: task.id,
            priority_score,
            estimated_duration: estimate.estimated_duration,
            deadline: task.timeout.map(|timeout| {
                task.created_at + timeout
            }),
            resource_requirements: estimate.resource_requirements,
        };
        
        // Adicionar à fila
        self.schedule_queue.write().await.push(schedule_item);
        
        info!("Tarefa {} agendada com prioridade {:.2}", task.id, priority_score);
        Ok(())
    }

    /// Obtém a próxima tarefa para execução
    pub async fn get_next_task(&self, available_resources: &ResourceAllocation) -> Option<TaskId> {
        let mut queue = self.schedule_queue.write().await;
        
        // Verificar se há tarefas na fila
        if queue.is_empty() {
            return None;
        }
        
        // Encontrar tarefa que pode ser executada com recursos disponíveis
        let mut temp_queue = BinaryHeap::new();
        let mut selected_task = None;
        
        while let Some(item) = queue.pop() {
            if self.can_execute_with_resources(&item, available_resources).await {
                if self.dependencies_satisfied(&item.task_id).await {
                    selected_task = Some(item.task_id);
                    break;
                }
            }
            temp_queue.push(item);
        }
        
        // Restaurar fila
        while let Some(item) = temp_queue.pop() {
            queue.push(item);
        }
        
        if let Some(task_id) = selected_task {
            debug!("Próxima tarefa selecionada: {}", task_id);
        }
        
        selected_task
    }

    /// Gera plano de execução otimizado
    pub async fn generate_execution_plan(&self) -> TaskMeshResult<ExecutionPlan> {
        debug!("Gerando plano de execução");
        
        let graph = self.dependency_graph.read().await;
        
        // Ordenação topológica
        let topo_order = match toposort(&*graph, None) {
            Ok(order) => order,
            Err(cycle) => {
                error!("Ciclo detectado no grafo de dependências: {:?}", cycle);
                return Err(TaskMeshError::CircularDependency(vec![]));
            }
        };
        
        let node_map = self.node_map.read().await;
        let estimates = self.execution_estimates.read().await;
        
        // Converter para TaskIds
        let mut execution_order = Vec::new();
        for node_idx in topo_order {
            if let Some((task_id, _)) = node_map.iter().find(|(_, &idx)| idx == node_idx) {
                execution_order.push(*task_id);
            }
        }
        
        // Identificar grupos paralelos
        let parallel_groups = self.identify_parallel_groups(&execution_order).await;
        
        // Calcular estimativas
        let total_estimated_time = self.calculate_total_time(&execution_order, &estimates);
        let critical_path_length = self.calculate_critical_path(&execution_order, &estimates);
        
        // Calcular métricas
        let plan_metrics = PlanMetrics {
            avg_parallelism: self.calculate_avg_parallelism(&parallel_groups),
            resource_efficiency: self.calculate_resource_efficiency(&execution_order, &estimates).await,
            load_factor: self.calculate_load_factor(&parallel_groups),
            critical_path_length,
        };
        
        let plan = ExecutionPlan {
            execution_order,
            total_estimated_time,
            parallel_groups,
            sync_points: vec![], // TODO: Implementar pontos de sincronização
            plan_metrics,
        };
        
        info!("Plano gerado: {} tarefas, tempo estimado: {:?}", 
              plan.execution_order.len(), plan.total_estimated_time);
        
        Ok(plan)
    }

    /// Atualiza heurística de agendamento
    pub async fn update_heuristic(&mut self, heuristic: SchedulingHeuristic) {
        info!("Atualizando heurística: {:?}", heuristic);
        self.heuristic = heuristic;
        
        // Recalcular prioridades
        self.recalculate_priorities().await;
    }

    /// Relata conclusão de tarefa para aprendizado
    pub async fn report_task_completion(&self, task_id: TaskId, metrics: ExecutionMetrics) {
        debug!("Relatando conclusão da tarefa: {}", task_id);
        
        if self.config.enable_adaptive_learning {
            self.update_performance_history(task_id, metrics).await;
            self.adjust_estimates_based_on_history().await;
        }
    }

    /// Relata falha de tarefa
    pub async fn report_task_failure(&self, task_id: TaskId, error: String) {
        warn!("Relatando falha da tarefa {}: {}", task_id, error);
        
        // TODO: Implementar ajuste de estimativas baseado em falhas
    }

    /// Adiciona tarefa ao grafo de dependências
    async fn add_to_dependency_graph(&self, task: &Task) -> TaskMeshResult<()> {
        let mut graph = self.dependency_graph.write().await;
        let mut node_map = self.node_map.write().await;
        
        // Adicionar nó da tarefa se não existir
        let task_node = if let Some(&node_idx) = node_map.get(&task.id) {
            node_idx
        } else {
            let node_idx = graph.add_node(task.id);
            node_map.insert(task.id, node_idx);
            node_idx
        };
        
        // Adicionar arestas de dependência
        for dep_id in &task.dependencies {
            let dep_node = if let Some(&node_idx) = node_map.get(dep_id) {
                node_idx
            } else {
                let node_idx = graph.add_node(*dep_id);
                node_map.insert(*dep_id, node_idx);
                node_idx
            };
            
            graph.add_edge(dep_node, task_node, ());
        }
        
        Ok(())
    }

    /// Estima tempo de execução de uma tarefa
    async fn estimate_execution(&self, task: &Task) -> ExecutionEstimate {
        // Buscar histórico similar
        let history = self.performance_history.read().await;
        let task_type = self.classify_task(task);
        
        let historical_data = history.get(&task_type)
            .cloned()
            .unwrap_or_default();
        
        let estimated_duration = if historical_data.is_empty() {
            // Estimativa padrão baseada no tipo de tarefa
            self.default_estimate_for_task(task)
        } else {
            // Média ponderada do histórico
            let total_time: Duration = historical_data.iter()
                .map(|m| m.execution_time)
                .sum();
            total_time / historical_data.len() as u32
        };
        
        // Aplicar fator de segurança
        let adjusted_duration = Duration::from_millis(
            (estimated_duration.as_millis() as f64 * self.config.safety_factor) as u64
        );
        
        let confidence = if historical_data.is_empty() {
            0.3 // Baixa confiança sem histórico
        } else {
            (historical_data.len() as f64 / 10.0).min(1.0) // Aumenta com mais dados
        };
        
        ExecutionEstimate {
            estimated_duration: adjusted_duration,
            resource_requirements: ResourceAllocation::default(),
            confidence,
            historical_data,
        }
    }

    /// Calcula score de prioridade baseado na heurística
    async fn calculate_priority_score(&self, task: &Task, estimate: &ExecutionEstimate) -> f64 {
        match &self.heuristic {
            SchedulingHeuristic::FIFO => {
                // Timestamp mais antigo = prioridade mais alta
                -(task.created_at.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() as f64)
            },
            SchedulingHeuristic::LIFO => {
                // Timestamp mais recente = prioridade mais alta
                task.created_at.duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap_or_default().as_secs() as f64
            },
            SchedulingHeuristic::Priority => {
                task.priority as f64
            },
            SchedulingHeuristic::ShortestJobFirst => {
                // Tarefas mais curtas têm prioridade mais alta
                1.0 / (estimate.estimated_duration.as_secs_f64() + 1.0)
            },
            SchedulingHeuristic::EarliestDeadlineFirst => {
                if let Some(timeout) = task.timeout {
                    let deadline = task.created_at + timeout;
                    let time_to_deadline = deadline.duration_since(SystemTime::now())
                        .unwrap_or_default().as_secs_f64();
                    1.0 / (time_to_deadline + 1.0)
                } else {
                    task.priority as f64
                }
            },
            SchedulingHeuristic::CriticalRatio => {
                if let Some(timeout) = task.timeout {
                    let deadline = task.created_at + timeout;
                    let time_to_deadline = deadline.duration_since(SystemTime::now())
                        .unwrap_or_default().as_secs_f64();
                    let processing_time = estimate.estimated_duration.as_secs_f64();
                    
                    if processing_time > 0.0 {
                        time_to_deadline / processing_time
                    } else {
                        f64::INFINITY
                    }
                } else {
                    task.priority as f64
                }
            },
            SchedulingHeuristic::Genetic { .. } => {
                // TODO: Implementar algoritmo genético
                task.priority as f64
            },
            SchedulingHeuristic::Hybrid { primary, secondary, threshold } => {
                let primary_score = self.calculate_priority_score_for_heuristic(task, estimate, primary).await;
                let secondary_score = self.calculate_priority_score_for_heuristic(task, estimate, secondary).await;
                
                // Usar heurística primária se confiança é alta
                if estimate.confidence >= *threshold {
                    primary_score
                } else {
                    // Combinar ambas
                    primary_score * estimate.confidence + secondary_score * (1.0 - estimate.confidence)
                }
            },
        }
    }

    /// Cálculo recursivo para heurísticas híbridas
    async fn calculate_priority_score_for_heuristic(
        &self, 
        task: &Task, 
        estimate: &ExecutionEstimate, 
        heuristic: &SchedulingHeuristic
    ) -> f64 {
        // Implementação similar ao método principal, mas sem recursão infinita
        match heuristic {
            SchedulingHeuristic::Priority => task.priority as f64,
            SchedulingHeuristic::ShortestJobFirst => {
                1.0 / (estimate.estimated_duration.as_secs_f64() + 1.0)
            },
            SchedulingHeuristic::CriticalRatio => {
                if let Some(timeout) = task.timeout {
                    let deadline = task.created_at + timeout;
                    let time_to_deadline = deadline.duration_since(SystemTime::now())
                        .unwrap_or_default().as_secs_f64();
                    let processing_time = estimate.estimated_duration.as_secs_f64();
                    
                    if processing_time > 0.0 {
                        time_to_deadline / processing_time
                    } else {
                        f64::INFINITY
                    }
                } else {
                    task.priority as f64
                }
            },
            _ => task.priority as f64, // Fallback
        }
    }

    /// Verifica se uma tarefa pode ser executada com recursos disponíveis
    async fn can_execute_with_resources(
        &self, 
        item: &ScheduleItem, 
        available: &ResourceAllocation
    ) -> bool {
        let required = &item.resource_requirements;
        
        available.cpu_cores >= required.cpu_cores &&
        available.memory_bytes >= required.memory_bytes
    }

    /// Verifica se dependências estão satisfeitas
    async fn dependencies_satisfied(&self, _task_id: &TaskId) -> bool {
        // TODO: Verificar estado das dependências
        true
    }

    /// Identifica grupos de tarefas que podem executar em paralelo
    async fn identify_parallel_groups(&self, execution_order: &[TaskId]) -> Vec<Vec<TaskId>> {
        let mut groups = Vec::new();
        let mut current_group = Vec::new();
        
        // Implementação simples - TODO: melhorar lógica
        for &task_id in execution_order {
            current_group.push(task_id);
            
            // Limitar tamanho do grupo
            if current_group.len() >= self.config.max_parallel_tasks {
                groups.push(current_group.clone());
                current_group.clear();
            }
        }
        
        if !current_group.is_empty() {
            groups.push(current_group);
        }
        
        groups
    }

    /// Calcula tempo total estimado
    fn calculate_total_time(
        &self, 
        execution_order: &[TaskId], 
        estimates: &HashMap<TaskId, ExecutionEstimate>
    ) -> Duration {
        execution_order.iter()
            .filter_map(|id| estimates.get(id))
            .map(|est| est.estimated_duration)
            .sum()
    }

    /// Calcula caminho crítico
    fn calculate_critical_path(
        &self, 
        execution_order: &[TaskId], 
        estimates: &HashMap<TaskId, ExecutionEstimate>
    ) -> Duration {
        // Implementação simplificada - TODO: algoritmo de caminho crítico real
        self.calculate_total_time(execution_order, estimates)
    }

    /// Calcula paralelismo médio
    fn calculate_avg_parallelism(&self, parallel_groups: &[Vec<TaskId>]) -> f64 {
        if parallel_groups.is_empty() {
            return 0.0;
        }
        
        let total_tasks: usize = parallel_groups.iter().map(|g| g.len()).sum();
        total_tasks as f64 / parallel_groups.len() as f64
    }

    /// Calcula eficiência de recursos
    async fn calculate_resource_efficiency(
        &self, 
        _execution_order: &[TaskId], 
        _estimates: &HashMap<TaskId, ExecutionEstimate>
    ) -> f64 {
        // TODO: Implementar cálculo real de eficiência
        0.8
    }

    /// Calcula fator de carga
    fn calculate_load_factor(&self, parallel_groups: &[Vec<TaskId>]) -> f64 {
        if parallel_groups.is_empty() {
            return 0.0;
        }
        
        let max_group_size = parallel_groups.iter().map(|g| g.len()).max().unwrap_or(0);
        let avg_group_size = self.calculate_avg_parallelism(parallel_groups);
        
        if max_group_size > 0 {
            avg_group_size / max_group_size as f64
        } else {
            0.0
        }
    }

    /// Recalcula prioridades de todas as tarefas
    async fn recalculate_priorities(&self) {
        let mut queue = self.schedule_queue.write().await;
        let estimates = self.execution_estimates.read().await;
        
        let items: Vec<_> = queue.drain().collect();
        
        for mut item in items {
            if let Some(estimate) = estimates.get(&item.task_id) {
                // Criar tarefa temporária para cálculo
                let temp_task = Task {
                    id: item.task_id,
                    name: "temp".to_string(),
                    definition: TaskDefinition::Command("temp".to_string()),
                    dependencies: vec![],
                    priority: 50,
                    metadata: HashMap::new(),
                    created_at: SystemTime::now(),
                    timeout: None,
                    max_retries: 0,
                    tags: vec![],
                };
                
                item.priority_score = self.calculate_priority_score(&temp_task, estimate).await;
            }
            queue.push(item);
        }
    }

    /// Atualiza histórico de performance
    async fn update_performance_history(&self, task_id: TaskId, metrics: ExecutionMetrics) {
        let mut history = self.performance_history.write().await;
        
        // Classificar tipo de tarefa (simplificado)
        let task_type = format!("task_{}", task_id); // TODO: Melhorar classificação
        
        history.entry(task_type)
            .or_insert_with(Vec::new)
            .push(metrics);
        
        // Limitar histórico
        if let Some(task_history) = history.get_mut(&format!("task_{}", task_id)) {
            if task_history.len() > 100 {
                task_history.drain(0..50); // Manter apenas os 50 mais recentes
            }
        }
    }

    /// Ajusta estimativas baseado no histórico
    async fn adjust_estimates_based_on_history(&self) {
        // TODO: Implementar ajuste inteligente de estimativas
    }

    /// Classifica tipo de tarefa para histórico
    fn classify_task(&self, task: &Task) -> String {
        match &task.definition {
            TaskDefinition::Command(_) => "command".to_string(),
            TaskDefinition::PythonScript { .. } => "python".to_string(),
            TaskDefinition::RustFunction { .. } => "rust".to_string(),
            TaskDefinition::HttpRequest { .. } => "http".to_string(),
            TaskDefinition::Workflow { .. } => "workflow".to_string(),
        }
    }

    /// Estimativa padrão para tipos de tarefa
    fn default_estimate_for_task(&self, task: &Task) -> Duration {
        match &task.definition {
            TaskDefinition::Command(_) => Duration::from_secs(30),
            TaskDefinition::PythonScript { .. } => Duration::from_secs(60),
            TaskDefinition::RustFunction { .. } => Duration::from_secs(10),
            TaskDefinition::HttpRequest { .. } => Duration::from_secs(5),
            TaskDefinition::Workflow { .. } => Duration::from_secs(300),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_task(name: &str, priority: Priority) -> Task {
        Task::new(
            name.to_string(),
            TaskDefinition::Command("echo test".to_string()),
            vec![],
        ).with_priority(priority)
    }

    #[tokio::test]
    async fn test_schedule_task() {
        let scheduler = Scheduler::new(SchedulingHeuristic::Priority);
        let task = create_test_task("test", 80);
        
        let result = scheduler.schedule_task(task).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_priority_scheduling() {
        let scheduler = Scheduler::new(SchedulingHeuristic::Priority);
        
        let task1 = create_test_task("low", 20);
        let task2 = create_test_task("high", 80);
        
        scheduler.schedule_task(task1).await.unwrap();
        scheduler.schedule_task(task2).await.unwrap();
        
        let resources = ResourceAllocation::default();
        let next_task = scheduler.get_next_task(&resources).await;
        
        assert!(next_task.is_some());
        // A tarefa de maior prioridade deve ser selecionada
    }

    #[tokio::test]
    async fn test_execution_plan_generation() {
        let scheduler = Scheduler::new(SchedulingHeuristic::Priority);
        
        let task1 = create_test_task("task1", 50);
        let task2 = create_test_task("task2", 60);
        
        scheduler.schedule_task(task1).await.unwrap();
        scheduler.schedule_task(task2).await.unwrap();
        
        let plan = scheduler.generate_execution_plan().await;
        assert!(plan.is_ok());
        
        let plan = plan.unwrap();
        assert_eq!(plan.execution_order.len(), 2);
    }
}


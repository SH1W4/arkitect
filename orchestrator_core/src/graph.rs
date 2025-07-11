//! # Task Mesh Graph
//!
//! Implementação do grafo de tarefas (DAG) com nós e arestas de dependência.

use chrono::{DateTime, Utc};
use petgraph::{Graph, Directed, Direction};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use uuid::Uuid;

use crate::errors::{OrchestratorError, Result};
use crate::layers::ExecutionLayer;

/// Identificador único para tarefas
pub type TaskId = Uuid;

/// Identificador único para arestas de dependência
pub type EdgeId = Uuid;

/// Status de execução de uma tarefa
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Tarefa criada mas não iniciada
    Pending,
    /// Dependências sendo verificadas
    Waiting,
    /// Tarefa em execução
    Running,
    /// Tarefa concluída com sucesso
    Completed,
    /// Tarefa falhou
    Failed,
    /// Tarefa cancelada
    Cancelled,
    /// Tarefa pausada temporariamente
    Paused,
}

/// Prioridade de execução da tarefa
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Tipo de tarefa baseado no esforço estimado
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskType {
    /// Tarefa pequena (< 1h)
    Small,
    /// Tarefa média (1-4h)
    Medium,
    /// Tarefa grande (4-8h)
    Large,
    /// Tarefa muito grande (> 8h)
    ExtraLarge,
}

/// Metadados de execução da tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskMetrics {
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub network_usage: f64,
    pub execution_layer: ExecutionLayer,
    pub retry_count: u32,
    pub error_messages: Vec<String>,
}

/// Nó do grafo representando uma tarefa
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskNode {
    pub id: TaskId,
    pub name: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,
    pub task_type: TaskType,
    pub tags: HashSet<String>,
    pub components: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub scheduled_at: Option<DateTime<Utc>>,
    pub deadline: Option<DateTime<Utc>>,
    pub metrics: TaskMetrics,
    pub configuration: HashMap<String, serde_json::Value>,
    pub execution_context: HashMap<String, serde_json::Value>,
}

impl TaskNode {
    /// Cria uma nova tarefa
    pub fn new(name: String, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            description,
            status: TaskStatus::Pending,
            priority: TaskPriority::Medium,
            task_type: TaskType::Medium,
            tags: HashSet::new(),
            components: Vec::new(),
            created_at: now,
            updated_at: now,
            scheduled_at: None,
            deadline: None,
            metrics: TaskMetrics {
                start_time: None,
                end_time: None,
                cpu_usage: 0.0,
                memory_usage: 0.0,
                network_usage: 0.0,
                execution_layer: ExecutionLayer::Local,
                retry_count: 0,
                error_messages: Vec::new(),
            },
            configuration: HashMap::new(),
            execution_context: HashMap::new(),
        }
    }

    /// Atualiza o status da tarefa
    pub fn update_status(&mut self, status: TaskStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Adiciona uma tag à tarefa
    pub fn add_tag(&mut self, tag: String) {
        self.tags.insert(tag);
        self.updated_at = Utc::now();
    }

    /// Verifica se a tarefa pode ser executada
    pub fn can_execute(&self) -> bool {
        matches!(self.status, TaskStatus::Pending | TaskStatus::Waiting)
    }

    /// Verifica se a tarefa está completa
    pub fn is_complete(&self) -> bool {
        matches!(self.status, TaskStatus::Completed | TaskStatus::Failed | TaskStatus::Cancelled)
    }

    /// Calcula a duração da execução
    pub fn execution_duration(&self) -> Option<chrono::Duration> {
        match (self.metrics.start_time, self.metrics.end_time) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

/// Tipo de dependência entre tarefas
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    /// Dependência hard - deve ser concluída antes
    Hard,
    /// Dependência soft - preferência de ordem
    Soft,
    /// Dependência de recurso - compartilha recurso
    Resource,
    /// Dependência de dados - necessita de saída
    Data,
}

/// Aresta do grafo representando dependência entre tarefas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub id: EdgeId,
    pub source: TaskId,
    pub target: TaskId,
    pub dependency_type: DependencyType,
    pub weight: f64,
    pub metadata: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl DependencyEdge {
    /// Cria uma nova aresta de dependência
    pub fn new(source: TaskId, target: TaskId, dependency_type: DependencyType) -> Self {
        Self {
            id: Uuid::new_v4(),
            source,
            target,
            dependency_type,
            weight: 1.0,
            metadata: HashMap::new(),
            created_at: Utc::now(),
        }
    }

    /// Define o peso da dependência
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = weight;
        self
    }

    /// Adiciona metadados à dependência
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

/// Grafo de tarefas (DAG) principal
#[derive(Debug)]
pub struct TaskMesh {
    graph: Graph<TaskNode, DependencyEdge, Directed>,
    task_index: HashMap<TaskId, petgraph::graph::NodeIndex>,
    edge_index: HashMap<EdgeId, petgraph::graph::EdgeIndex>,
}

impl TaskMesh {
    /// Cria um novo grafo de tarefas vazio
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            task_index: HashMap::new(),
            edge_index: HashMap::new(),
        }
    }

    /// Adiciona uma tarefa ao grafo
    pub fn add_task(&mut self, task: TaskNode) -> Result<TaskId> {
        let task_id = task.id;
        let node_index = self.graph.add_node(task);
        self.task_index.insert(task_id, node_index);
        Ok(task_id)
    }

    /// Adiciona uma dependência entre tarefas
    pub fn add_dependency(&mut self, edge: DependencyEdge) -> Result<EdgeId> {
        let source_idx = self.task_index.get(&edge.source)
            .ok_or_else(|| OrchestratorError::TaskNotFound(edge.source))?;
        let target_idx = self.task_index.get(&edge.target)
            .ok_or_else(|| OrchestratorError::TaskNotFound(edge.target))?;

        let edge_id = edge.id;
        let edge_index = self.graph.add_edge(*source_idx, *target_idx, edge);
        self.edge_index.insert(edge_id, edge_index);
        
        // Verifica se o grafo continua sendo acíclico
        if !petgraph::algo::is_cyclic_directed(&self.graph) {
            Ok(edge_id)
        } else {
            // Remove a aresta que criou o ciclo
            self.graph.remove_edge(edge_index);
            self.edge_index.remove(&edge_id);
            Err(OrchestratorError::CyclicDependency)
        }
    }

    /// Obtém uma tarefa pelo ID
    pub fn get_task(&self, task_id: &TaskId) -> Option<&TaskNode> {
        let node_idx = self.task_index.get(task_id)?;
        self.graph.node_weight(*node_idx)
    }

    /// Obtém uma tarefa mutável pelo ID
    pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut TaskNode> {
        let node_idx = self.task_index.get(task_id)?;
        self.graph.node_weight_mut(*node_idx)
    }

    /// Lista todas as tarefas
    pub fn get_all_tasks(&self) -> Vec<&TaskNode> {
        self.graph.node_weights().collect()
    }

    /// Obtém dependências de uma tarefa (predecessores)
    pub fn get_dependencies(&self, task_id: &TaskId) -> Result<Vec<&TaskNode>> {
        let node_idx = self.task_index.get(task_id)
            .ok_or_else(|| OrchestratorError::TaskNotFound(*task_id))?;
        
        let dependencies = self.graph
            .neighbors_directed(*node_idx, Direction::Incoming)
            .filter_map(|idx| self.graph.node_weight(idx))
            .collect();
        
        Ok(dependencies)
    }

    /// Obtém dependentes de uma tarefa (sucessores)
    pub fn get_dependents(&self, task_id: &TaskId) -> Result<Vec<&TaskNode>> {
        let node_idx = self.task_index.get(task_id)
            .ok_or_else(|| OrchestratorError::TaskNotFound(*task_id))?;
        
        let dependents = self.graph
            .neighbors_directed(*node_idx, Direction::Outgoing)
            .filter_map(|idx| self.graph.node_weight(idx))
            .collect();
        
        Ok(dependents)
    }

    /// Verifica se uma tarefa pode ser executada (todas dependências satisfeitas)
    pub fn can_execute_task(&self, task_id: &TaskId) -> Result<bool> {
        let task = self.get_task(task_id)
            .ok_or_else(|| OrchestratorError::TaskNotFound(*task_id))?;
        
        if !task.can_execute() {
            return Ok(false);
        }

        let dependencies = self.get_dependencies(task_id)?;
        Ok(dependencies.iter().all(|dep| dep.is_complete()))
    }

    /// Obtém tarefas prontas para execução
    pub fn get_ready_tasks(&self) -> Result<Vec<&TaskNode>> {
        let mut ready_tasks = Vec::new();
        
        for task in self.get_all_tasks() {
            if self.can_execute_task(&task.id)? {
                ready_tasks.push(task);
            }
        }
        
        // Ordena por prioridade
        ready_tasks.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        Ok(ready_tasks)
    }

    /// Obtém ordem topológica das tarefas
    pub fn topological_sort(&self) -> Result<Vec<&TaskNode>> {
        let sorted_indices = petgraph::algo::toposort(&self.graph, None)
            .map_err(|_| OrchestratorError::CyclicDependency)?;
        
        let sorted_tasks = sorted_indices
            .iter()
            .filter_map(|idx| self.graph.node_weight(*idx))
            .collect();
        
        Ok(sorted_tasks)
    }

    /// Calcula o caminho crítico
    pub fn critical_path(&self) -> Result<Vec<&TaskNode>> {
        // Implementação básica do caminho crítico
        // TODO: Implementar algoritmo mais sofisticado considerando duração estimada
        self.topological_sort()
    }

    /// Estatísticas do grafo
    pub fn statistics(&self) -> TaskMeshStatistics {
        let total_tasks = self.graph.node_count();
        let total_dependencies = self.graph.edge_count();
        
        let mut status_counts = HashMap::new();
        let mut priority_counts = HashMap::new();
        let mut type_counts = HashMap::new();
        
        for task in self.get_all_tasks() {
            *status_counts.entry(task.status.clone()).or_insert(0) += 1;
            *priority_counts.entry(task.priority.clone()).or_insert(0) += 1;
            *type_counts.entry(task.task_type.clone()).or_insert(0) += 1;
        }
        
        TaskMeshStatistics {
            total_tasks,
            total_dependencies,
            status_counts,
            priority_counts,
            type_counts,
        }
    }
}

impl Default for TaskMesh {
    fn default() -> Self {
        Self::new()
    }
}

/// Estatísticas do Task Mesh
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskMeshStatistics {
    pub total_tasks: usize,
    pub total_dependencies: usize,
    pub status_counts: HashMap<TaskStatus, usize>,
    pub priority_counts: HashMap<TaskPriority, usize>,
    pub type_counts: HashMap<TaskType, usize>,
}

impl fmt::Display for TaskStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskStatus::Pending => write!(f, "Pending"),
            TaskStatus::Waiting => write!(f, "Waiting"),
            TaskStatus::Running => write!(f, "Running"),
            TaskStatus::Completed => write!(f, "Completed"),
            TaskStatus::Failed => write!(f, "Failed"),
            TaskStatus::Cancelled => write!(f, "Cancelled"),
            TaskStatus::Paused => write!(f, "Paused"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = TaskNode::new("Test Task".to_string(), Some("Description".to_string()));
        assert_eq!(task.name, "Test Task");
        assert_eq!(task.status, TaskStatus::Pending);
        assert!(task.can_execute());
    }

    #[test]
    fn test_task_mesh_creation() {
        let mut mesh = TaskMesh::new();
        let task = TaskNode::new("Test".to_string(), None);
        let task_id = task.id;
        
        let result = mesh.add_task(task);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), task_id);
        
        let retrieved_task = mesh.get_task(&task_id);
        assert!(retrieved_task.is_some());
        assert_eq!(retrieved_task.unwrap().name, "Test");
    }

    #[test]
    fn test_dependency_addition() {
        let mut mesh = TaskMesh::new();
        let task1 = TaskNode::new("Task 1".to_string(), None);
        let task2 = TaskNode::new("Task 2".to_string(), None);
        let task1_id = task1.id;
        let task2_id = task2.id;
        
        mesh.add_task(task1).unwrap();
        mesh.add_task(task2).unwrap();
        
        let edge = DependencyEdge::new(task1_id, task2_id, DependencyType::Hard);
        let result = mesh.add_dependency(edge);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_cyclic_dependency_detection() {
        let mut mesh = TaskMesh::new();
        let task1 = TaskNode::new("Task 1".to_string(), None);
        let task2 = TaskNode::new("Task 2".to_string(), None);
        let task1_id = task1.id;
        let task2_id = task2.id;
        
        mesh.add_task(task1).unwrap();
        mesh.add_task(task2).unwrap();
        
        // Adiciona dependência A -> B
        let edge1 = DependencyEdge::new(task1_id, task2_id, DependencyType::Hard);
        mesh.add_dependency(edge1).unwrap();
        
        // Tenta adicionar dependência B -> A (criaria ciclo)
        let edge2 = DependencyEdge::new(task2_id, task1_id, DependencyType::Hard);
        let result = mesh.add_dependency(edge2);
        
        assert!(matches!(result, Err(OrchestratorError::CyclicDependency)));
    }
}


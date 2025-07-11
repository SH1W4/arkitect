//! Registro centralizado de tarefas com metadados e indexação avançada

use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use tracing::{debug, error, info, warn};

use crate::types::*;
use crate::TaskMeshResult;

/// Registro centralizado de tarefas
/// 
/// Mantém um índice de todas as tarefas registradas no sistema,
/// permitindo consultas eficientes por diferentes critérios.
pub struct TaskRegistry {
    /// Tarefas indexadas por ID
    tasks: HashMap<TaskId, Task>,
    
    /// Índice por nome de tarefa
    name_index: HashMap<String, TaskId>,
    
    /// Índice por tags
    tag_index: HashMap<String, HashSet<TaskId>>,
    
    /// Índice por prioridade
    priority_index: HashMap<Priority, HashSet<TaskId>>,
    
    /// Índice de dependências (tarefa -> suas dependências)
    dependency_index: HashMap<TaskId, HashSet<TaskId>>,
    
    /// Índice reverso de dependências (tarefa -> tarefas que dependem dela)
    reverse_dependency_index: HashMap<TaskId, HashSet<TaskId>>,
    
    /// Metadados do registro
    metadata: RegistryMetadata,
}

/// Metadados do registro
#[derive(Debug, Clone)]
struct RegistryMetadata {
    /// Timestamp de criação do registro
    created_at: SystemTime,
    /// Última atualização
    last_updated: SystemTime,
    /// Número total de tarefas registradas
    total_tasks: usize,
    /// Número de tarefas ativas
    active_tasks: usize,
}

impl Default for RegistryMetadata {
    fn default() -> Self {
        let now = SystemTime::now();
        Self {
            created_at: now,
            last_updated: now,
            total_tasks: 0,
            active_tasks: 0,
        }
    }
}

/// Critérios de busca para tarefas
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    /// Filtrar por nome (busca parcial)
    pub name_pattern: Option<String>,
    /// Filtrar por tags
    pub tags: Option<Vec<String>>,
    /// Filtrar por prioridade mínima
    pub min_priority: Option<Priority>,
    /// Filtrar por prioridade máxima
    pub max_priority: Option<Priority>,
    /// Filtrar por metadados
    pub metadata_filter: Option<HashMap<String, String>>,
    /// Incluir apenas tarefas sem dependências
    pub no_dependencies: Option<bool>,
    /// Filtrar por período de criação
    pub created_after: Option<SystemTime>,
    pub created_before: Option<SystemTime>,
}

impl Default for SearchCriteria {
    fn default() -> Self {
        Self {
            name_pattern: None,
            tags: None,
            min_priority: None,
            max_priority: None,
            metadata_filter: None,
            no_dependencies: None,
            created_after: None,
            created_before: None,
        }
    }
}

/// Estatísticas do registro
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryStats {
    /// Número total de tarefas
    pub total_tasks: usize,
    /// Distribuição por prioridade
    pub priority_distribution: HashMap<Priority, usize>,
    /// Tags mais utilizadas
    pub popular_tags: Vec<(String, usize)>,
    /// Média de dependências por tarefa
    pub avg_dependencies: f64,
    /// Número de ciclos detectados
    pub detected_cycles: usize,
}

impl TaskRegistry {
    /// Cria um novo registro de tarefas
    pub fn new() -> Self {
        info!("Inicializando TaskRegistry");
        
        Self {
            tasks: HashMap::new(),
            name_index: HashMap::new(),
            tag_index: HashMap::new(),
            priority_index: HashMap::new(),
            dependency_index: HashMap::new(),
            reverse_dependency_index: HashMap::new(),
            metadata: RegistryMetadata::default(),
        }
    }

    /// Registra uma nova tarefa
    pub fn register_task(&mut self, task: Task) -> TaskMeshResult<()> {
        let task_id = task.id;
        
        debug!("Registrando tarefa: {} ({})", task.name, task_id);

        // Verificar se já existe
        if self.tasks.contains_key(&task_id) {
            warn!("Tarefa {} já registrada, atualizando", task_id);
        }

        // Validar dependências
        self.validate_dependencies(&task)?;

        // Atualizar índices
        self.update_indices(&task);

        // Inserir tarefa
        self.tasks.insert(task_id, task);
        
        // Atualizar metadados
        self.metadata.total_tasks = self.tasks.len();
        self.metadata.last_updated = SystemTime::now();
        
        info!("Tarefa {} registrada com sucesso", task_id);
        Ok(())
    }

    /// Obtém uma tarefa por ID
    pub fn get_task(&self, task_id: &TaskId) -> Option<&Task> {
        self.tasks.get(task_id)
    }

    /// Obtém uma tarefa mutável por ID
    pub fn get_task_mut(&mut self, task_id: &TaskId) -> Option<&mut Task> {
        self.tasks.get_mut(task_id)
    }

    /// Remove uma tarefa do registro
    pub fn unregister_task(&mut self, task_id: &TaskId) -> TaskMeshResult<Task> {
        debug!("Removendo tarefa: {}", task_id);
        
        let task = self.tasks.remove(task_id)
            .ok_or_else(|| TaskMeshError::TaskNotFound(*task_id))?;

        // Remover dos índices
        self.remove_from_indices(&task);
        
        // Atualizar metadados
        self.metadata.total_tasks = self.tasks.len();
        self.metadata.last_updated = SystemTime::now();
        
        info!("Tarefa {} removida", task_id);
        Ok(task)
    }

    /// Lista todas as tarefas
    pub fn list_tasks(&self) -> TaskMeshResult<Vec<Task>> {
        Ok(self.tasks.values().cloned().collect())
    }

    /// Busca tarefas por critérios
    pub fn search_tasks(&self, criteria: &SearchCriteria) -> TaskMeshResult<Vec<&Task>> {
        let mut results: Vec<&Task> = self.tasks.values().collect();

        // Filtrar por nome
        if let Some(pattern) = &criteria.name_pattern {
            results.retain(|task| task.name.contains(pattern));
        }

        // Filtrar por tags
        if let Some(tags) = &criteria.tags {
            results.retain(|task| {
                tags.iter().any(|tag| task.tags.contains(tag))
            });
        }

        // Filtrar por prioridade
        if let Some(min_priority) = criteria.min_priority {
            results.retain(|task| task.priority >= min_priority);
        }
        if let Some(max_priority) = criteria.max_priority {
            results.retain(|task| task.priority <= max_priority);
        }

        // Filtrar por metadados
        if let Some(metadata_filter) = &criteria.metadata_filter {
            results.retain(|task| {
                metadata_filter.iter().all(|(key, value)| {
                    task.metadata.get(key) == Some(value)
                })
            });
        }

        // Filtrar por dependências
        if let Some(true) = criteria.no_dependencies {
            results.retain(|task| task.dependencies.is_empty());
        }

        // Filtrar por data de criação
        if let Some(after) = criteria.created_after {
            results.retain(|task| task.created_at >= after);
        }
        if let Some(before) = criteria.created_before {
            results.retain(|task| task.created_at <= before);
        }

        Ok(results)
    }

    /// Obtém tarefas por tag
    pub fn get_tasks_by_tag(&self, tag: &str) -> Vec<&Task> {
        self.tag_index
            .get(tag)
            .map(|task_ids| {
                task_ids
                    .iter()
                    .filter_map(|id| self.tasks.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Obtém tarefas por prioridade
    pub fn get_tasks_by_priority(&self, priority: Priority) -> Vec<&Task> {
        self.priority_index
            .get(&priority)
            .map(|task_ids| {
                task_ids
                    .iter()
                    .filter_map(|id| self.tasks.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Obtém dependências de uma tarefa
    pub fn get_dependencies(&self, task_id: &TaskId) -> Option<&HashSet<TaskId>> {
        self.dependency_index.get(task_id)
    }

    /// Obtém tarefas que dependem de uma tarefa específica
    pub fn get_dependents(&self, task_id: &TaskId) -> Option<&HashSet<TaskId>> {
        self.reverse_dependency_index.get(task_id)
    }

    /// Verifica se existe dependência circular
    pub fn has_circular_dependency(&self, task: &Task) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        
        self.detect_cycle_dfs(&task.id, &mut visited, &mut rec_stack)
    }

    /// Obtém todas as dependências transitivas de uma tarefa
    pub fn get_transitive_dependencies(&self, task_id: &TaskId) -> HashSet<TaskId> {
        let mut result = HashSet::new();
        let mut to_visit = vec![*task_id];
        
        while let Some(current) = to_visit.pop() {
            if let Some(deps) = self.dependency_index.get(&current) {
                for dep in deps {
                    if result.insert(*dep) {
                        to_visit.push(*dep);
                    }
                }
            }
        }
        
        result
    }

    /// Obtém tarefas prontas para execução (sem dependências não resolvidas)
    pub fn get_ready_tasks(&self, completed_tasks: &HashSet<TaskId>) -> Vec<&Task> {
        self.tasks
            .values()
            .filter(|task| {
                task.dependencies
                    .iter()
                    .all(|dep| completed_tasks.contains(dep))
            })
            .collect()
    }

    /// Gera estatísticas do registro
    pub fn generate_stats(&self) -> RegistryStats {
        let mut priority_distribution = HashMap::new();
        let mut tag_counts = HashMap::new();
        let mut total_dependencies = 0;
        
        for task in self.tasks.values() {
            // Distribuição por prioridade
            *priority_distribution.entry(task.priority).or_insert(0) += 1;
            
            // Contagem de tags
            for tag in &task.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
            
            // Soma de dependências
            total_dependencies += task.dependencies.len();
        }
        
        // Tags mais populares (top 10)
        let mut popular_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        popular_tags.sort_by(|a, b| b.1.cmp(&a.1));
        popular_tags.truncate(10);
        
        let avg_dependencies = if self.tasks.is_empty() {
            0.0
        } else {
            total_dependencies as f64 / self.tasks.len() as f64
        };
        
        RegistryStats {
            total_tasks: self.tasks.len(),
            priority_distribution,
            popular_tags,
            avg_dependencies,
            detected_cycles: self.count_cycles(),
        }
    }

    /// Atualiza os índices para uma nova tarefa
    fn update_indices(&mut self, task: &Task) {
        let task_id = task.id;
        
        // Índice por nome
        self.name_index.insert(task.name.clone(), task_id);
        
        // Índice por tags
        for tag in &task.tags {
            self.tag_index
                .entry(tag.clone())
                .or_insert_with(HashSet::new)
                .insert(task_id);
        }
        
        // Índice por prioridade
        self.priority_index
            .entry(task.priority)
            .or_insert_with(HashSet::new)
            .insert(task_id);
        
        // Índice de dependências
        let deps: HashSet<TaskId> = task.dependencies.iter().cloned().collect();
        self.dependency_index.insert(task_id, deps);
        
        // Índice reverso de dependências
        for dep in &task.dependencies {
            self.reverse_dependency_index
                .entry(*dep)
                .or_insert_with(HashSet::new)
                .insert(task_id);
        }
    }

    /// Remove uma tarefa de todos os índices
    fn remove_from_indices(&mut self, task: &Task) {
        let task_id = task.id;
        
        // Remover do índice de nomes
        self.name_index.remove(&task.name);
        
        // Remover do índice de tags
        for tag in &task.tags {
            if let Some(tag_set) = self.tag_index.get_mut(tag) {
                tag_set.remove(&task_id);
                if tag_set.is_empty() {
                    self.tag_index.remove(tag);
                }
            }
        }
        
        // Remover do índice de prioridade
        if let Some(priority_set) = self.priority_index.get_mut(&task.priority) {
            priority_set.remove(&task_id);
            if priority_set.is_empty() {
                self.priority_index.remove(&task.priority);
            }
        }
        
        // Remover dos índices de dependências
        self.dependency_index.remove(&task_id);
        
        for dep in &task.dependencies {
            if let Some(dependents) = self.reverse_dependency_index.get_mut(dep) {
                dependents.remove(&task_id);
                if dependents.is_empty() {
                    self.reverse_dependency_index.remove(dep);
                }
            }
        }
    }

    /// Valida as dependências de uma tarefa
    fn validate_dependencies(&self, task: &Task) -> TaskMeshResult<()> {
        for dep_id in &task.dependencies {
            if !self.tasks.contains_key(dep_id) {
                return Err(TaskMeshError::TaskNotFound(*dep_id));
            }
        }
        
        // Verificar ciclos
        if self.would_create_cycle(task) {
            return Err(TaskMeshError::CircularDependency(
                task.dependencies.clone()
            ));
        }
        
        Ok(())
    }

    /// Verifica se adicionar uma tarefa criaria um ciclo
    fn would_create_cycle(&self, task: &Task) -> bool {
        for dep_id in &task.dependencies {
            if self.has_path_to(dep_id, &task.id) {
                return true;
            }
        }
        false
    }

    /// Verifica se existe um caminho de uma tarefa para outra
    fn has_path_to(&self, from: &TaskId, to: &TaskId) -> bool {
        if from == to {
            return true;
        }
        
        let mut visited = HashSet::new();
        let mut stack = vec![*from];
        
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            
            if let Some(dependents) = self.reverse_dependency_index.get(&current) {
                for dependent in dependents {
                    if dependent == to {
                        return true;
                    }
                    stack.push(*dependent);
                }
            }
        }
        
        false
    }

    /// Detecta ciclo usando DFS
    fn detect_cycle_dfs(
        &self,
        task_id: &TaskId,
        visited: &mut HashSet<TaskId>,
        rec_stack: &mut HashSet<TaskId>,
    ) -> bool {
        visited.insert(*task_id);
        rec_stack.insert(*task_id);
        
        if let Some(deps) = self.dependency_index.get(task_id) {
            for dep in deps {
                if !visited.contains(dep) {
                    if self.detect_cycle_dfs(dep, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(dep) {
                    return true;
                }
            }
        }
        
        rec_stack.remove(task_id);
        false
    }

    /// Conta o número de ciclos no grafo
    fn count_cycles(&self) -> usize {
        let mut count = 0;
        let mut visited = HashSet::new();
        
        for task_id in self.tasks.keys() {
            if !visited.contains(task_id) {
                let mut rec_stack = HashSet::new();
                if self.detect_cycle_dfs(task_id, &mut visited, &mut rec_stack) {
                    count += 1;
                }
            }
        }
        
        count
    }
}

impl Default for TaskRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn create_test_task(name: &str, deps: Vec<TaskId>) -> Task {
        Task::new(
            name.to_string(),
            TaskDefinition::Command("echo test".to_string()),
            deps,
        )
    }

    #[test]
    fn test_register_task() {
        let mut registry = TaskRegistry::new();
        let task = create_test_task("test", vec![]);
        let task_id = task.id;
        
        let result = registry.register_task(task);
        assert!(result.is_ok());
        assert!(registry.get_task(&task_id).is_some());
    }

    #[test]
    fn test_search_by_tag() {
        let mut registry = TaskRegistry::new();
        let mut task = create_test_task("test", vec![]);
        task.tags.push("test-tag".to_string());
        
        registry.register_task(task).unwrap();
        
        let tasks = registry.get_tasks_by_tag("test-tag");
        assert_eq!(tasks.len(), 1);
    }

    #[test]
    fn test_circular_dependency_detection() {
        let mut registry = TaskRegistry::new();
        
        let task1 = create_test_task("task1", vec![]);
        let task1_id = task1.id;
        registry.register_task(task1).unwrap();
        
        let task2 = create_test_task("task2", vec![task1_id]);
        let task2_id = task2.id;
        registry.register_task(task2).unwrap();
        
        // Tentar criar ciclo
        let task3 = create_test_task("task3", vec![task2_id, task1_id]);
        let result = registry.register_task(task3);
        assert!(result.is_ok()); // Não deve ser um ciclo ainda
    }

    #[test]
    fn test_get_ready_tasks() {
        let mut registry = TaskRegistry::new();
        
        let task1 = create_test_task("task1", vec![]);
        let task1_id = task1.id;
        registry.register_task(task1).unwrap();
        
        let task2 = create_test_task("task2", vec![task1_id]);
        registry.register_task(task2).unwrap();
        
        // Sem tarefas completadas
        let ready = registry.get_ready_tasks(&HashSet::new());
        assert_eq!(ready.len(), 1); // Apenas task1
        
        // Com task1 completada
        let mut completed = HashSet::new();
        completed.insert(task1_id);
        let ready = registry.get_ready_tasks(&completed);
        assert_eq!(ready.len(), 1); // Agora task2 está pronta
    }

    #[test]
    fn test_generate_stats() {
        let mut registry = TaskRegistry::new();
        
        let mut task1 = create_test_task("task1", vec![]);
        task1.priority = 80;
        task1.tags.push("high-priority".to_string());
        
        let mut task2 = create_test_task("task2", vec![]);
        task2.priority = 20;
        task2.tags.push("low-priority".to_string());
        
        registry.register_task(task1).unwrap();
        registry.register_task(task2).unwrap();
        
        let stats = registry.generate_stats();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.priority_distribution.len(), 2);
        assert_eq!(stats.popular_tags.len(), 2);
    }
}


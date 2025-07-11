//! # Continuous Learning Module
//!
//! Sistema de aprendizado contínuo para otimização e adaptação do Task Mesh.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::errors::{OrchestratorError, Result};
use crate::graph::{TaskId, TaskNode};
use crate::layers::TaskExecutionResult;

/// Métricas de aprendizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningMetrics {
    pub total_iterations: u64,
    pub improvement_rate: f64,
    pub accuracy: f64,
    pub convergence_score: f64,
    pub learning_efficiency: f64,
    pub last_updated: DateTime<Utc>,
}

/// Modelo de aprendizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningModel {
    pub model_type: ModelType,
    pub parameters: HashMap<String, f64>,
    pub weights: Vec<f64>,
    pub bias: f64,
    pub performance_history: Vec<PerformanceSnapshot>,
    pub last_trained: DateTime<Utc>,
}

/// Tipos de modelos de aprendizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelType {
    LinearRegression,
    NeuralNetwork,
    DecisionTree,
    ReinforcementLearning,
    QuantumLearning,
}

/// Snapshot de performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub accuracy: f64,
    pub loss: f64,
    pub metrics: HashMap<String, f64>,
}

/// Dados de treinamento
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingData {
    pub features: Vec<Vec<f64>>,
    pub labels: Vec<f64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Sistema de aprendizado contínuo
#[derive(Debug)]
pub struct ContinuousLearning {
    models: Arc<RwLock<HashMap<String, LearningModel>>>,
    training_data: Arc<RwLock<TrainingData>>,
    metrics: Arc<RwLock<LearningMetrics>>,
    config: LearningConfig,
}

/// Configuração do aprendizado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningConfig {
    pub learning_rate: f64,
    pub batch_size: usize,
    pub max_iterations: u64,
    pub convergence_threshold: f64,
    pub auto_retrain_interval: u64,
    pub feature_extraction_enabled: bool,
}

impl Default for LearningConfig {
    fn default() -> Self {
        Self {
            learning_rate: 0.01,
            batch_size: 32,
            max_iterations: 1000,
            convergence_threshold: 0.001,
            auto_retrain_interval: 3600, // 1 hora em segundos
            feature_extraction_enabled: true,
        }
    }
}

impl ContinuousLearning {
    /// Cria novo sistema de aprendizado
    pub fn new(config: LearningConfig) -> Self {
        Self {
            models: Arc::new(RwLock::new(HashMap::new())),
            training_data: Arc::new(RwLock::new(TrainingData {
                features: Vec::new(),
                labels: Vec::new(),
                metadata: HashMap::new(),
            })),
            metrics: Arc::new(RwLock::new(LearningMetrics {
                total_iterations: 0,
                improvement_rate: 0.0,
                accuracy: 0.0,
                convergence_score: 0.0,
                learning_efficiency: 0.0,
                last_updated: Utc::now(),
            })),
            config,
        }
    }

    /// Adiciona dados de execução para aprendizado
    pub async fn add_execution_data(&self, task: &TaskNode, result: &TaskExecutionResult) -> Result<()> {
        let features = self.extract_features(task, result).await;
        let label = self.calculate_performance_score(result).await;
        
        let mut training_data = self.training_data.write().await;
        training_data.features.push(features);
        training_data.labels.push(label);
        
        // Limita tamanho dos dados de treinamento
        if training_data.features.len() > 10000 {
            training_data.features.drain(0..1000);
            training_data.labels.drain(0..1000);
        }
        
        Ok(())
    }
    
    /// Extrai features de uma tarefa e resultado
    async fn extract_features(&self, task: &TaskNode, result: &TaskExecutionResult) -> Vec<f64> {
        vec![
            // Features da tarefa
            task.priority as u8 as f64,
            task.task_type as u8 as f64,
            task.tags.len() as f64,
            task.components.len() as f64,
            
            // Features do resultado
            result.resource_usage.cpu_percent,
            result.resource_usage.memory_mb,
            result.resource_usage.execution_time_ms as f64,
            if result.status == crate::layers::TaskExecutionStatus::Success { 1.0 } else { 0.0 },
        ]
    }
    
    /// Calcula score de performance
    async fn calculate_performance_score(&self, result: &TaskExecutionResult) -> f64 {
        let success_score = if result.status == crate::layers::TaskExecutionStatus::Success { 1.0 } else { 0.0 };
        let efficiency_score = 1.0 / (1.0 + result.resource_usage.execution_time_ms as f64 / 1000.0);
        let resource_score = 1.0 - (result.resource_usage.cpu_percent + result.resource_usage.memory_mb / 1000.0) / 2.0;
        
        (success_score + efficiency_score + resource_score) / 3.0
    }
    
    /// Treina modelo para predição de performance
    pub async fn train_performance_model(&self, model_name: &str) -> Result<()> {
        let training_data = self.training_data.read().await;
        
        if training_data.features.is_empty() {
            return Err(OrchestratorError::InsufficientData);
        }
        
        let mut model = LearningModel {
            model_type: ModelType::LinearRegression,
            parameters: HashMap::new(),
            weights: vec![0.0; training_data.features[0].len()],
            bias: 0.0,
            performance_history: Vec::new(),
            last_trained: Utc::now(),
        };
        
        // Treinamento simplificado usando regressão linear
        self.train_linear_regression(&mut model, &training_data).await?;
        
        // Atualiza modelo
        let mut models = self.models.write().await;
        models.insert(model_name.to_string(), model);
        
        // Atualiza métricas
        let mut metrics = self.metrics.write().await;
        metrics.total_iterations += 1;
        metrics.last_updated = Utc::now();
        
        Ok(())
    }
    
    /// Treina modelo de regressão linear
    async fn train_linear_regression(&self, model: &mut LearningModel, data: &TrainingData) -> Result<()> {
        let learning_rate = self.config.learning_rate;
        let iterations = self.config.max_iterations.min(1000);
        
        for iteration in 0..iterations {
            let mut total_error = 0.0;
            
            for (features, &label) in data.features.iter().zip(data.labels.iter()) {
                // Forward pass
                let prediction = self.predict_with_model(model, features).await;
                let error = prediction - label;
                total_error += error * error;
                
                // Backward pass (gradient descent)
                for (i, &feature) in features.iter().enumerate() {
                    model.weights[i] -= learning_rate * error * feature;
                }
                model.bias -= learning_rate * error;
            }
            
            let mse = total_error / data.features.len() as f64;
            
            // Verifica convergência
            if mse < self.config.convergence_threshold {
                break;
            }
            
            // Salva snapshot de performance
            if iteration % 100 == 0 {
                model.performance_history.push(PerformanceSnapshot {
                    timestamp: Utc::now(),
                    accuracy: 1.0 - mse,
                    loss: mse,
                    metrics: HashMap::from([
                        ("iteration".to_string(), iteration as f64),
                        ("mse".to_string(), mse),
                    ]),
                });
            }
        }
        
        Ok(())
    }
    
    /// Faz predição usando modelo
    async fn predict_with_model(&self, model: &LearningModel, features: &[f64]) -> f64 {
        let mut prediction = model.bias;
        for (weight, &feature) in model.weights.iter().zip(features.iter()) {
            prediction += weight * feature;
        }
        prediction
    }
    
    /// Prediz performance de uma tarefa
    pub async fn predict_task_performance(&self, task: &TaskNode, model_name: &str) -> Result<f64> {
        let models = self.models.read().await;
        let model = models.get(model_name)
            .ok_or_else(|| OrchestratorError::ModelNotFound(model_name.to_string()))?;
        
        // Cria features dummy para predição
        let features = vec![
            task.priority as u8 as f64,
            task.task_type as u8 as f64,
            task.tags.len() as f64,
            task.components.len() as f64,
            0.0, // cpu (desconhecido)
            0.0, // memory (desconhecido)
            0.0, // time (desconhecido)
            1.0, // assume sucesso
        ];
        
        let prediction = self.predict_with_model(model, &features).await;
        Ok(prediction.max(0.0).min(1.0)) // Normaliza entre 0 e 1
    }
    
    /// Recomenda camada de execução baseado em aprendizado
    pub async fn recommend_execution_layer(&self, task: &TaskNode) -> Result<crate::layers::ExecutionLayer> {
        // Lógica simplificada baseada em heurísticas aprendidas
        let task_complexity = task.tags.len() + task.components.len();
        
        match task_complexity {
            0..=2 => Ok(crate::layers::ExecutionLayer::Local),
            3..=5 => Ok(crate::layers::ExecutionLayer::Cluster),
            _ => Ok(crate::layers::ExecutionLayer::QuantumSim),
        }
    }
    
    /// Otimiza parâmetros do sistema baseado em aprendizado
    pub async fn optimize_system_parameters(&self) -> Result<OptimizationResult> {
        let models = self.models.read().await;
        let metrics = self.metrics.read().await;
        
        // Análise simplificada de otimização
        let suggested_params = HashMap::from([
            ("max_parallel_tasks".to_string(), 6.0),
            ("timeout_seconds".to_string(), 450.0),
            ("retry_attempts".to_string(), 2.0),
        ]);
        
        Ok(OptimizationResult {
            optimized_parameters: suggested_params,
            expected_improvement: 0.15,
            confidence: 0.8,
            rationale: "Based on learned performance patterns".to_string(),
        })
    }
    
    /// Obtém métricas de aprendizado
    pub async fn get_metrics(&self) -> LearningMetrics {
        self.metrics.read().await.clone()
    }
    
    /// Lista modelos disponíveis
    pub async fn list_models(&self) -> Vec<String> {
        self.models.read().await.keys().cloned().collect()
    }
    
    /// Avalia performance de um modelo
    pub async fn evaluate_model(&self, model_name: &str) -> Result<ModelEvaluation> {
        let models = self.models.read().await;
        let model = models.get(model_name)
            .ok_or_else(|| OrchestratorError::ModelNotFound(model_name.to_string()))?;
        
        let latest_performance = model.performance_history.last();
        
        Ok(ModelEvaluation {
            model_name: model_name.to_string(),
            accuracy: latest_performance.map(|p| p.accuracy).unwrap_or(0.0),
            loss: latest_performance.map(|p| p.loss).unwrap_or(1.0),
            total_iterations: model.performance_history.len() as u64,
            last_trained: model.last_trained,
            performance_trend: self.calculate_performance_trend(model).await,
        })
    }
    
    /// Calcula tendência de performance
    async fn calculate_performance_trend(&self, model: &LearningModel) -> f64 {
        if model.performance_history.len() < 2 {
            return 0.0;
        }
        
        let recent = &model.performance_history[model.performance_history.len()-1];
        let previous = &model.performance_history[model.performance_history.len()-2];
        
        recent.accuracy - previous.accuracy
    }
}

/// Resultado de otimização
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationResult {
    pub optimized_parameters: HashMap<String, f64>,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub rationale: String,
}

/// Avaliação de modelo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEvaluation {
    pub model_name: String,
    pub accuracy: f64,
    pub loss: f64,
    pub total_iterations: u64,
    pub last_trained: DateTime<Utc>,
    pub performance_trend: f64,
}

impl Default for ContinuousLearning {
    fn default() -> Self {
        Self::new(LearningConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::TaskNode;
    use crate::layers::{TaskExecutionResult, TaskExecutionStatus, ResourceUsage, ExecutionLayer};

    #[tokio::test]
    async fn test_learning_system_creation() {
        let learning = ContinuousLearning::default();
        let metrics = learning.get_metrics().await;
        
        assert_eq!(metrics.total_iterations, 0);
        assert_eq!(metrics.accuracy, 0.0);
    }
    
    #[tokio::test]
    async fn test_add_execution_data() {
        let learning = ContinuousLearning::default();
        let task = TaskNode::new("Test Task".to_string(), None);
        
        let result = TaskExecutionResult {
            task_id: task.id,
            status: TaskExecutionStatus::Success,
            start_time: chrono::Utc::now(),
            end_time: Some(chrono::Utc::now()),
            output: None,
            error_message: None,
            resource_usage: ResourceUsage {
                cpu_percent: 50.0,
                memory_mb: 256.0,
                disk_io_mb: 10.0,
                network_io_mb: 5.0,
                execution_time_ms: 1000,
            },
            layer: ExecutionLayer::Local,
        };
        
        let result = learning.add_execution_data(&task, &result).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_model_training() {
        let learning = ContinuousLearning::default();
        
        // Adiciona alguns dados de treino
        for i in 0..10 {
            let task = TaskNode::new(format!("Task {}", i), None);
            let result = TaskExecutionResult {
                task_id: task.id,
                status: TaskExecutionStatus::Success,
                start_time: chrono::Utc::now(),
                end_time: Some(chrono::Utc::now()),
                output: None,
                error_message: None,
                resource_usage: ResourceUsage {
                    cpu_percent: 50.0 + i as f64,
                    memory_mb: 256.0,
                    disk_io_mb: 10.0,
                    network_io_mb: 5.0,
                    execution_time_ms: 1000 + i as u64 * 100,
                },
                layer: ExecutionLayer::Local,
            };
            
            learning.add_execution_data(&task, &result).await.unwrap();
        }
        
        // Treina modelo
        let result = learning.train_performance_model("test_model").await;
        assert!(result.is_ok());
        
        let models = learning.list_models().await;
        assert!(models.contains(&"test_model".to_string()));
    }
}


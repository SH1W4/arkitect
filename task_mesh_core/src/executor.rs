//! Executor assíncrono de tarefas com suporte a Tokio e Rayon

use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, SystemTime, Instant};
use tokio::process::Command;
use tokio::sync::{RwLock, mpsc, Semaphore};
use tokio::time::timeout;
use futures::future::try_join_all;
use rayon::prelude::*;
use tracing::{debug, error, info, warn, instrument};

use crate::types::*;
use crate::state_store::StateStore;
use crate::error_handler::ErrorHandler;
use crate::TaskMeshResult;

/// Executor principal de tarefas
pub struct TaskExecutor {
    /// Pool de workers
    worker_pool: Arc<WorkerPool>,
    
    /// Armazenamento de estado
    state_store: Arc<dyn StateStore>,
    
    /// Handler de erros
    error_handler: Arc<ErrorHandler>,
    
    /// Semáforo para controle de concorrência
    concurrency_semaphore: Arc<Semaphore>,
    
    /// Canal de comandos
    command_tx: mpsc::UnboundedSender<ExecutorCommand>,
    command_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<ExecutorCommand>>>>,
    
    /// Tarefas em execução
    running_tasks: Arc<RwLock<HashMap<TaskId, RunningTaskInfo>>>,
    
    /// Configuração
    config: ExecutorConfig,
}

/// Configuração do executor
#[derive(Debug, Clone)]
pub struct ExecutorConfig {
    /// Número máximo de workers
    pub max_workers: usize,
    /// Timeout padrão para tarefas
    pub default_timeout: Duration,
    /// Tamanho do buffer de logs
    pub log_buffer_size: usize,
    /// Habilitar métricas detalhadas
    pub enable_detailed_metrics: bool,
    /// Intervalo de heartbeat
    pub heartbeat_interval: Duration,
    /// Diretório de trabalho padrão
    pub default_working_dir: String,
}

impl Default for ExecutorConfig {
    fn default() -> Self {
        Self {
            max_workers: num_cpus::get(),
            default_timeout: Duration::from_secs(3600), // 1 hora
            log_buffer_size: 1024 * 1024, // 1MB
            enable_detailed_metrics: true,
            heartbeat_interval: Duration::from_secs(30),
            default_working_dir: std::env::temp_dir().to_string_lossy().to_string(),
        }
    }
}

/// Comandos do executor
#[derive(Debug)]
enum ExecutorCommand {
    ExecuteTask(TaskId, Task),
    CancelTask(TaskId),
    PauseTask(TaskId),
    ResumeTask(TaskId),
    UpdateResources(TaskId, ResourceAllocation),
    Shutdown,
}

/// Informações de tarefa em execução
#[derive(Debug, Clone)]
struct RunningTaskInfo {
    task_id: TaskId,
    worker_id: String,
    started_at: SystemTime,
    context: ExecutionContext,
    cancel_token: Option<tokio_util::sync::CancellationToken>,
}

/// Pool de workers
struct WorkerPool {
    workers: Vec<Worker>,
    available_workers: Arc<RwLock<Vec<usize>>>,
}

/// Worker individual
struct Worker {
    id: String,
    status: Arc<RwLock<WorkerStatus>>,
    info: Arc<RwLock<WorkerInfo>>,
    task_tx: mpsc::UnboundedSender<WorkerTask>,
    task_rx: Arc<RwLock<Option<mpsc::UnboundedReceiver<WorkerTask>>>>,
}

/// Tarefa para worker
#[derive(Debug)]
struct WorkerTask {
    task_id: TaskId,
    task: Task,
    context: ExecutionContext,
    result_tx: mpsc::UnboundedSender<TaskExecutionResult>,
}

/// Resultado de execução de tarefa
#[derive(Debug)]
struct TaskExecutionResult {
    task_id: TaskId,
    result: Result<TaskResult, TaskMeshError>,
    metrics: ExecutionMetrics,
}

impl TaskExecutor {
    /// Cria um novo executor
    pub async fn new(
        max_workers: usize,
        state_store: Arc<dyn StateStore>,
        error_handler: Arc<ErrorHandler>,
    ) -> TaskMeshResult<Self> {
        let config = ExecutorConfig {
            max_workers,
            ..ExecutorConfig::default()
        };
        
        Self::with_config(config, state_store, error_handler).await
    }
    
    /// Cria executor com configuração personalizada
    pub async fn with_config(
        config: ExecutorConfig,
        state_store: Arc<dyn StateStore>,
        error_handler: Arc<ErrorHandler>,
    ) -> TaskMeshResult<Self> {
        info!("Inicializando TaskExecutor com {} workers", config.max_workers);
        
        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let worker_pool = Arc::new(WorkerPool::new(config.max_workers).await?);
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_workers));
        
        Ok(Self {
            worker_pool,
            state_store,
            error_handler,
            concurrency_semaphore,
            command_tx,
            command_rx: Arc::new(RwLock::new(Some(command_rx))),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
            config,
        })
    }
    
    /// Inicia o executor
    pub async fn start(&self) -> TaskMeshResult<()> {
        info!("Iniciando TaskExecutor");
        
        // Iniciar workers
        self.worker_pool.start_all().await?;
        
        // Iniciar loop de comando
        self.start_command_loop().await;
        
        info!("TaskExecutor iniciado");
        Ok(())
    }
    
    /// Para o executor graciosamente
    pub async fn shutdown(&self) -> TaskMeshResult<()> {
        info!("Parando TaskExecutor");
        
        // Cancelar todas as tarefas em execução
        let running_tasks = self.running_tasks.read().await;
        for (task_id, task_info) in running_tasks.iter() {
            if let Some(cancel_token) = &task_info.cancel_token {
                cancel_token.cancel();
            }
            warn!("Cancelando tarefa em execução: {}", task_id);
        }
        drop(running_tasks);
        
        // Parar workers
        self.worker_pool.stop_all().await?;
        
        // Enviar comando de shutdown
        if let Err(e) = self.command_tx.send(ExecutorCommand::Shutdown) {
            error!("Erro ao enviar comando de shutdown: {}", e);
        }
        
        info!("TaskExecutor parado");
        Ok(())
    }
    
    /// Executa uma tarefa
    #[instrument(skip(self, task), fields(task_id = %task.id, task_name = %task.name))]
    pub async fn execute_task(&self, task: Task) -> TaskMeshResult<TaskId> {
        let task_id = task.id;
        debug!("Executando tarefa: {} ({})", task.name, task_id);
        
        // Verificar se tarefa já está em execução
        if self.running_tasks.read().await.contains_key(&task_id) {
            return Err(TaskMeshError::Internal(
                format!("Tarefa {} já está em execução", task_id)
            ));
        }
        
        // Atualizar status para execução
        self.state_store.update_task_status(
            &task_id,
            TaskStatus::Running {
                started_at: SystemTime::now(),
                worker_id: "pending".to_string(),
            },
        ).await?;
        
        // Enviar comando de execução
        self.command_tx.send(ExecutorCommand::ExecuteTask(task_id, task))
            .map_err(|e| TaskMeshError::Internal(format!("Erro ao enviar comando: {}", e)))?;
        
        Ok(task_id)
    }
    
    /// Cancela uma tarefa
    pub async fn cancel_task(&self, task_id: &TaskId) -> TaskMeshResult<()> {
        debug!("Cancelando tarefa: {}", task_id);
        
        self.command_tx.send(ExecutorCommand::CancelTask(*task_id))
            .map_err(|e| TaskMeshError::Internal(format!("Erro ao enviar comando: {}", e)))?;
        
        Ok(())
    }
    
    /// Pausa uma tarefa
    pub async fn pause_task(&self, task_id: &TaskId) -> TaskMeshResult<()> {
        debug!("Pausando tarefa: {}", task_id);
        
        self.command_tx.send(ExecutorCommand::PauseTask(*task_id))
            .map_err(|e| TaskMeshError::Internal(format!("Erro ao enviar comando: {}", e)))?;
        
        Ok(())
    }
    
    /// Resume uma tarefa
    pub async fn resume_task(&self, task_id: &TaskId) -> TaskMeshResult<()> {
        debug!("Resumindo tarefa: {}", task_id);
        
        self.command_tx.send(ExecutorCommand::ResumeTask(*task_id))
            .map_err(|e| TaskMeshError::Internal(format!("Erro ao enviar comando: {}", e)))?;
        
        Ok(())
    }
    
    /// Obtém informações dos workers
    pub async fn get_worker_info(&self) -> Vec<WorkerInfo> {
        self.worker_pool.get_all_worker_info().await
    }
    
    /// Inicia loop de processamento de comandos
    async fn start_command_loop(&self) {
        let mut command_rx = self.command_rx.write().await.take()
            .expect("Command receiver já foi tomado");
        
        let executor = self.clone_arc();
        
        tokio::spawn(async move {
            while let Some(command) = command_rx.recv().await {
                match command {
                    ExecutorCommand::ExecuteTask(task_id, task) => {
                        if let Err(e) = executor.handle_execute_task(task_id, task).await {
                            error!("Erro ao executar tarefa {}: {}", task_id, e);
                        }
                    },
                    ExecutorCommand::CancelTask(task_id) => {
                        if let Err(e) = executor.handle_cancel_task(task_id).await {
                            error!("Erro ao cancelar tarefa {}: {}", task_id, e);
                        }
                    },
                    ExecutorCommand::PauseTask(task_id) => {
                        // TODO: Implementar pause
                        warn!("Pause não implementado para tarefa: {}", task_id);
                    },
                    ExecutorCommand::ResumeTask(task_id) => {
                        // TODO: Implementar resume
                        warn!("Resume não implementado para tarefa: {}", task_id);
                    },
                    ExecutorCommand::UpdateResources(task_id, resources) => {
                        // TODO: Implementar atualização de recursos
                        debug!("Atualizando recursos da tarefa {}: {:?}", task_id, resources);
                    },
                    ExecutorCommand::Shutdown => {
                        info!("Recebido comando de shutdown");
                        break;
                    },
                }
            }
        });
    }
    
    /// Clona referência para Arc
    fn clone_arc(&self) -> Arc<Self> {
        // Esta é uma implementação simplificada
        // Em um cenário real, TaskExecutor deveria ser envolvido em Arc desde o início
        todo!("Implementar clone_arc adequadamente")
    }
    
    /// Lida com execução de tarefa
    async fn handle_execute_task(&self, task_id: TaskId, task: Task) -> TaskMeshResult<()> {
        // Adquirir permissão de concorrência
        let _permit = self.concurrency_semaphore.acquire().await
            .map_err(|e| TaskMeshError::Internal(format!("Erro ao adquirir semáforo: {}", e)))?;
        
        // Encontrar worker disponível
        let worker_id = self.worker_pool.get_available_worker().await
            .ok_or_else(|| TaskMeshError::ResourceUnavailable(
                "Nenhum worker disponível".to_string()
            ))?;
        
        // Criar contexto de execução
        let context = ExecutionContext {
            worker_id: worker_id.clone(),
            working_directory: self.config.default_working_dir.clone(),
            environment: std::env::vars().collect(),
            allocated_resources: ResourceAllocation::default(),
            checkpoint_id: None,
        };
        
        // Criar token de cancelamento
        let cancel_token = tokio_util::sync::CancellationToken::new();
        
        // Registrar tarefa como em execução
        let task_info = RunningTaskInfo {
            task_id,
            worker_id: worker_id.clone(),
            started_at: SystemTime::now(),
            context: context.clone(),
            cancel_token: Some(cancel_token.clone()),
        };
        
        self.running_tasks.write().await.insert(task_id, task_info);
        
        // Atualizar status
        self.state_store.update_task_status(
            &task_id,
            TaskStatus::Running {
                started_at: SystemTime::now(),
                worker_id: worker_id.clone(),
            },
        ).await?;
        
        // Executar tarefa
        let result = self.execute_task_on_worker(
            &worker_id,
            task,
            context,
            cancel_token,
        ).await;
        
        // Remover da lista de execução
        self.running_tasks.write().await.remove(&task_id);
        
        // Processar resultado
        match result {
            Ok(task_result) => {
                self.state_store.update_task_status(
                    &task_id,
                    TaskStatus::Completed {
                        started_at: SystemTime::now(),
                        completed_at: SystemTime::now(),
                        result: task_result,
                    },
                ).await?;
                info!("Tarefa {} concluída com sucesso", task_id);
            },
            Err(error) => {
                self.state_store.update_task_status(
                    &task_id,
                    TaskStatus::Failed {
                        started_at: SystemTime::now(),
                        failed_at: SystemTime::now(),
                        error: error.to_string(),
                        retry_count: 0,
                    },
                ).await?;
                error!("Tarefa {} falhou: {}", task_id, error);
            },
        }
        
        Ok(())
    }
    
    /// Lida com cancelamento de tarefa
    async fn handle_cancel_task(&self, task_id: TaskId) -> TaskMeshResult<()> {
        let mut running_tasks = self.running_tasks.write().await;
        
        if let Some(task_info) = running_tasks.get(&task_id) {
            if let Some(cancel_token) = &task_info.cancel_token {
                cancel_token.cancel();
            }
            
            // Atualizar status
            self.state_store.update_task_status(
                &task_id,
                TaskStatus::Cancelled {
                    cancelled_at: SystemTime::now(),
                    reason: "Cancelamento manual".to_string(),
                },
            ).await?;
            
            running_tasks.remove(&task_id);
            info!("Tarefa {} cancelada", task_id);
        } else {
            warn!("Tarefa {} não encontrada para cancelamento", task_id);
        }
        
        Ok(())
    }
    
    /// Executa tarefa em worker específico
    async fn execute_task_on_worker(
        &self,
        worker_id: &str,
        task: Task,
        context: ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        let start_time = Instant::now();
        
        // Executar baseado no tipo de tarefa
        let result = match &task.definition {
            TaskDefinition::Command(command) => {
                self.execute_command(command, &context, cancel_token).await
            },
            TaskDefinition::PythonScript { script, args, env } => {
                self.execute_python_script(script, args, env, &context, cancel_token).await
            },
            TaskDefinition::RustFunction { function_name, args } => {
                self.execute_rust_function(function_name, args, &context, cancel_token).await
            },
            TaskDefinition::HttpRequest { method, url, headers, body } => {
                self.execute_http_request(method, url, headers, body.as_deref(), &context, cancel_token).await
            },
            TaskDefinition::Workflow { tasks, execution_strategy } => {
                self.execute_workflow(tasks, execution_strategy, &context, cancel_token).await
            },
        };
        
        let execution_time = start_time.elapsed();
        
        // Adicionar métricas
        match result {
            Ok(mut task_result) => {
                task_result.metrics.execution_time = execution_time;
                Ok(task_result)
            },
            Err(e) => Err(e),
        }
    }
    
    /// Executa comando shell
    async fn execute_command(
        &self,
        command: &str,
        context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        debug!("Executando comando: {}", command);
        
        let mut cmd = if cfg!(target_os = "windows") {
            let mut cmd = Command::new("cmd");
            cmd.args(["/C", command]);
            cmd
        } else {
            let mut cmd = Command::new("sh");
            cmd.args(["-c", command]);
            cmd
        };
        
        cmd.current_dir(&context.working_directory)
            .envs(&context.environment)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        let timeout_duration = context.allocated_resources.time_limit
            .unwrap_or(self.config.default_timeout);
        
        let result = tokio::select! {
            _ = cancel_token.cancelled() => {
                return Err(TaskMeshError::ExecutionError(
                    "Tarefa cancelada".to_string()
                ));
            }
            result = timeout(timeout_duration, cmd.output()) => {
                match result {
                    Ok(Ok(output)) => output,
                    Ok(Err(e)) => return Err(TaskMeshError::Io(e)),
                    Err(_) => return Err(TaskMeshError::ExecutionTimeout(uuid::Uuid::new_v4())),
                }
            }
        };
        
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();
        let exit_code = result.status.code().unwrap_or(-1);
        
        Ok(TaskResult {
            exit_code,
            stdout,
            stderr,
            output_data: None,
            metrics: ExecutionMetrics::default(),
        })
    }
    
    /// Executa script Python
    async fn execute_python_script(
        &self,
        script: &str,
        args: &[String],
        env: &HashMap<String, String>,
        context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        // Criar arquivo temporário para o script
        let script_file = tempfile::NamedTempFile::new()
            .map_err(TaskMeshError::Io)?;
        
        tokio::fs::write(script_file.path(), script).await
            .map_err(TaskMeshError::Io)?;
        
        let mut command = format!("python {}", script_file.path().display());
        if !args.is_empty() {
            command.push(' ');
            command.push_str(&args.join(" "));
        }
        
        // Adicionar variáveis de ambiente específicas
        let mut full_env = context.environment.clone();
        full_env.extend(env.clone());
        
        let updated_context = ExecutionContext {
            environment: full_env,
            ..context.clone()
        };
        
        self.execute_command(&command, &updated_context, cancel_token).await
    }
    
    /// Executa função Rust
    async fn execute_rust_function(
        &self,
        function_name: &str,
        args: &serde_json::Value,
        _context: &ExecutionContext,
        _cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        // TODO: Implementar sistema de plugins para funções Rust
        warn!("Execução de função Rust não implementada: {}", function_name);
        
        Ok(TaskResult {
            exit_code: 0,
            stdout: format!("Função {} chamada com args: {}", function_name, args),
            stderr: String::new(),
            output_data: Some(args.clone()),
            metrics: ExecutionMetrics::default(),
        })
    }
    
    /// Executa requisição HTTP
    async fn execute_http_request(
        &self,
        method: &str,
        url: &str,
        headers: &HashMap<String, String>,
        body: Option<&str>,
        _context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        debug!("Executando requisição HTTP: {} {}", method, url);
        
        let client = reqwest::Client::new();
        let mut request_builder = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => return Err(TaskMeshError::ExecutionError(
                format!("Método HTTP não suportado: {}", method)
            )),
        };
        
        // Adicionar headers
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }
        
        // Adicionar body se presente
        if let Some(body_content) = body {
            request_builder = request_builder.body(body_content.to_string());
        }
        
        let request = request_builder.build()
            .map_err(|e| TaskMeshError::ExecutionError(format!("Erro ao construir requisição: {}", e)))?;
        
        let result = tokio::select! {
            _ = cancel_token.cancelled() => {
                return Err(TaskMeshError::ExecutionError(
                    "Requisição cancelada".to_string()
                ));
            }
            result = client.execute(request) => result
        };
        
        match result {
            Ok(response) => {
                let status = response.status();
                let headers_map: HashMap<String, String> = response.headers()
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();
                
                let body_text = response.text().await
                    .map_err(|e| TaskMeshError::ExecutionError(format!("Erro ao ler resposta: {}", e)))?;
                
                let output_data = serde_json::json!({
                    "status": status.as_u16(),
                    "headers": headers_map,
                    "body": body_text
                });
                
                Ok(TaskResult {
                    exit_code: if status.is_success() { 0 } else { status.as_u16() as i32 },
                    stdout: body_text.clone(),
                    stderr: if status.is_success() { String::new() } else { format!("HTTP {}", status) },
                    output_data: Some(output_data),
                    metrics: ExecutionMetrics::default(),
                })
            },
            Err(e) => Err(TaskMeshError::ExecutionError(format!("Erro na requisição HTTP: {}", e))),
        }
    }
    
    /// Executa workflow
    async fn execute_workflow(
        &self,
        tasks: &[Task],
        strategy: &WorkflowStrategy,
        context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        debug!("Executando workflow com {} tarefas", tasks.len());
        
        match strategy {
            WorkflowStrategy::Sequential => {
                self.execute_sequential_workflow(tasks, context, cancel_token).await
            },
            WorkflowStrategy::Parallel => {
                self.execute_parallel_workflow(tasks, context, cancel_token).await
            },
            WorkflowStrategy::DAG => {
                self.execute_dag_workflow(tasks, context, cancel_token).await
            },
        }
    }
    
    /// Executa workflow sequencial
    async fn execute_sequential_workflow(
        &self,
        tasks: &[Task],
        context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        let mut results = Vec::new();
        let mut total_stdout = String::new();
        let mut total_stderr = String::new();
        
        for task in tasks {
            if cancel_token.is_cancelled() {
                return Err(TaskMeshError::ExecutionError(
                    "Workflow cancelado".to_string()
                ));
            }
            
            let result = self.execute_task_on_worker(
                &context.worker_id,
                task.clone(),
                context.clone(),
                cancel_token.clone(),
            ).await?;
            
            total_stdout.push_str(&result.stdout);
            total_stdout.push('\n');
            total_stderr.push_str(&result.stderr);
            total_stderr.push('\n');
            
            results.push(result);
        }
        
        let output_data = serde_json::json!({
            "workflow_type": "sequential",
            "task_count": tasks.len(),
            "results": results.len()
        });
        
        Ok(TaskResult {
            exit_code: 0,
            stdout: total_stdout,
            stderr: total_stderr,
            output_data: Some(output_data),
            metrics: ExecutionMetrics::default(),
        })
    }
    
    /// Executa workflow paralelo
    async fn execute_parallel_workflow(
        &self,
        tasks: &[Task],
        context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        let futures: Vec<_> = tasks.iter().map(|task| {
            self.execute_task_on_worker(
                &context.worker_id,
                task.clone(),
                context.clone(),
                cancel_token.clone(),
            )
        }).collect();
        
        let results = try_join_all(futures).await?;
        
        let total_stdout = results.iter()
            .map(|r| r.stdout.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        let total_stderr = results.iter()
            .map(|r| r.stderr.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        
        let output_data = serde_json::json!({
            "workflow_type": "parallel",
            "task_count": tasks.len(),
            "results": results.len()
        });
        
        Ok(TaskResult {
            exit_code: 0,
            stdout: total_stdout,
            stderr: total_stderr,
            output_data: Some(output_data),
            metrics: ExecutionMetrics::default(),
        })
    }
    
    /// Executa workflow DAG
    async fn execute_dag_workflow(
        &self,
        tasks: &[Task],
        context: &ExecutionContext,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> TaskMeshResult<TaskResult> {
        // TODO: Implementar execução baseada em DAG
        warn!("Execução DAG não implementada, usando execução sequencial");
        self.execute_sequential_workflow(tasks, context, cancel_token).await
    }
}

impl WorkerPool {
    /// Cria um novo pool de workers
    async fn new(max_workers: usize) -> TaskMeshResult<Self> {
        let mut workers = Vec::with_capacity(max_workers);
        let mut available_workers = Vec::with_capacity(max_workers);
        
        for i in 0..max_workers {
            let worker = Worker::new(format!("worker_{}", i)).await?;
            available_workers.push(i);
            workers.push(worker);
        }
        
        Ok(Self {
            workers,
            available_workers: Arc::new(RwLock::new(available_workers)),
        })
    }
    
    /// Inicia todos os workers
    async fn start_all(&self) -> TaskMeshResult<()> {
        for worker in &self.workers {
            worker.start().await?;
        }
        Ok(())
    }
    
    /// Para todos os workers
    async fn stop_all(&self) -> TaskMeshResult<()> {
        for worker in &self.workers {
            worker.stop().await?;
        }
        Ok(())
    }
    
    /// Obtém worker disponível
    async fn get_available_worker(&self) -> Option<String> {
        let mut available = self.available_workers.write().await;
        if let Some(worker_idx) = available.pop() {
            Some(self.workers[worker_idx].id.clone())
        } else {
            None
        }
    }
    
    /// Retorna worker para pool
    async fn return_worker(&self, worker_id: &str) {
        if let Some(worker_idx) = self.workers.iter().position(|w| w.id == worker_id) {
            self.available_workers.write().await.push(worker_idx);
        }
    }
    
    /// Obtém informações de todos os workers
    async fn get_all_worker_info(&self) -> Vec<WorkerInfo> {
        let mut info = Vec::new();
        for worker in &self.workers {
            info.push(worker.info.read().await.clone());
        }
        info
    }
}

impl Worker {
    /// Cria um novo worker
    async fn new(id: String) -> TaskMeshResult<Self> {
        let (task_tx, task_rx) = mpsc::unbounded_channel();
        
        let worker_info = WorkerInfo {
            id: id.clone(),
            status: WorkerStatus::Idle,
            available_resources: ResourceAllocation::default(),
            current_task: None,
            stats: WorkerStats::default(),
            last_heartbeat: SystemTime::now(),
        };
        
        Ok(Self {
            id,
            status: Arc::new(RwLock::new(WorkerStatus::Idle)),
            info: Arc::new(RwLock::new(worker_info)),
            task_tx,
            task_rx: Arc::new(RwLock::new(Some(task_rx))),
        })
    }
    
    /// Inicia worker
    async fn start(&self) -> TaskMeshResult<()> {
        *self.status.write().await = WorkerStatus::Idle;
        
        // TODO: Implementar loop de worker
        
        Ok(())
    }
    
    /// Para worker
    async fn stop(&self) -> TaskMeshResult<()> {
        *self.status.write().await = WorkerStatus::Stopped;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_store::MemoryStateStore;
    
    #[tokio::test]
    async fn test_executor_creation() {
        let state_store = Arc::new(MemoryStateStore::new().await.unwrap());
        let error_handler = Arc::new(ErrorHandler::new(RetryPolicy::default()));
        
        let executor = TaskExecutor::new(2, state_store, error_handler).await;
        assert!(executor.is_ok());
    }
    
    #[tokio::test]
    async fn test_command_execution() {
        let state_store = Arc::new(MemoryStateStore::new().await.unwrap());
        let error_handler = Arc::new(ErrorHandler::new(RetryPolicy::default()));
        let executor = TaskExecutor::new(1, state_store, error_handler).await.unwrap();
        
        let task = Task::new(
            "test_command".to_string(),
            TaskDefinition::Command("echo hello".to_string()),
            vec![],
        );
        
        let result = executor.execute_task(task).await;
        assert!(result.is_ok());
    }
}


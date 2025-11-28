#!/usr/bin/env python3
"""
ARKITECT OrchestratorAPI - Endpoints Flexíveis

FastAPI com rotas /tasks, /metrics, /alerts, /admin.
Suporta WebSocket e gRPC.
Utiliza Pydantic para validação/sanitização de input.
Configuração dinâmica via .env ou /config endpoint.
"""

import asyncio
import json
import logging
import os
import uuid
from contextlib import asynccontextmanager
from datetime import datetime, timedelta
from typing import Dict, List, Optional, Any, Union

from fastapi import (
    FastAPI, HTTPException, Depends, BackgroundTasks, 
    WebSocket, WebSocketDisconnect, status
)
from fastapi.middleware.cors import CORSMiddleware
from fastapi.middleware.gzip import GZipMiddleware
from fastapi.responses import JSONResponse, PlainTextResponse
from fastapi.security import HTTPBearer, HTTPAuthorizationCredentials
from pydantic import BaseModel, Field, validator, root_validator
from pydantic_settings import BaseSettings
import uvicorn
import grpc
from grpc import aio as aio_grpc
import redis.asyncio as redis

# ============================================================================
# Configurações e Modelos Pydantic
# ============================================================================

class Settings(BaseSettings):
    """Configurações dinâmicas do sistema"""
    
    # Configurações da API
    app_name: str = "ARKITECT OrchestratorAPI"
    app_version: str = "1.0.0"
    debug: bool = False
    
    # Servidor
    host: str = "localhost"
    port: int = 8000
    workers: int = 4
    
    # Segurança
    secret_key: str = "your-secret-key-here"
    algorithm: str = "HS256"
    access_token_expire_minutes: int = 30
    
    # CORS
    cors_origins: List[str] = ["*"]
    
    # Redis
    redis_url: str = "redis://localhost:6379"
    redis_password: Optional[str] = None
    redis_db: int = 0
    
    # gRPC
    grpc_host: str = "localhost"
    grpc_port: int = 50051
    
    # WebSocket
    websocket_timeout: int = 300
    max_websocket_connections: int = 100
    
    # Métricas
    enable_metrics: bool = True
    metrics_port: int = 9090
    
    # Alertas
    enable_alerts: bool = True
    alert_threshold_cpu: float = 80.0
    alert_threshold_memory: float = 85.0
    alert_threshold_disk: float = 90.0
    
    # Logging
    log_level: str = "INFO"
    log_format: str = "json"
    
    class Config:
        env_file = ".env"
        env_file_encoding = "utf-8"
        case_sensitive = False

# Modelos Pydantic para validação de entrada

class TaskCreate(BaseModel):
    """Modelo para criação de tarefa"""
    name: str = Field(..., min_length=1, max_length=255, description="Nome da tarefa")
    description: Optional[str] = Field(None, max_length=1000, description="Descrição da tarefa")
    priority: str = Field("medium", regex="^(low|medium|high|critical)$", description="Prioridade da tarefa")
    layer: str = Field("default", regex="^(default|system|user|admin)$", description="Camada de execução")
    parameters: Dict[str, Any] = Field(default_factory=dict, description="Parâmetros da tarefa")
    timeout: Optional[int] = Field(None, ge=1, le=3600, description="Timeout em segundos")
    dependencies: List[str] = Field(default_factory=list, description="IDs de tarefas dependentes")
    
    @validator("parameters")
    def validate_parameters(cls, v):
        """Valida parâmetros da tarefa"""
        if not isinstance(v, dict):
            raise ValueError("Parâmetros devem ser um dicionário")
        return v

class TaskUpdate(BaseModel):
    """Modelo para atualização de tarefa"""
    name: Optional[str] = Field(None, min_length=1, max_length=255)
    description: Optional[str] = Field(None, max_length=1000)
    priority: Optional[str] = Field(None, regex="^(low|medium|high|critical)$")
    status: Optional[str] = Field(None, regex="^(pending|running|completed|failed|cancelled)$")
    parameters: Optional[Dict[str, Any]] = None

class TaskResponse(BaseModel):
    """Modelo de resposta de tarefa"""
    id: str
    name: str
    description: Optional[str]
    status: str
    priority: str
    layer: str
    parameters: Dict[str, Any]
    created_at: datetime
    updated_at: datetime
    completed_at: Optional[datetime]
    result: Optional[Dict[str, Any]]
    error: Optional[str]

class MetricsResponse(BaseModel):
    """Modelo de resposta de métricas"""
    system_health: Dict[str, float]
    task_metrics: Dict[str, int]
    performance_metrics: Dict[str, float]
    resource_usage: Dict[str, float]
    timestamp: datetime

class AlertCreate(BaseModel):
    """Modelo para criação de alerta"""
    title: str = Field(..., min_length=1, max_length=255)
    message: str = Field(..., min_length=1, max_length=1000)
    severity: str = Field(..., regex="^(info|warning|error|critical)$")
    source: str = Field(..., min_length=1, max_length=100)
    tags: List[str] = Field(default_factory=list)
    metadata: Dict[str, Any] = Field(default_factory=dict)

class AlertResponse(BaseModel):
    """Modelo de resposta de alerta"""
    id: str
    title: str
    message: str
    severity: str
    source: str
    status: str
    tags: List[str]
    metadata: Dict[str, Any]
    created_at: datetime
    acknowledged_at: Optional[datetime]
    resolved_at: Optional[datetime]

class ConfigUpdate(BaseModel):
    """Modelo para atualização de configuração"""
    debug: Optional[bool] = None
    log_level: Optional[str] = Field(None, regex="^(DEBUG|INFO|WARNING|ERROR|CRITICAL)$")
    enable_metrics: Optional[bool] = None
    enable_alerts: Optional[bool] = None
    alert_threshold_cpu: Optional[float] = Field(None, ge=0, le=100)
    alert_threshold_memory: Optional[float] = Field(None, ge=0, le=100)
    alert_threshold_disk: Optional[float] = Field(None, ge=0, le=100)
    websocket_timeout: Optional[int] = Field(None, ge=30, le=3600)
    max_websocket_connections: Optional[int] = Field(None, ge=1, le=1000)

class AdminAction(BaseModel):
    """Modelo para ações administrativas"""
    action: str = Field(..., regex="^(restart|shutdown|maintenance|backup|cleanup)$")
    parameters: Dict[str, Any] = Field(default_factory=dict)
    force: bool = Field(False, description="Forçar execução da ação")

class WebSocketMessage(BaseModel):
    """Modelo para mensagens WebSocket"""
    type: str = Field(..., regex="^(subscribe|unsubscribe|message|heartbeat)$")
    channel: Optional[str] = None
    data: Optional[Dict[str, Any]] = None
    timestamp: datetime = Field(default_factory=datetime.utcnow)

# ============================================================================
# Inicialização e Configurações Globais
# ============================================================================

# Configurações
global_settings = Settings()

# Configuração de logging
logging.basicConfig(
    level=getattr(logging, global_settings.log_level.upper()),
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s"
)
logger = logging.getLogger("orchestrator_api")

# Instâncias globais
redis_client: Optional[redis.Redis] = None
websocket_connections: Dict[str, WebSocket] = {}
tasks_store: Dict[str, Dict[str, Any]] = {}
alerts_store: Dict[str, Dict[str, Any]] = {}
metrics_store: Dict[str, Any] = {}

# Segurança
security = HTTPBearer()

# ============================================================================
# Funções de Utilitário
# ============================================================================

async def get_redis_client() -> redis.Redis:
    """Obtém cliente Redis"""
    global redis_client
    if redis_client is None:
        redis_client = redis.from_url(
            global_settings.redis_url,
            password=global_settings.redis_password,
            db=global_settings.redis_db,
            decode_responses=True
        )
    return redis_client

async def validate_token(credentials: HTTPAuthorizationCredentials = Depends(security)):
    """Valida token de autenticação"""
    # Implementação básica - deve ser expandida com JWT real
    if credentials.credentials != "admin-token":
        raise HTTPException(
            status_code=status.HTTP_401_UNAUTHORIZED,
            detail="Token inválido",
            headers={"WWW-Authenticate": "Bearer"},
        )
    return credentials.credentials

async def broadcast_websocket_message(channel: str, message: Dict[str, Any]):
    """Envia mensagem para todos os clientes WebSocket em um canal"""
    if not websocket_connections:
        return
        
    message_data = {
        "type": "message",
        "channel": channel,
        "data": message,
        "timestamp": datetime.utcnow().isoformat()
    }
    
    disconnected = []
    for connection_id, websocket in websocket_connections.items():
        try:
            await websocket.send_json(message_data)
        except Exception as e:
            logger.warning(f"Failed to send message to {connection_id}: {e}")
            disconnected.append(connection_id)
    
    # Remove conexões desconectadas
    for connection_id in disconnected:
        websocket_connections.pop(connection_id, None)

def generate_task_id() -> str:
    """Gera ID único para tarefa"""
    return f"task_{uuid.uuid4().hex[:12]}"

def generate_alert_id() -> str:
    """Gera ID único para alerta"""
    return f"alert_{uuid.uuid4().hex[:12]}"

async def collect_metrics() -> Dict[str, Any]:
    """Coleta métricas do sistema"""
    import psutil
    
    cpu_percent = psutil.cpu_percent(interval=1)
    memory = psutil.virtual_memory()
    disk = psutil.disk_usage('/')
    
    return {
        "system_health": {
            "cpu_usage": cpu_percent,
            "memory_usage": memory.percent,
            "disk_usage": disk.percent,
            "load_average": psutil.getloadavg()[0] if hasattr(psutil, 'getloadavg') else 0.0
        },
        "task_metrics": {
            "total_tasks": len(tasks_store),
            "pending_tasks": len([t for t in tasks_store.values() if t.get("status") == "pending"]),
            "running_tasks": len([t for t in tasks_store.values() if t.get("status") == "running"]),
            "completed_tasks": len([t for t in tasks_store.values() if t.get("status") == "completed"]),
            "failed_tasks": len([t for t in tasks_store.values() if t.get("status") == "failed"])
        },
        "performance_metrics": {
            "response_time_avg": 0.15,  # Mock data
            "throughput_per_second": 45.2,
            "error_rate": 0.02,
            "success_rate": 0.98
        },
        "resource_usage": {
            "cpu_cores": psutil.cpu_count(),
            "memory_total_gb": round(memory.total / (1024**3), 2),
            "disk_total_gb": round(disk.total / (1024**3), 2),
            "network_connections": len(psutil.net_connections())
        },
        "timestamp": datetime.utcnow()
    }

# ============================================================================
# Lifecycle Management
# ============================================================================

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Gerenciamento do ciclo de vida da aplicação"""
    # Startup
    logger.info(f"Starting {global_settings.app_name} v{global_settings.app_version}")
    
    # Inicializa Redis
    try:
        redis_client = await get_redis_client()
        await redis_client.ping()
        logger.info("Redis connection established")
    except Exception as e:
        logger.warning(f"Redis connection failed: {e}")
    
    # Inicializa métricas
    global metrics_store
    metrics_store = await collect_metrics()
    
    logger.info("API started successfully")
    
    yield
    
    # Shutdown
    logger.info("Shutting down API")
    
    # Fecha conexões WebSocket
    for websocket in websocket_connections.values():
        try:
            await websocket.close()
        except Exception:
            pass
    
    # Fecha Redis
    if redis_client:
        await redis_client.close()
    
    logger.info("API shutdown completed")

# ============================================================================
# Criação da Aplicação FastAPI
# ============================================================================

app = FastAPI(
    title=global_settings.app_name,
    description="API para orquestração simbiótica de tarefas com IA - Endpoints Flexíveis",
    version=global_settings.app_version,
    docs_url="/docs",
    redoc_url="/redoc",
    lifespan=lifespan
)

# Middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=global_settings.cors_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

app.add_middleware(GZipMiddleware, minimum_size=1000)

@app.middleware("http")
async def logging_middleware(request, call_next):
    """Middleware de logging"""
    start_time = datetime.utcnow()
    response = await call_next(request)
    process_time = (datetime.utcnow() - start_time).total_seconds()
    
    logger.info(
        f"{request.method} {request.url.path} - {response.status_code} - {process_time:.3f}s"
    )
    
    return response

# ============================================================================
# Rotas de Health Check e Status
# ============================================================================

@app.get("/health", response_model=Dict[str, str])
async def health_check():
    """Verifica saúde da API"""
    return {
        "status": "healthy",
        "timestamp": datetime.utcnow().isoformat(),
        "version": global_settings.app_version,
        "environment": "development" if global_settings.debug else "production"
    }

@app.get("/status")
async def get_system_status():
    """Obtém status completo do sistema"""
    try:
        metrics = await collect_metrics()
        
        return {
            "api_status": "running",
            "timestamp": datetime.utcnow().isoformat(),
            "uptime_seconds": 3600,  # Mock data
            "version": global_settings.app_version,
            "metrics": metrics,
            "websocket_connections": len(websocket_connections),
            "total_tasks": len(tasks_store),
            "total_alerts": len(alerts_store)
        }
    except Exception as e:
        logger.error(f"Error getting system status: {e}")
        raise HTTPException(status_code=500, detail=str(e))

# ============================================================================
# Rotas de Gerenciamento de Tarefas (/tasks)
# ============================================================================

@app.post("/tasks", response_model=TaskResponse, status_code=201)
async def create_task(task_data: TaskCreate, background_tasks: BackgroundTasks):
    """Cria nova tarefa no sistema"""
    try:
        task_id = generate_task_id()
        now = datetime.utcnow()
        
        # Cria tarefa no store
        task = {
            "id": task_id,
            "name": task_data.name,
            "description": task_data.description,
            "status": "pending",
            "priority": task_data.priority,
            "layer": task_data.layer,
            "parameters": task_data.parameters,
            "timeout": task_data.timeout,
            "dependencies": task_data.dependencies,
            "created_at": now,
            "updated_at": now,
            "completed_at": None,
            "result": None,
            "error": None
        }
        
        tasks_store[task_id] = task
        
        # Envia notificação via WebSocket
        await broadcast_websocket_message("tasks", {
            "action": "task_created",
            "task_id": task_id,
            "task_name": task_data.name
        })
        
        logger.info(f"Task created: {task_id} - {task_data.name}")
        return TaskResponse(**task)
        
    except Exception as e:
        logger.error(f"Error creating task: {e}")
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/tasks", response_model=List[TaskResponse])
async def list_tasks(
    status: Optional[str] = None,
    priority: Optional[str] = None,
    layer: Optional[str] = None,
    limit: int = 100,
    offset: int = 0
):
    """Lista tarefas com filtros opcionais"""
    try:
        # Aplica filtros
        filtered_tasks = list(tasks_store.values())
        
        if status:
            filtered_tasks = [t for t in filtered_tasks if t.get("status") == status]
        if priority:
            filtered_tasks = [t for t in filtered_tasks if t.get("priority") == priority]
        if layer:
            filtered_tasks = [t for t in filtered_tasks if t.get("layer") == layer]
        
        # Aplica paginação
        total = len(filtered_tasks)
        paginated_tasks = filtered_tasks[offset:offset + limit]
        
        return [TaskResponse(**task) for task in paginated_tasks]
        
    except Exception as e:
        logger.error(f"Error listing tasks: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/tasks/{task_id}", response_model=TaskResponse)
async def get_task(task_id: str):
    """Obtém detalhes de uma tarefa específica"""
    try:
        task = tasks_store.get(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found")
            
        return TaskResponse(**task)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting task {task_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.put("/tasks/{task_id}", response_model=TaskResponse)
async def update_task(task_id: str, task_update: TaskUpdate):
    """Atualiza uma tarefa existente"""
    try:
        task = tasks_store.get(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found")
        
        # Atualiza campos fornecidos
        update_data = task_update.dict(exclude_unset=True)
        for field, value in update_data.items():
            task[field] = value
        
        task["updated_at"] = datetime.utcnow()
        tasks_store[task_id] = task
        
        # Notifica via WebSocket
        await broadcast_websocket_message("tasks", {
            "action": "task_updated",
            "task_id": task_id,
            "updates": list(update_data.keys())
        })
            
        logger.info(f"Task updated: {task_id}")
        return TaskResponse(**task)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error updating task {task_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.delete("/tasks/{task_id}", status_code=204)
async def delete_task(task_id: str):
    """Remove uma tarefa do sistema"""
    try:
        if task_id not in tasks_store:
            raise HTTPException(status_code=404, detail="Task not found")
        
        # Remove tarefa
        del tasks_store[task_id]
        
        # Notifica via WebSocket
        await broadcast_websocket_message("tasks", {
            "action": "task_deleted",
            "task_id": task_id
        })
            
        logger.info(f"Task deleted: {task_id}")
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error deleting task {task_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/tasks/{task_id}/execute")
async def execute_task(task_id: str, background_tasks: BackgroundTasks):
    """Executa uma tarefa específica"""
    try:
        task = tasks_store.get(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found")
        
        # Atualiza status para running
        task["status"] = "running"
        task["updated_at"] = datetime.utcnow()
        tasks_store[task_id] = task
        
        # Simula execução (em produção seria integração com o core Rust)
        background_tasks.add_task(simulate_task_execution, task_id)
        
        return {
            "task_id": task_id,
            "status": "running",
            "message": "Task execution started",
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error executing task {task_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/tasks/{task_id}/cancel")
async def cancel_task(task_id: str):
    """Cancela execução de uma tarefa"""
    try:
        task = tasks_store.get(task_id)
        if not task:
            raise HTTPException(status_code=404, detail="Task not found")
        
        if task["status"] not in ["pending", "running"]:
            raise HTTPException(status_code=400, detail="Task cannot be cancelled")
        
        # Atualiza status
        task["status"] = "cancelled"
        task["updated_at"] = datetime.utcnow()
        tasks_store[task_id] = task
        
        # Notifica via WebSocket
        await broadcast_websocket_message("tasks", {
            "action": "task_cancelled",
            "task_id": task_id
        })
            
        logger.info(f"Task cancelled: {task_id}")
        return {
            "task_id": task_id,
            "status": "cancelled",
            "message": "Task cancelled successfully",
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error cancelling task {task_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================================
# Rotas de Métricas (/metrics)
# ============================================================================

@app.get("/metrics", response_model=MetricsResponse)
async def get_system_metrics():
    """Obtém métricas completas do sistema"""
    try:
        global metrics_store
        metrics_store = await collect_metrics()
        return MetricsResponse(**metrics_store)
        
    except Exception as e:
        logger.error(f"Error getting system metrics: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/metrics/prometheus")
async def get_prometheus_metrics():
    """Exporta métricas no formato Prometheus"""
    try:
        metrics = await collect_metrics()
        
        # Formato Prometheus simples
        prometheus_text = f"""# HELP arkitect_cpu_usage CPU usage percentage
# TYPE arkitect_cpu_usage gauge
arkitect_cpu_usage {metrics['system_health']['cpu_usage']}

# HELP arkitect_memory_usage Memory usage percentage
# TYPE arkitect_memory_usage gauge
arkitect_memory_usage {metrics['system_health']['memory_usage']}

# HELP arkitect_tasks_total Total number of tasks
# TYPE arkitect_tasks_total counter
arkitect_tasks_total {metrics['task_metrics']['total_tasks']}

# HELP arkitect_tasks_pending Pending tasks
# TYPE arkitect_tasks_pending gauge
arkitect_tasks_pending {metrics['task_metrics']['pending_tasks']}

# HELP arkitect_tasks_running Running tasks
# TYPE arkitect_tasks_running gauge
arkitect_tasks_running {metrics['task_metrics']['running_tasks']}
"""
        
        return PlainTextResponse(content=prometheus_text, media_type="text/plain")
        
    except Exception as e:
        logger.error(f"Error exporting Prometheus metrics: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/metrics/tasks")
async def get_task_metrics():
    """Obtém métricas específicas de tarefas"""
    try:
        task_stats = {
            "total_tasks": len(tasks_store),
            "by_status": {},
            "by_priority": {},
            "by_layer": {},
            "execution_times": [],
            "success_rate": 0.0
        }
        
        # Calcula estatísticas
        for task in tasks_store.values():
            status = task.get("status", "unknown")
            priority = task.get("priority", "unknown")
            layer = task.get("layer", "unknown")
            
            task_stats["by_status"][status] = task_stats["by_status"].get(status, 0) + 1
            task_stats["by_priority"][priority] = task_stats["by_priority"].get(priority, 0) + 1
            task_stats["by_layer"][layer] = task_stats["by_layer"].get(layer, 0) + 1
            
            # Simula tempo de execução
            if status == "completed":
                task_stats["execution_times"].append(2.5)  # Mock data
        
        # Calcula taxa de sucesso
        completed = task_stats["by_status"].get("completed", 0)
        failed = task_stats["by_status"].get("failed", 0)
        total_finished = completed + failed
        
        if total_finished > 0:
            task_stats["success_rate"] = completed / total_finished
        
        task_stats["timestamp"] = datetime.utcnow().isoformat()
        
        return task_stats
        
    except Exception as e:
        logger.error(f"Error getting task metrics: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/metrics/realtime")
async def get_realtime_metrics():
    """Obtém métricas em tempo real"""
    try:
        import psutil
        
        # Métricas em tempo real
        realtime_data = {
            "cpu_percent": psutil.cpu_percent(interval=0.1),
            "memory_percent": psutil.virtual_memory().percent,
            "active_connections": len(websocket_connections),
            "running_tasks": len([t for t in tasks_store.values() if t.get("status") == "running"]),
            "pending_tasks": len([t for t in tasks_store.values() if t.get("status") == "pending"]),
            "total_alerts": len(alerts_store),
            "unresolved_alerts": len([a for a in alerts_store.values() if a.get("status") != "resolved"]),
            "timestamp": datetime.utcnow().isoformat()
        }
        
        return realtime_data
        
    except Exception as e:
        logger.error(f"Error getting realtime metrics: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================================
# Rotas de Alertas (/alerts)
# ============================================================================

@app.post("/alerts", response_model=AlertResponse, status_code=201)
async def create_alert(alert_data: AlertCreate):
    """Cria novo alerta no sistema"""
    try:
        alert_id = generate_alert_id()
        now = datetime.utcnow()
        
        # Cria alerta no store
        alert = {
            "id": alert_id,
            "title": alert_data.title,
            "message": alert_data.message,
            "severity": alert_data.severity,
            "source": alert_data.source,
            "status": "active",
            "tags": alert_data.tags,
            "metadata": alert_data.metadata,
            "created_at": now,
            "acknowledged_at": None,
            "resolved_at": None
        }
        
        alerts_store[alert_id] = alert
        
        # Notifica via WebSocket
        await broadcast_websocket_message("alerts", {
            "action": "alert_created",
            "alert_id": alert_id,
            "severity": alert_data.severity,
            "title": alert_data.title
        })
        
        logger.info(f"Alert created: {alert_id} - {alert_data.title}")
        return AlertResponse(**alert)
        
    except Exception as e:
        logger.error(f"Error creating alert: {e}")
        raise HTTPException(status_code=400, detail=str(e))

@app.get("/alerts", response_model=List[AlertResponse])
async def list_alerts(
    severity: Optional[str] = None,
    status: Optional[str] = None,
    source: Optional[str] = None,
    limit: int = 100,
    offset: int = 0
):
    """Lista alertas com filtros opcionais"""
    try:
        # Aplica filtros
        filtered_alerts = list(alerts_store.values())
        
        if severity:
            filtered_alerts = [a for a in filtered_alerts if a.get("severity") == severity]
        if status:
            filtered_alerts = [a for a in filtered_alerts if a.get("status") == status]
        if source:
            filtered_alerts = [a for a in filtered_alerts if a.get("source") == source]
        
        # Ordena por data de criação (mais recentes primeiro)
        filtered_alerts.sort(key=lambda x: x.get("created_at", datetime.min), reverse=True)
        
        # Aplica paginação
        paginated_alerts = filtered_alerts[offset:offset + limit]
        
        return [AlertResponse(**alert) for alert in paginated_alerts]
        
    except Exception as e:
        logger.error(f"Error listing alerts: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/alerts/{alert_id}", response_model=AlertResponse)
async def get_alert(alert_id: str):
    """Obtém detalhes de um alerta específico"""
    try:
        alert = alerts_store.get(alert_id)
        if not alert:
            raise HTTPException(status_code=404, detail="Alert not found")
            
        return AlertResponse(**alert)
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error getting alert {alert_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/alerts/{alert_id}/acknowledge")
async def acknowledge_alert(alert_id: str):
    """Reconhece um alerta"""
    try:
        alert = alerts_store.get(alert_id)
        if not alert:
            raise HTTPException(status_code=404, detail="Alert not found")
        
        if alert["status"] != "active":
            raise HTTPException(status_code=400, detail="Alert is not active")
        
        # Atualiza status
        alert["status"] = "acknowledged"
        alert["acknowledged_at"] = datetime.utcnow()
        alerts_store[alert_id] = alert
        
        # Notifica via WebSocket
        await broadcast_websocket_message("alerts", {
            "action": "alert_acknowledged",
            "alert_id": alert_id
        })
        
        logger.info(f"Alert acknowledged: {alert_id}")
        return {
            "alert_id": alert_id,
            "status": "acknowledged",
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error acknowledging alert {alert_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/alerts/{alert_id}/resolve")
async def resolve_alert(alert_id: str):
    """Resolve um alerta"""
    try:
        alert = alerts_store.get(alert_id)
        if not alert:
            raise HTTPException(status_code=404, detail="Alert not found")
        
        # Atualiza status
        alert["status"] = "resolved"
        alert["resolved_at"] = datetime.utcnow()
        if not alert["acknowledged_at"]:
            alert["acknowledged_at"] = datetime.utcnow()
        alerts_store[alert_id] = alert
        
        # Notifica via WebSocket
        await broadcast_websocket_message("alerts", {
            "action": "alert_resolved",
            "alert_id": alert_id
        })
        
        logger.info(f"Alert resolved: {alert_id}")
        return {
            "alert_id": alert_id,
            "status": "resolved",
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error resolving alert {alert_id}: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/alerts/stats")
async def get_alert_stats():
    """Obtém estatísticas de alertas"""
    try:
        stats = {
            "total_alerts": len(alerts_store),
            "by_severity": {},
            "by_status": {},
            "by_source": {},
            "recent_alerts": 0,
            "avg_resolution_time": 0.0,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        # Calcula estatísticas
        resolution_times = []
        recent_threshold = datetime.utcnow() - timedelta(hours=24)
        
        for alert in alerts_store.values():
            severity = alert.get("severity", "unknown")
            status = alert.get("status", "unknown")
            source = alert.get("source", "unknown")
            created_at = alert.get("created_at")
            resolved_at = alert.get("resolved_at")
            
            stats["by_severity"][severity] = stats["by_severity"].get(severity, 0) + 1
            stats["by_status"][status] = stats["by_status"].get(status, 0) + 1
            stats["by_source"][source] = stats["by_source"].get(source, 0) + 1
            
            # Alertas recentes (últimas 24h)
            if created_at and created_at > recent_threshold:
                stats["recent_alerts"] += 1
            
            # Tempo de resolução
            if created_at and resolved_at:
                resolution_time = (resolved_at - created_at).total_seconds() / 3600  # em horas
                resolution_times.append(resolution_time)
        
        # Tempo médio de resolução
        if resolution_times:
            stats["avg_resolution_time"] = sum(resolution_times) / len(resolution_times)
        
        return stats
        
    except Exception as e:
        logger.error(f"Error getting alert stats: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================================
# Rotas Administrativas (/admin)
# ============================================================================

@app.get("/admin/system")
async def get_admin_system_info(token: str = Depends(validate_token)):
    """Obtém informações completas do sistema (requer autenticação)"""
    try:
        import platform
        import psutil
        
        system_info = {
            "platform": {
                "system": platform.system(),
                "release": platform.release(),
                "version": platform.version(),
                "machine": platform.machine(),
                "processor": platform.processor()
            },
            "resources": {
                "cpu_count": psutil.cpu_count(),
                "cpu_percent": psutil.cpu_percent(interval=1),
                "memory_total": psutil.virtual_memory().total,
                "memory_available": psutil.virtual_memory().available,
                "memory_percent": psutil.virtual_memory().percent,
                "disk_usage": dict(psutil.disk_usage('/')._asdict())
            },
            "application": {
                "name": global_settings.app_name,
                "version": global_settings.app_version,
                "debug": global_settings.debug,
                "websocket_connections": len(websocket_connections),
                "tasks_count": len(tasks_store),
                "alerts_count": len(alerts_store)
            },
            "timestamp": datetime.utcnow().isoformat()
        }
        
        return system_info
        
    except Exception as e:
        logger.error(f"Error getting admin system info: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/admin/actions")
async def execute_admin_action(
    action_data: AdminAction, 
    token: str = Depends(validate_token),
    background_tasks: BackgroundTasks = None
):
    """Executa ações administrativas"""
    try:
        action = action_data.action
        parameters = action_data.parameters
        force = action_data.force
        
        logger.info(f"Executing admin action: {action} with parameters: {parameters}")
        
        if action == "restart":
            return {
                "action": action,
                "status": "scheduled",
                "message": "System restart scheduled",
                "timestamp": datetime.utcnow().isoformat()
            }
        
        elif action == "shutdown":
            return {
                "action": action,
                "status": "scheduled",
                "message": "System shutdown scheduled",
                "timestamp": datetime.utcnow().isoformat()
            }
        
        elif action == "maintenance":
            # Ativa modo de manutenção
            return {
                "action": action,
                "status": "activated",
                "message": "Maintenance mode activated",
                "timestamp": datetime.utcnow().isoformat()
            }
        
        elif action == "backup":
            # Inicia backup
            if background_tasks:
                background_tasks.add_task(perform_backup, parameters)
            return {
                "action": action,
                "status": "started",
                "message": "Backup process started",
                "timestamp": datetime.utcnow().isoformat()
            }
        
        elif action == "cleanup":
            # Limpa dados antigos
            cleanup_result = await perform_cleanup(parameters)
            return {
                "action": action,
                "status": "completed",
                "message": "Cleanup completed",
                "result": cleanup_result,
                "timestamp": datetime.utcnow().isoformat()
            }
        
        else:
            raise HTTPException(status_code=400, detail=f"Unknown action: {action}")
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error executing admin action: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/admin/logs")
async def get_system_logs(
    level: Optional[str] = None,
    limit: int = 100,
    token: str = Depends(validate_token)
):
    """Obtém logs do sistema"""
    try:
        # Mock data - em produção seria leitura real dos logs
        logs = [
            {
                "timestamp": (datetime.utcnow() - timedelta(minutes=i)).isoformat(),
                "level": "INFO" if i % 3 == 0 else "DEBUG" if i % 2 == 0 else "WARNING",
                "module": "orchestrator_api",
                "message": f"Sample log message {i}"
            }
            for i in range(min(limit, 50))
        ]
        
        # Filtra por nível se especificado
        if level:
            logs = [log for log in logs if log["level"] == level.upper()]
        
        return {
            "logs": logs,
            "count": len(logs),
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Error getting system logs: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/admin/health")
async def get_admin_health(token: str = Depends(validate_token)):
    """Verifica saúde detalhada do sistema"""
    try:
        import psutil
        
        # Verificações de saúde
        checks = {
            "database": {"status": "healthy", "message": "In-memory store operational"},
            "redis": {"status": "unknown", "message": "Redis connection not tested"},
            "memory": {
                "status": "healthy" if psutil.virtual_memory().percent < global_settings.alert_threshold_memory else "warning",
                "usage": psutil.virtual_memory().percent,
                "threshold": global_settings.alert_threshold_memory
            },
            "cpu": {
                "status": "healthy" if psutil.cpu_percent(interval=1) < global_settings.alert_threshold_cpu else "warning",
                "usage": psutil.cpu_percent(interval=1),
                "threshold": global_settings.alert_threshold_cpu
            },
            "disk": {
                "status": "healthy" if psutil.disk_usage('/').percent < global_settings.alert_threshold_disk else "warning",
                "usage": psutil.disk_usage('/').percent,
                "threshold": global_settings.alert_threshold_disk
            }
        }
        
        # Determina status geral
        overall_status = "healthy"
        for check in checks.values():
            if isinstance(check, dict) and check.get("status") == "warning":
                overall_status = "warning"
            elif isinstance(check, dict) and check.get("status") == "error":
                overall_status = "error"
                break
        
        return {
            "overall_status": overall_status,
            "checks": checks,
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Error getting admin health: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================================
# Configuração Dinâmica (/config)
# ============================================================================

@app.get("/config")
async def get_current_config():
    """Obtém configuração atual do sistema"""
    try:
        # Retorna configurações (sem informações sensíveis)
        config = {
            "app_name": global_settings.app_name,
            "app_version": global_settings.app_version,
            "debug": global_settings.debug,
            "host": global_settings.host,
            "port": global_settings.port,
            "workers": global_settings.workers,
            "cors_origins": global_settings.cors_origins,
            "websocket_timeout": global_settings.websocket_timeout,
            "max_websocket_connections": global_settings.max_websocket_connections,
            "enable_metrics": global_settings.enable_metrics,
            "metrics_port": global_settings.metrics_port,
            "enable_alerts": global_settings.enable_alerts,
            "alert_threshold_cpu": global_settings.alert_threshold_cpu,
            "alert_threshold_memory": global_settings.alert_threshold_memory,
            "alert_threshold_disk": global_settings.alert_threshold_disk,
            "log_level": global_settings.log_level,
            "log_format": global_settings.log_format,
            "timestamp": datetime.utcnow().isoformat()
        }
        
        return config
        
    except Exception as e:
        logger.error(f"Error getting config: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/config")
async def update_config(
    config_update: ConfigUpdate,
    token: str = Depends(validate_token)
):
    """Atualiza configuração do sistema dinamicamente"""
    try:
        global global_settings
        
        # Atualiza configurações
        update_data = config_update.dict(exclude_unset=True)
        updated_fields = []
        
        for field, value in update_data.items():
            if hasattr(global_settings, field):
                old_value = getattr(global_settings, field)
                setattr(global_settings, field, value)
                updated_fields.append(f"{field}: {old_value} -> {value}")
                
                # Aplica mudanças especiais
                if field == "log_level":
                    logging.getLogger().setLevel(getattr(logging, value.upper()))
        
        logger.info(f"Configuration updated: {', '.join(updated_fields)}")
        
        # Notifica via WebSocket
        await broadcast_websocket_message("config", {
            "action": "config_updated",
            "updated_fields": list(update_data.keys()),
            "timestamp": datetime.utcnow().isoformat()
        })
        
        return {
            "status": "success",
            "message": "Configuration updated successfully",
            "updated_fields": updated_fields,
            "timestamp": datetime.utcnow().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Error updating config: {e}")
        raise HTTPException(status_code=500, detail=str(e))


# ============================================================================
# WebSocket Support
# ============================================================================

@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """Endpoint WebSocket para comunicação em tempo real"""
    connection_id = f"ws_{uuid.uuid4().hex[:8]}"
    
    try:
        await websocket.accept()
        websocket_connections[connection_id] = websocket
        
        logger.info(f"WebSocket connected: {connection_id}")
        
        # Envia mensagem de boas-vindas
        await websocket.send_json({
            "type": "welcome",
            "connection_id": connection_id,
            "timestamp": datetime.utcnow().isoformat(),
            "available_channels": ["tasks", "alerts", "metrics", "config"]
        })
        
        # Loop de escuta
        while True:
            try:
                # Timeout para evitar conexões ociosas
                data = await asyncio.wait_for(
                    websocket.receive_json(),
                    timeout=global_settings.websocket_timeout
                )
                
                # Processa mensagem
                await process_websocket_message(websocket, connection_id, data)
                
            except asyncio.TimeoutError:
                logger.warning(f"WebSocket timeout: {connection_id}")
                break
            except WebSocketDisconnect:
                logger.info(f"WebSocket disconnected: {connection_id}")
                break
                
    except Exception as e:
        logger.error(f"WebSocket error: {connection_id} - {e}")
    finally:
        # Remove conexão
        websocket_connections.pop(connection_id, None)
        logger.info(f"WebSocket cleanup: {connection_id}")

async def process_websocket_message(websocket: WebSocket, connection_id: str, data: dict):
    """Processa mensagens recebidas via WebSocket"""
    try:
        message = WebSocketMessage(**data)
        
        if message.type == "subscribe":
            # Lógica de inscrição em canais
            await websocket.send_json({
                "type": "subscription_confirmed",
                "channel": message.channel,
                "timestamp": datetime.utcnow().isoformat()
            })
            
        elif message.type == "heartbeat":
            # Resposta ao heartbeat
            await websocket.send_json({
                "type": "heartbeat_response",
                "timestamp": datetime.utcnow().isoformat()
            })
            
        elif message.type == "message":
            # Processa mensagem customizada
            logger.info(f"WebSocket message from {connection_id}: {message.data}")
            
    except Exception as e:
        logger.error(f"Error processing WebSocket message: {e}")
        await websocket.send_json({
            "type": "error",
            "message": "Failed to process message",
            "timestamp": datetime.utcnow().isoformat()
        })


# ============================================================================
# Funções de Background e Utilitários
# ============================================================================

async def simulate_task_execution(task_id: str):
    """Simula execução de tarefa"""
    try:
        task = tasks_store.get(task_id)
        if not task:
            return
        
        # Simula processamento
        await asyncio.sleep(2)
        
        # Simula sucesso/falha (90% sucesso)
        import random
        success = random.random() > 0.1
        
        if success:
            task["status"] = "completed"
            task["result"] = {"output": "Task completed successfully", "value": random.randint(1, 100)}
            task["completed_at"] = datetime.utcnow()
        else:
            task["status"] = "failed"
            task["error"] = "Simulated task failure"
        
        task["updated_at"] = datetime.utcnow()
        tasks_store[task_id] = task
        
        # Notifica via WebSocket
        await broadcast_websocket_message("tasks", {
            "action": "task_completed" if success else "task_failed",
            "task_id": task_id,
            "status": task["status"]
        })
        
        # Cria alerta se falhou
        if not success:
            alert_id = generate_alert_id()
            alert = {
                "id": alert_id,
                "title": f"Task Execution Failed",
                "message": f"Task {task_id} ({task['name']}) failed to execute",
                "severity": "error",
                "source": "task_executor",
                "status": "active",
                "tags": ["task_failure", "execution"],
                "metadata": {"task_id": task_id, "task_name": task["name"]},
                "created_at": datetime.utcnow(),
                "acknowledged_at": None,
                "resolved_at": None
            }
            alerts_store[alert_id] = alert
            
            await broadcast_websocket_message("alerts", {
                "action": "alert_created",
                "alert_id": alert_id,
                "severity": "error",
                "title": alert["title"]
            })
        
        logger.info(f"Task execution completed: {task_id} - {'success' if success else 'failed'}")
        
    except Exception as e:
        logger.error(f"Error simulating task execution {task_id}: {e}")

async def perform_backup(parameters: Dict[str, Any]):
    """Realiza backup do sistema"""
    try:
        logger.info("Starting system backup")
        
        # Simula backup
        await asyncio.sleep(5)
        
        backup_data = {
            "tasks": dict(tasks_store),
            "alerts": dict(alerts_store),
            "metrics": dict(metrics_store),
            "timestamp": datetime.utcnow().isoformat()
        }
        
        # Em produção, salvaria em arquivo ou storage externo
        logger.info("System backup completed")
        
        return backup_data
        
    except Exception as e:
        logger.error(f"Error performing backup: {e}")
        raise

async def perform_cleanup(parameters: Dict[str, Any]):
    """Realiza limpeza de dados antigos"""
    try:
        cleanup_days = parameters.get("days", 30)
        threshold = datetime.utcnow() - timedelta(days=cleanup_days)
        
        # Limpa tarefas antigas
        old_tasks = [
            task_id for task_id, task in tasks_store.items()
            if task.get("created_at", datetime.utcnow()) < threshold
            and task.get("status") in ["completed", "failed", "cancelled"]
        ]
        
        for task_id in old_tasks:
            del tasks_store[task_id]
        
        # Limpa alertas resolvidos antigos
        old_alerts = [
            alert_id for alert_id, alert in alerts_store.items()
            if alert.get("created_at", datetime.utcnow()) < threshold
            and alert.get("status") == "resolved"
        ]
        
        for alert_id in old_alerts:
            del alerts_store[alert_id]
        
        result = {
            "tasks_cleaned": len(old_tasks),
            "alerts_cleaned": len(old_alerts),
            "threshold_date": threshold.isoformat()
        }
        
        logger.info(f"Cleanup completed: {result}")
        return result
        
    except Exception as e:
        logger.error(f"Error performing cleanup: {e}")
        raise


# ============================================================================
# gRPC Support (Placeholder)
# ============================================================================

class OrchestratorgRPCService:
    """Serviço gRPC para integração com componentes Rust"""
    
    async def CreateTask(self, request):
        """Cria tarefa via gRPC"""
        # Placeholder - implementação real requer proto definitions
        logger.info("gRPC CreateTask called")
        return {"task_id": generate_task_id(), "status": "created"}
    
    async def GetTaskStatus(self, request):
        """Obtém status de tarefa via gRPC"""
        task_id = request.get("task_id")
        task = tasks_store.get(task_id)
        
        if task:
            return {"task_id": task_id, "status": task["status"]}
        else:
            return {"task_id": task_id, "status": "not_found"}
    
    async def StreamMetrics(self, request):
        """Stream de métricas via gRPC"""
        while True:
            metrics = await collect_metrics()
            yield metrics
            await asyncio.sleep(5)

async def start_grpc_server():
    """Inicia servidor gRPC"""
    try:
        # Placeholder - implementação real requer configuração completa do gRPC
        logger.info(f"gRPC server would start on {global_settings.grpc_host}:{global_settings.grpc_port}")
        # server = aio_grpc.server()
        # await server.start()
    except Exception as e:
        logger.error(f"Error starting gRPC server: {e}")

# ============================================================================
# Background Tasks de Monitoramento
# ============================================================================

async def monitor_system_health():
    """Monitora saúde do sistema e cria alertas"""
    try:
        while True:
            if global_settings.enable_alerts:
                metrics = await collect_metrics()
                
                # Verifica thresholds
                cpu_usage = metrics["system_health"]["cpu_usage"]
                memory_usage = metrics["system_health"]["memory_usage"]
                disk_usage = metrics["system_health"]["disk_usage"]
                
                # Cria alertas se necessário
                if cpu_usage > global_settings.alert_threshold_cpu:
                    await create_system_alert(
                        "High CPU Usage",
                        f"CPU usage is at {cpu_usage:.1f}%, exceeding threshold of {global_settings.alert_threshold_cpu}%",
                        "warning",
                        "system_monitor"
                    )
                
                if memory_usage > global_settings.alert_threshold_memory:
                    await create_system_alert(
                        "High Memory Usage",
                        f"Memory usage is at {memory_usage:.1f}%, exceeding threshold of {global_settings.alert_threshold_memory}%",
                        "warning",
                        "system_monitor"
                    )
                
                if disk_usage > global_settings.alert_threshold_disk:
                    await create_system_alert(
                        "High Disk Usage",
                        f"Disk usage is at {disk_usage:.1f}%, exceeding threshold of {global_settings.alert_threshold_disk}%",
                        "critical",
                        "system_monitor"
                    )
            
            await asyncio.sleep(60)  # Verifica a cada minuto
            
    except Exception as e:
        logger.error(f"Error in system health monitoring: {e}")

async def create_system_alert(title: str, message: str, severity: str, source: str):
    """Cria alerta de sistema"""
    try:
        # Verifica se já existe alerta similar ativo
        similar_alert = None
        for alert in alerts_store.values():
            if (alert.get("title") == title and 
                alert.get("source") == source and 
                alert.get("status") == "active"):
                similar_alert = alert
                break
        
        if similar_alert:
            # Atualiza timestamp do alerta existente
            similar_alert["metadata"]["last_occurrence"] = datetime.utcnow().isoformat()
            return
        
        # Cria novo alerta
        alert_id = generate_alert_id()
        alert = {
            "id": alert_id,
            "title": title,
            "message": message,
            "severity": severity,
            "source": source,
            "status": "active",
            "tags": ["system", "monitoring"],
            "metadata": {"auto_generated": True},
            "created_at": datetime.utcnow(),
            "acknowledged_at": None,
            "resolved_at": None
        }
        
        alerts_store[alert_id] = alert
        
        # Notifica via WebSocket
        await broadcast_websocket_message("alerts", {
            "action": "alert_created",
            "alert_id": alert_id,
            "severity": severity,
            "title": title
        })
        
        logger.warning(f"System alert created: {alert_id} - {title}")
        
    except Exception as e:
        logger.error(f"Error creating system alert: {e}")


# ============================================================================
# Startup Tasks
# ============================================================================

@app.on_event("startup")
async def startup_event():
    """Inicialização da aplicação"""
    logger.info(f"Starting {global_settings.app_name} v{global_settings.app_version}")
    
    # Inicia monitoramento de saúde em background
    if global_settings.enable_alerts:
        asyncio.create_task(monitor_system_health())
    
    # Inicia servidor gRPC (placeholder)
    # asyncio.create_task(start_grpc_server())
    
    logger.info("API started successfully")

@app.on_event("shutdown")
async def shutdown_event():
    """Finalização da aplicação"""
    logger.info("Shutting down API")
    
    # Fecha conexões WebSocket
    for websocket in websocket_connections.values():
        try:
            await websocket.close()
        except Exception:
            pass
    
    logger.info("API shutdown completed")


# ============================================================================
# Ponto de Entrada
# ============================================================================

if __name__ == "__main__":
    # Configurações para desenvolvimento
    uvicorn_config = {
        "app": "main:app",
        "host": global_settings.host,
        "port": global_settings.port,
        "reload": global_settings.debug,
        "log_level": global_settings.log_level.lower(),
        "access_log": True
    }
    
    logger.info(f"Starting server with config: {uvicorn_config}")
    uvicorn.run(**uvicorn_config)


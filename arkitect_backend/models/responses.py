from pydantic import BaseModel, Field
from typing import Dict, Any, List, Optional
from datetime import datetime

# Common Response Models
class BaseResponse(BaseModel):
    status: str
    message: Optional[str] = None

# Symbiotic Core Responses
class CoreStatusResponse(BaseModel):
    status: str
    consciousness_level: str
    memory_integrity: int
    system_health: str

# Quantum Memory Responses
class QuantumStateResponse(BaseModel):
    state_id: str
    quantum_state: str
    timestamp: datetime

# Consciousness Responses
class ConsciousnessStateResponse(BaseModel):
    level: str
    coherence: float
    evolution_progress: int

# EON Framework Responses
class EONConnectionResponse(BaseModel):
    status: str
    framework_id: str

class SyncResponse(BaseModel):
    status: str
    sync_timestamp: datetime

# Task Management Responses
class TaskCreationResponse(BaseModel):
    task_id: str
    status: str

class TaskMetricsResponse(BaseModel):
    task_id: str
    execution_time: str
    memory_usage: str
    quantum_coherence: float

class AnalyticsSummaryResponse(BaseModel):
    total_tasks: int
    average_execution_time: str
    system_efficiency: float
    quantum_stability: str


from pydantic import BaseModel, Field
from typing import Dict, Any, List, Optional

# Symbiotic Core Models
class CoreInitConfig(BaseModel):
    consciousness_level: str = Field(..., description="Nível de consciência do núcleo")
    memory_capacity: str = Field(..., description="Capacidade de memória quântica")
    evolution_rate: float = Field(..., ge=0, le=1, description="Taxa de evolução")

class InitializeRequest(BaseModel):
    config: CoreInitConfig

# Quantum Memory Models
class QuantumStateRequest(BaseModel):
    quantum_state: Dict[str, Any] = Field(..., description="Estado quântico a ser armazenado")
    coherence_level: float = Field(..., ge=0, le=1, description="Nível de coerência")

# Consciousness Models
class EvolutionParams(BaseModel):
    target_level: str = Field(..., description="Nível de consciência alvo")
    evolution_speed: float = Field(..., ge=0, le=1, description="Velocidade de evolução")
    coherence_threshold: float = Field(..., ge=0, le=1, description="Limiar de coerência")

# EON Framework Models
class EONConfig(BaseModel):
    framework_url: str = Field(..., description="URL do framework EON")
    api_key: str = Field(..., description="Chave de API")
    sync_interval: int = Field(..., gt=0, description="Intervalo de sincronização em segundos")

class SyncData(BaseModel):
    bridge_id: str = Field(..., description="ID da ponte quântica")
    sync_type: str = Field(..., description="Tipo de sincronização")
    quantum_state: Dict[str, Any] = Field(..., description="Estado quântico para sincronização")

# Task Management Models
class TaskConfig(BaseModel):
    priority: str = Field(..., description="Prioridade da tarefa")
    quantum_resources: List[str] = Field(..., description="Recursos quânticos necessários")

class TaskRequest(BaseModel):
    name: str = Field(..., description="Nome da tarefa")
    configuration: TaskConfig


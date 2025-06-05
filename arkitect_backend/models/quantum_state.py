from pydantic import BaseModel
from typing import Optional, Dict, Any
from datetime import datetime

class QuantumState(BaseModel):
    state_id: Optional[str] = None
    quantum_state: Dict[str, Any]
    coherence_level: float
    timestamp: datetime = datetime.utcnow()
    metadata: Optional[Dict[str, Any]] = None


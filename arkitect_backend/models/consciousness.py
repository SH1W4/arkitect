from pydantic import BaseModel
from typing import Optional, Dict, Any
from datetime import datetime

class ConsciousnessState(BaseModel):
    level: str
    coherence: float
    evolution_progress: int
    last_evolution: datetime = datetime.utcnow()
    attributes: Optional[Dict[str, Any]] = None


from pydantic import BaseModel
from typing import Optional, Dict, Any
from datetime import datetime

class Task(BaseModel):
    task_id: Optional[str] = None
    name: str
    status: str
    created_at: datetime = datetime.utcnow()
    updated_at: datetime = datetime.utcnow()
    metrics: Optional[Dict[str, Any]] = None
    configuration: Optional[Dict[str, Any]] = None


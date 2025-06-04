from fastapi import APIRouter
from pydantic import BaseModel
from arkitect_engine.engine_interface import ARKSemanticEngine

router = APIRouter()

class SeedInput(BaseModel):
    seed: str

engine = ARKSemanticEngine()

@router.post("/ark/process/")
def process_ark(input_data: SeedInput):
    result = engine.process(input_data.seed)
    return result

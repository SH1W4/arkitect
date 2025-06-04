# app/api/parsing_routes.py
from fastapi import APIRouter, UploadFile
from app.core import parsing
import shutil
import os
import uuid

router = APIRouter()

@router.post("/parse_structure/")
async def parse_structure(file: UploadFile):
    """Recebe um projeto zip, extrai e retorna estrutura."""
    temp_id = str(uuid.uuid4())
    temp_path = f"temp_projects/{temp_id}.zip"
    
    with open(temp_path, "wb") as buffer:
        shutil.copyfileobj(file.file, buffer)
    
    project_root = parsing.extract_and_parse_zip(temp_path)
    return {"structure": project_root}

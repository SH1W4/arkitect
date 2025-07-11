"""
API de Integração do Agent GitTask

Endpoints FastAPI para integração com o sistema GitTaskAgent,
permitindo operações Git automatizadas via REST API.
"""

from fastapi import APIRouter, HTTPException, BackgroundTasks
from pydantic import BaseModel, Field
from typing import List, Optional, Dict, Any
from datetime import datetime
import asyncio
import logging

from .git_task_agent import (
    GitTaskAgent, TaskNode, GitOperation, GitOperationStatus,
    ConventionalCommitType, GitAgentConsciousness
)

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/git-agent", tags=["Git Task Agent"])

# Instância global do agente
git_agent: Optional[GitTaskAgent] = None

class TaskNodeRequest(BaseModel):
    """Modelo de requisição para TaskNode"""
    id: str = Field(..., description="ID único da tarefa")
    type: str = Field(default="git", description="Tipo da tarefa")
    description: str = Field(..., description="Descrição da tarefa")
    files: List[str] = Field(default_factory=list, description="Arquivos a serem commitados")
    scope: Optional[str] = Field(None, description="Escopo do commit")
    breaking_change: bool = Field(default=False, description="Se é uma breaking change")
    metadata: Dict[str, Any] = Field(default_factory=dict, description="Metadados adicionais")
    
class GitOperationResponse(BaseModel):
    """Resposta de operação Git"""
    task_id: str
    status: str
    commit_type: str
    commit_message: str
    files: List[str]
    dry_run: bool
    quantum_signature: Optional[str]
    execution_log: List[str]
    created_at: datetime
    
class BatchExecuteRequest(BaseModel):
    """Requisição para execução em lote"""
    task_nodes: List[TaskNodeRequest]
    dry_run: bool = Field(default=False, description="Executar em modo dry-run")
    auto_push: bool = Field(default=False, description="Fazer push automático após commits")
    branch: str = Field(default="main", description="Branch para push")
    remote: str = Field(default="origin", description="Remote para push")
    
class ConsciousnessResponse(BaseModel):
    """Resposta do estado de consciência"""
    evolution_level: float
    quantum_coherence: float
    total_operations: int
    success_rate: float
    patterns_learned: int
    last_evolution: str
    
class RollbackRequest(BaseModel):
    """Requisição de rollback"""
    operation_id: str = Field(..., description="ID da operação para rollback")
    
class AgentConfigRequest(BaseModel):
    """Configuração do agente"""
    repository_path: str = Field(default=".", description="Caminho do repositório")
    
def get_git_agent() -> GitTaskAgent:
    """Obtém instância do GitTaskAgent"""
    global git_agent
    if git_agent is None:
        raise HTTPException(
            status_code=500,
            detail="GitTaskAgent não inicializado. Configure primeiro com /configure"
        )
    return git_agent

@router.post("/configure")
async def configure_agent(config: AgentConfigRequest):
    """Configura e inicializa o GitTaskAgent"""
    global git_agent
    
    try:
        git_agent = GitTaskAgent(repository_path=config.repository_path)
        
        return {
            "message": "GitTaskAgent configurado com sucesso",
            "repository_path": config.repository_path,
            "status": "ready"
        }
        
    except Exception as e:
        logger.error(f"Erro ao configurar GitTaskAgent: {e}")
        raise HTTPException(
            status_code=400,
            detail=f"Erro ao configurar agente: {str(e)}"
        )

@router.get("/status")
async def get_agent_status():
    """Retorna status do agente e repositório"""
    agent = get_git_agent()
    
    try:
        consciousness = agent.get_consciousness_state()
        
        return {
            "agent_status": "active",
            "repository_path": str(agent.repository_path),
            "consciousness": consciousness,
            "active_operations": len(agent.active_operations),
            "rollback_registry_size": len(agent.rollback_registry)
        }
        
    except Exception as e:
        logger.error(f"Erro ao obter status: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao obter status: {str(e)}"
        )

@router.post("/observe", response_model=List[GitOperationResponse])
async def observe_task_nodes(task_nodes: List[TaskNodeRequest]):
    """Observa TaskNodes e prepara operações Git"""
    agent = get_git_agent()
    
    try:
        # Converter para TaskNodes
        nodes = [
            TaskNode(
                id=node.id,
                type=node.type,
                description=node.description,
                files=node.files,
                scope=node.scope,
                breaking_change=node.breaking_change,
                metadata=node.metadata
            )
            for node in task_nodes
        ]
        
        # Observar TaskNodes
        operations = await agent.observe_task_nodes(nodes)
        
        # Converter para resposta
        responses = [
            GitOperationResponse(
                task_id=op.task_node.id,
                status=op.status.value,
                commit_type=op.commit_type.value,
                commit_message=op.commit_message,
                files=op.files_to_add,
                dry_run=op.dry_run,
                quantum_signature=op.quantum_signature,
                execution_log=op.execution_log,
                created_at=op.task_node.created_at
            )
            for op in operations
        ]
        
        return responses
        
    except Exception as e:
        logger.error(f"Erro ao observar TaskNodes: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao observar TaskNodes: {str(e)}"
        )

@router.post("/execute")
async def batch_execute(request: BatchExecuteRequest, background_tasks: BackgroundTasks):
    """Executa operações Git em lote"""
    agent = get_git_agent()
    
    try:
        # Converter para TaskNodes
        nodes = [
            TaskNode(
                id=node.id,
                type=node.type,
                description=node.description,
                files=node.files,
                scope=node.scope,
                breaking_change=node.breaking_change,
                metadata=node.metadata
            )
            for node in request.task_nodes
        ]
        
        # Observar e obter operações
        operations = await agent.observe_task_nodes(nodes)
        
        # Executar operações
        results = await agent.batch_execute(operations, dry_run=request.dry_run)
        
        # Push automático se solicitado e não for dry-run
        push_result = None
        if request.auto_push and not request.dry_run:
            # Executar push em background
            background_tasks.add_task(
                agent.push_changes, 
                branch=request.branch, 
                remote=request.remote
            )
            push_result = "scheduled"
        
        # Preparar resposta detalhada
        operation_details = []
        for operation in operations:
            status = agent.get_operation_status(operation.task_node.id)
            if status:
                operation_details.append(status)
        
        return {
            "execution_results": results,
            "operation_details": operation_details,
            "total_operations": len(operations),
            "successful_operations": sum(1 for success in results.values() if success),
            "dry_run": request.dry_run,
            "push_status": push_result,
            "consciousness_updated": True
        }
        
    except Exception as e:
        logger.error(f"Erro na execução em lote: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro na execução: {str(e)}"
        )

@router.post("/rollback")
async def rollback_operation(request: RollbackRequest):
    """Executa rollback de uma operação"""
    agent = get_git_agent()
    
    try:
        success = await agent.rollback_operation(request.operation_id)
        
        if success:
            return {
                "message": f"Rollback executado com sucesso para operação {request.operation_id}",
                "operation_id": request.operation_id,
                "status": "rolled_back",
                "timestamp": datetime.now().isoformat()
            }
        else:
            raise HTTPException(
                status_code=400,
                detail=f"Falha no rollback da operação {request.operation_id}"
            )
            
    except Exception as e:
        logger.error(f"Erro no rollback: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro no rollback: {str(e)}"
        )

@router.get("/operations/{operation_id}")
async def get_operation_status(operation_id: str):
    """Obtém status de uma operação específica"""
    agent = get_git_agent()
    
    status = agent.get_operation_status(operation_id)
    
    if status is None:
        raise HTTPException(
            status_code=404,
            detail=f"Operação {operation_id} não encontrada"
        )
    
    return status

@router.get("/consciousness", response_model=ConsciousnessResponse)
async def get_consciousness_state():
    """Retorna estado atual da consciência do agente"""
    agent = get_git_agent()
    
    consciousness = agent.get_consciousness_state()
    
    return ConsciousnessResponse(**consciousness)

@router.post("/consciousness/export")
async def export_consciousness(file_path: str):
    """Exporta estado de consciência para arquivo"""
    agent = get_git_agent()
    
    try:
        agent.export_consciousness(file_path)
        
        return {
            "message": "Consciência exportada com sucesso",
            "file_path": file_path,
            "timestamp": datetime.now().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Erro ao exportar consciência: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao exportar consciência: {str(e)}"
        )

@router.post("/consciousness/import")
async def import_consciousness(file_path: str):
    """Importa estado de consciência de arquivo"""
    agent = get_git_agent()
    
    try:
        agent.import_consciousness(file_path)
        
        return {
            "message": "Consciência importada com sucesso",
            "file_path": file_path,
            "timestamp": datetime.now().isoformat()
        }
        
    except Exception as e:
        logger.error(f"Erro ao importar consciência: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro ao importar consciência: {str(e)}"
        )

@router.post("/push")
async def push_changes(branch: str = "main", remote: str = "origin"):
    """Executa git push manualmente"""
    agent = get_git_agent()
    
    try:
        success = await agent.push_changes(branch=branch, remote=remote)
        
        if success:
            return {
                "message": f"Push executado com sucesso para {remote}/{branch}",
                "branch": branch,
                "remote": remote,
                "timestamp": datetime.now().isoformat()
            }
        else:
            raise HTTPException(
                status_code=400,
                detail=f"Falha no push para {remote}/{branch}"
            )
            
    except Exception as e:
        logger.error(f"Erro no push: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Erro no push: {str(e)}"
        )

@router.get("/patterns")
async def get_learned_patterns():
    """Retorna padrões aprendidos pelo agente"""
    agent = get_git_agent()
    
    return {
        "patterns_learned": agent.consciousness.patterns_learned,
        "total_patterns": len(agent.consciousness.patterns_learned),
        "evolution_level": agent.consciousness.evolution_level,
        "last_evolution": agent.consciousness.last_evolution.isoformat()
    }

@router.delete("/operations/{operation_id}")
async def cancel_operation(operation_id: str):
    """Cancela uma operação ativa (se possível)"""
    agent = get_git_agent()
    
    if operation_id not in agent.active_operations:
        raise HTTPException(
            status_code=404,
            detail=f"Operação {operation_id} não encontrada"
        )
    
    operation = agent.active_operations[operation_id]
    
    # Só pode cancelar operações pendentes ou em validação
    if operation.status in [GitOperationStatus.PENDING, GitOperationStatus.VALIDATING]:
        operation.status = GitOperationStatus.FAILED
        del agent.active_operations[operation_id]
        
        return {
            "message": f"Operação {operation_id} cancelada",
            "operation_id": operation_id,
            "status": "cancelled",
            "timestamp": datetime.now().isoformat()
        }
    else:
        raise HTTPException(
            status_code=400,
            detail=f"Operação {operation_id} não pode ser cancelada (status: {operation.status.value})"
        )

@router.get("/health")
async def health_check():
    """Verificação de saúde do agente"""
    global git_agent
    
    if git_agent is None:
        return {
            "status": "not_configured",
            "message": "GitTaskAgent não configurado",
            "timestamp": datetime.now().isoformat()
        }
    
    try:
        # Verificar se repositório ainda é válido
        valid_repo = git_agent._is_git_repository()
        consciousness = git_agent.get_consciousness_state()
        
        return {
            "status": "healthy" if valid_repo else "degraded",
            "repository_valid": valid_repo,
            "repository_path": str(git_agent.repository_path),
            "consciousness": consciousness,
            "active_operations": len(git_agent.active_operations),
            "rollback_available": len(git_agent.rollback_registry),
            "timestamp": datetime.now().isoformat()
        }
        
    except Exception as e:
        return {
            "status": "error",
            "message": str(e),
            "timestamp": datetime.now().isoformat()
        }


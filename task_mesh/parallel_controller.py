"""
ARKITECT Parallel Task Controller
Sistema de controle de execução paralela de tarefas via Task Mesh
"""

import asyncio
from typing import Dict, List, Optional
import logging
from datetime import datetime
import uuid
from dataclasses import dataclass
from enum import Enum

@dataclass
class ParallelTask:
    """Representação de tarefa para execução paralela"""
    id: str
    name: str
    area: str
    priority: int
    estimated_time: int
    dependencies: List[str]
    subtasks: List[str]
    status: str = "pending"
    progress: float = 0.0
    started_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None

class TaskPriority(Enum):
    CRITICAL = 3
    HIGH = 2
    MEDIUM = 1

class ParallelExecutionController:
    """Controlador de execução paralela de tarefas"""
    
    def __init__(self):
        self.logger = logging.getLogger("parallel.controller")
        self.tasks: Dict[str, ParallelTask] = {}
        self.running_tasks: Dict[str, asyncio.Task] = {}
        self.completion_events: Dict[str, asyncio.Event] = {}
        self.max_concurrent = 4
        
    async def initialize_execution(self, task_definitions: Dict):
        """Inicializa execução com definições de tarefas"""
        self.logger.info("Inicializando execução paralela")
        
        # Cria tarefas a partir das definições
        for area, tasks in task_definitions.items():
            for task_def in tasks:
                task_id = str(uuid.uuid4())
                task = ParallelTask(
                    id=task_id,
                    name=task_def["task"],
                    area=area,
                    priority=TaskPriority[task_def.get("priority", "MEDIUM")].value,
                    estimated_time=task_def["estimated_time"],
                    dependencies=task_def.get("dependencies", []),
                    subtasks=task_def.get("subtasks", [])
                )
                self.tasks[task_id] = task
                self.completion_events[task_id] = asyncio.Event()
                
        # Inicia processamento
        await self._process_task_queue()
        
    async def _process_task_queue(self):
        """Processa fila de tarefas respeitando dependências"""
        while self.tasks:
            # Filtra tarefas prontas para execução
            ready_tasks = self._get_ready_tasks()
            
            # Ordena por prioridade
            ready_tasks.sort(key=lambda x: (-x.priority, x.estimated_time))
            
            # Executa tarefas prontas
            executions = []
            for task in ready_tasks[:self.max_concurrent]:
                executions.append(self._execute_task(task))
                
            if executions:
                await asyncio.gather(*executions)
            else:
                await asyncio.sleep(1)
                
    def _get_ready_tasks(self) -> List[ParallelTask]:
        """Retorna tarefas prontas para execução"""
        ready = []
        for task in self.tasks.values():
            if task.status == "pending" and self._dependencies_met(task):
                ready.append(task)
        return ready
        
    def _dependencies_met(self, task: ParallelTask) -> bool:
        """Verifica se dependências foram atendidas"""
        for dep_id in task.dependencies:
            dep_task = self.tasks.get(dep_id)
            if not dep_task or dep_task.status != "completed":
                return False
        return True
        
    async def _execute_task(self, task: ParallelTask):
        """Executa uma tarefa específica"""
        try:
            self.logger.info(f"Iniciando execução da tarefa: {task.name}")
            task.status = "running"
            task.started_at = datetime.now()
            
            # Simula execução das subtasks
            for subtask in task.subtasks:
                self.logger.info(f"Executando subtask: {subtask}")
                progress_increment = 1.0 / len(task.subtasks)
                await asyncio.sleep(task.estimated_time / len(task.subtasks))
                task.progress += progress_increment
                
            task.status = "completed"
            task.completed_at = datetime.now()
            self.completion_events[task.id].set()
            
            self.logger.info(f"Tarefa concluída: {task.name}")
            
        except Exception as e:
            task.status = "failed"
            self.logger.error(f"Erro na execução da tarefa {task.name}: {str(e)}")
            raise
            
    async def get_execution_status(self) -> Dict:
        """Retorna status atual da execução"""
        return {
            "total_tasks": len(self.tasks),
            "completed": len([t for t in self.tasks.values() if t.status == "completed"]),
            "running": len([t for t in self.tasks.values() if t.status == "running"]),
            "pending": len([t for t in self.tasks.values() if t.status == "pending"]),
            "failed": len([t for t in self.tasks.values() if t.status == "failed"]),
            "tasks": {
                task.id: {
                    "name": task.name,
                    "status": task.status,
                    "progress": task.progress,
                    "area": task.area
                } for task in self.tasks.values()
            }
        }


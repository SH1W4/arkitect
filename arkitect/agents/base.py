"""
Base Agent Module - Foundation for All Agents

This module provides the base agent class with core functionality
for task execution, communication, and resource management.
"""

from typing import Dict, List, Any, Optional
from datetime import datetime
import logging
import uuid

logger = logging.getLogger(__name__)


class AgentStatus:
    """Agent status constants."""
    IDLE = "idle"
    BUSY = "busy"
    ERROR = "error"
    OFFLINE = "offline"


class BaseAgent:
    """
    Base Agent class with core functionality.
    
    Provides fundamental capabilities for task execution,
    state management, and interaction with other agents.
    """
    
    def __init__(
        self,
        id: Optional[str] = None,
        task_scheduler=None,
        agent_coordinator=None
    ):
        """
        Initialize the Base Agent.
        
        Args:
            id: Unique agent identifier (auto-generated if not provided)
            task_scheduler: Optional TaskScheduler instance
            agent_coordinator: Optional AgentCoordinator instance
        """
        self.id = id or f"agent_{uuid.uuid4().hex[:8]}"
        self.task_scheduler = task_scheduler
        self.agent_coordinator = agent_coordinator
        self.status = AgentStatus.IDLE
        self.created_at = datetime.utcnow()
        self.tasks_completed = 0
        self.tasks_failed = 0
        self.capabilities: List[str] = []
        self.memory: Dict[str, Any] = {}
        
        # Register with coordinator if provided
        if self.agent_coordinator:
            self.agent_coordinator.register_agent(self.id)
        
        logger.info(f"BaseAgent {self.id} initialized")
    
    def execute_task(self, task: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute a task.
        
        Args:
            task: Task dictionary with 'id', 'type', and 'parameters'
            
        Returns:
            Task execution result
        """
        self.status = AgentStatus.BUSY
        task_id = task.get('id', 'unknown')
        task_type = task.get('type', 'generic')
        
        logger.info(f"Agent {self.id} executing task {task_id}")
        
        try:
            # Process task based on type
            result = self._process_task(task)
            
            self.tasks_completed += 1
            self.status = AgentStatus.IDLE
            
            return {
                "success": True,
                "task_id": task_id,
                "agent_id": self.id,
                "result": result,
                "completed_at": datetime.utcnow().isoformat()
            }
        
        except Exception as e:
            self.tasks_failed += 1
            self.status = AgentStatus.ERROR
            logger.error(f"Task {task_id} failed: {e}")
            
            return {
                "success": False,
                "task_id": task_id,
                "agent_id": self.id,
                "error": str(e),
                "failed_at": datetime.utcnow().isoformat()
            }
    
    def _process_task(self, task: Dict[str, Any]) -> Any:
        """
        Process task logic (to be overridden by subclasses).
        
        Args:
            task: Task to process
            
        Returns:
            Processing result
        """
        # Default implementation
        task_type = task.get('type', 'generic')
        parameters = task.get('parameters', {})
        
        # Use quantum core if available
        if self.quantum_core and 'data' in parameters:
            return self.quantum_core.process(parameters['data'])
        
        return {"processed": True, "type": task_type}
    
    def communicate(self, target_agent_id: str, message: Dict[str, Any]) -> bool:
        """
        Send a message to another agent.
        
        Args:
            target_agent_id: ID of the target agent
            message: Message dictionary
            
        Returns:
            True if message sent successfully
        """
        if not self.symbiotic_engine:
            logger.warning("No symbiotic engine available for communication")
            return False
        
        # Check if connection exists
        connections = self.symbiotic_engine.get_connections(self.id)
        connected_agents = [c['partner'] for c in connections]
        
        if target_agent_id not in connected_agents:
            logger.warning(f"No connection to agent {target_agent_id}")
            return False
        
        logger.info(f"Agent {self.id} sent message to {target_agent_id}")
        return True
    
    def add_capability(self, capability: str) -> None:
        """
        Add a capability to the agent.
        
        Args:
            capability: Capability name
        """
        if capability not in self.capabilities:
            self.capabilities.append(capability)
            logger.debug(f"Agent {self.id} gained capability: {capability}")
    
    def has_capability(self, capability: str) -> bool:
        """
        Check if agent has a specific capability.
        
        Args:
            capability: Capability to check
            
        Returns:
            True if agent has the capability
        """
        return capability in self.capabilities
    
    def store_memory(self, key: str, value: Any) -> None:
        """
        Store information in agent memory.
        
        Args:
            key: Memory key
            value: Value to store
        """
        self.memory[key] = {
            "value": value,
            "stored_at": datetime.utcnow().isoformat()
        }
    
    def recall_memory(self, key: str) -> Optional[Any]:
        """
        Recall information from memory.
        
        Args:
            key: Memory key
            
        Returns:
            Stored value or None if not found
        """
        memory_item = self.memory.get(key)
        return memory_item['value'] if memory_item else None
    
    def get_stats(self) -> Dict[str, Any]:
        """
        Get agent statistics.
        
        Returns:
            Dictionary of agent stats
        """
        total_tasks = self.tasks_completed + self.tasks_failed
        success_rate = (
            self.tasks_completed / total_tasks if total_tasks > 0 else 0.0
        )
        
        return {
            "id": self.id,
            "status": self.status,
            "tasks_completed": self.tasks_completed,
            "tasks_failed": self.tasks_failed,
            "success_rate": success_rate,
            "capabilities": self.capabilities,
            "memory_items": len(self.memory),
            "uptime_seconds": (datetime.utcnow() - self.created_at).total_seconds()
        }


from mcp.server.fastmcp import FastMCP
from arkitect.orchestrator.main import tasks_store, generate_task_id, collect_metrics
import asyncio
from datetime import datetime
from typing import Optional, List, Dict, Any

# Initialize FastMCP server
mcp = FastMCP("Arkitect Orchestrator")

@mcp.tool()
async def list_tasks(status: Optional[str] = None, priority: Optional[str] = None) -> List[Dict[str, Any]]:
    """
    List tasks from the orchestrator, optionally filtered by status or priority.
    
    Args:
        status: Filter by task status (pending, running, completed, failed)
        priority: Filter by task priority (low, medium, high, critical)
    """
    tasks = list(tasks_store.values())
    
    if status:
        tasks = [t for t in tasks if t.get("status") == status]
    
    if priority:
        tasks = [t for t in tasks if t.get("priority") == priority]
        
    return tasks

@mcp.tool()
async def create_task(name: str, description: Optional[str] = None, priority: str = "medium", layer: str = "default") -> Dict[str, Any]:
    """
    Create a new task in the Arkitect orchestrator.
    
    Args:
        name: Name of the task
        description: Description of the task
        priority: Priority level (low, medium, high, critical)
        layer: Execution layer (default, system, user, admin)
    """
    task_id = generate_task_id()
    now = datetime.utcnow()
    
    task = {
        "id": task_id,
        "name": name,
        "description": description,
        "status": "pending",
        "priority": priority,
        "layer": layer,
        "parameters": {},
        "created_at": now,
        "updated_at": now,
        "completed_at": None,
        "result": None,
        "error": None
    }
    
    tasks_store[task_id] = task
    return task

@mcp.tool()
async def get_metrics() -> Dict[str, Any]:
    """
    Get current system metrics from the orchestrator.
    """
    return await collect_metrics()

@mcp.tool()
async def get_task_details(task_id: str) -> Dict[str, Any]:
    """
    Get details of a specific task by ID.
    
    Args:
        task_id: The ID of the task to retrieve
    """
    task = tasks_store.get(task_id)
    if not task:
        raise ValueError(f"Task with ID {task_id} not found")
    return task

def main():
    """Main entry point for the MCP server."""
    mcp.run()

if __name__ == "__main__":
    main()

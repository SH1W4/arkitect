# Arkitect MCP Server

This module provides a Model Context Protocol (MCP) server for the Arkitect platform, allowing AI agents to interact with the orchestrator.

## Features

- **List Tasks**: View active tasks with filtering.
- **Create Task**: Submit new tasks to the orchestrator.
- **Get Metrics**: Retrieve system performance metrics.
- **Get Task Details**: View detailed information about a specific task.

## Installation

Ensure you have installed the project dependencies:

```bash
pip install -e .
```

## Running the Server

You can run the MCP server using the `mcp` CLI or directly via Python:

```bash
# Using Python directly
python -m arkitect.mcp.server
```

## Tools Available

- `list_tasks(status=None, priority=None)`
- `create_task(name, description=None, priority="medium", layer="default")`
- `get_metrics()`
- `get_task_details(task_id)`

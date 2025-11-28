# MCP Integration Guide

## Quick Start

### Install Arkitect
```bash
pip install -e .
```

### Start MCP Server
```bash
# Option 1: Using the entry point (after pip install)
arkitect-mcp

# Option 2: Using Python module
python -m arkitect.mcp.server

# Option 3: Direct execution
cd arkitect/mcp
python server.py
```

## IDE/Agent Configuration

### Claude Desktop / Cline / Other MCP Clients

Add to your MCP configuration file (usually `~/AppData/Roaming/Claude/claude_desktop_config.json` on Windows or `~/.config/claude/claude_desktop_config.json` on macOS/Linux):

```json
{
  "mcpServers": {
    "arkitect": {
      "command": "python",
      "args": ["-m", "arkitect.mcp.server"],
      "env": {},
      "description": "Arkitect Task Orchestration Server"
    }
  }
}
```

Or use the entry point directly (preferred after `pip install -e .`):

```json
{
  "mcpServers": {
    "arkitect": {
      "command": "arkitect-mcp",
      "args": [],
      "env": {},
      "description": "Arkitect Task Orchestration Server"
    }
  }
}
```

### VSCode with Continue or Other Extensions

Add to your Continue config (`~/.continue/config.json`):

```json
{
  "mcpServers": [
    {
      "name": "arkitect",
      "command": "python",
      "args": ["-m", "arkitect.mcp.server"]
    }
  ]
}
```

### Cursor IDE

Add to Cursor's MCP settings:

```json
{
  "mcpServers": {
    "arkitect": {
      "command": "arkitect-mcp"
    }
  }
}
```

## Available Tools

Once connected, AI agents can use these tools:

### 1. list_tasks
List and filter tasks in the orchestrator.

**Parameters**:
- `status` (optional): Filter by status (pending, running, completed, failed)
- `priority` (optional): Filter by priority (critical, high, medium, low)

**Example**:
```
List all high-priority tasks
```

### 2. create_task
Create a new task in the orchestrator.

**Parameters**:
- `name` (required): Task name
- `description` (optional): Task description
- `priority` (optional): Priority level (default: medium)
- `layer` (optional): Execution layer (default: default)

**Example**:
```
Create a high-priority task to analyze the codebase
```

### 3. get_metrics
Get current system performance metrics.

**Example**:
```
Show me the current system metrics
```

### 4. get_task_details
Get detailed information about a specific task.

**Parameters**:
- `task_id` (required): Task identifier

**Example**:
```
Show details for task 12345
```

## Troubleshooting

### Server Not Starting

1. **Check Python version**: Requires Python 3.9+
   ```bash
   python --version
   ```

2. **Verify installation**:
   ```bash
   pip list | grep arkitect
   ```

3. **Test manually**:
   ```bash
   python -m arkitect.mcp.server
   ```

### Connection Issues

1. **Check the command path** in your IDE configuration
2. **Verify environment variables** are set correctly
3. **Check server logs** for errors

### Common Errors

**"Module not found: arkitect"**
- Run `pip install -e .` in the Arkitect directory

**"Command 'arkitect-mcp' not found"**
- Reinstall: `pip install -e .`
- Or use full path: `python -m arkitect.mcp.server`

## Testing the Connection

After configuring your IDE/agent:

1. Restart your IDE or AI agent client
2. Ask the agent: "List all tasks using Arkitect"
3. Try creating a task: "Create a test task in Arkitect"

## Environment Variables

Optional environment variables for the MCP server:

```env
# Logging level
LOG_LEVEL=INFO

# Server port (if running standalone API)
API_PORT=8000

# Redis connection (optional)
REDIS_URL=redis://localhost:6379
```

## Advanced Usage

### Running with Custom Configuration

```bash
# With environment file
env $(cat .env | xargs) arkitect-mcp

# With specific Python interpreter
/path/to/python -m arkitect.mcp.server
```

### Debugging

Enable debug logging:

```bash
LOG_LEVEL=DEBUG python -m arkitect.mcp.server
```

## Integration Examples

### Claude Desktop

After configuration, you can ask Claude:
- "List all pending tasks in Arkitect"
- "Create a high-priority task to review the authentication module"
- "Show me the current system metrics"
- "Get details for task abc123"

### Cursor / VSCode

The Arkitect tools will appear in your AI assistant's tool list. Simply reference them in natural language.

### Custom Scripts

You can also use the MCP server programmatically:

```python
from arkitect.mcp.server import list_tasks, create_task, get_metrics

# List tasks
tasks = await list_tasks(status="pending")

# Create a task
task = await create_task(
    name="Code Review",
    description="Review authentication module",
    priority="high"
)

# Get metrics
metrics = await get_metrics()
```

## Support

For issues or questions:
- GitHub Issues: https://github.com/SH1W4/arkitect/issues
- Documentation: https://github.com/SH1W4/arkitect#readme

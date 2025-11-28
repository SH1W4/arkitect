# CHANGELOG

## [0.2.0] - 2025-11-28

### Added
- **MCP Server Integration**: Implemented Model Context Protocol server in `arkitect/mcp/server.py`
  - `list_tasks`: List and filter orchestrator tasks
  - `create_task`: Create new tasks programmatically
  - `get_metrics`: Retrieve system performance metrics
  - `get_task_details`: Get detailed task information
- **Consolidated Project Structure**: Merged `orchestrator_api` into `arkitect/orchestrator`
- **Core Module Stubs**: Created stub implementations for `core`, `agents`, and `api` modules
- **Documentation**: Added `README_MCP.md` with MCP server usage instructions

### Changed
- **Restructured Python Package**: Moved orchestrator logic from standalone `orchestrator_api` to `arkitect.orchestrator`
- **Updated Dependencies**: Added `mcp`, `grpcio`, and `psutil` to `pyproject.toml`

### Removed
- **Redundant Backend**: Removed `arkitect_backend` directory (functionality consolidated into main package)

### Fixed
- Import errors by creating missing module files
- Package structure for proper Python imports

## [0.1.0] - Initial Release
- Initial project structure
- Rust core with quantum processing simulation
- FastAPI orchestrator
- Task mesh architecture

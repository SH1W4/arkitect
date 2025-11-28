"""ARKITECT - High-Performance Agent Orchestration Platform.

ARKITECT is a professional platform for task orchestration and agent coordination,
designed for developers who need reliable, scalable automation.

Key Features:
- Advanced task scheduling with multiple strategies
- Multi-agent coordination and collaboration
- Comprehensive metrics and monitoring
- MCP (Model Context Protocol) integration
- Production-ready architecture
"""

from importlib.metadata import version, PackageNotFoundError

try:
    __version__ = version("arkitect")
except PackageNotFoundError:
    __version__ = "0.2.0"

__author__ = "SH1W4"
__email__ = "contact@arkitect.dev"
__description__ = "High-Performance Agent Orchestration Platform"

# Expose main components
from arkitect.core import (
    TaskScheduler,
    SchedulingStrategy,
    AgentCoordinator,
    CoordinationType,
    MetricsCollector,
)

from arkitect.agents import (
    BaseAgent,
    EvolutionaryAgent,
    MetaGovernanceAgent,
)

from arkitect.api import (
    APIServer,
    create_app,
)

# Configure logging
import logging
import structlog

structlog.configure(
    processors=[
        structlog.stdlib.filter_by_level,
        structlog.stdlib.add_logger_name,
        structlog.stdlib.add_log_level,
        structlog.stdlib.PositionalArgumentsFormatter(),
        structlog.processors.TimeStamper(fmt="iso"),
        structlog.processors.StackInfoRenderer(),
        structlog.processors.format_exc_info,
        structlog.processors.UnicodeDecoder(),
        structlog.processors.JSONRenderer()
    ],
    context_class=dict,
    logger_factory=structlog.stdlib.LoggerFactory(),
    wrapper_class=structlog.stdlib.BoundLogger,
    cache_logger_on_first_use=True,
)

logger = structlog.get_logger(__name__)

__all__ = [
    "__version__",
    "__author__", 
    "__email__",
    "__description__",
    "TaskScheduler",
    "SchedulingStrategy",
    "AgentCoordinator",
    "CoordinationType",
    "MetricsCollector",
    "BaseAgent",
    "EvolutionaryAgent", 
    "MetaGovernanceAgent",
    "APIServer",
    "create_app",
    "logger",
]


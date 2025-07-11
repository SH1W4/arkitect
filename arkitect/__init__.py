"""ARKITECT - Plataforma Simbiótica de Meta-Governança para Agentes IA.

ARKITECT é uma plataforma avançada que combina componentes Rust de alta performance
com a flexibilidade do Python para criar um ecossistema simbiótico de agentes IA
com capacidades de meta-governança e evolução autônoma.

Características principais:
- Arquitetura híbrida Rust+Python com bindings via maturin
- Sistema de agentes autônomos com capacidades evolutivas
- Meta-governança adaptativa e auto-organizante
- Monitoramento e observabilidade integrados
- Escalabilidade horizontal e vertical
"""

from importlib.metadata import version, PackageNotFoundError

try:
    __version__ = version("arkitect")
except PackageNotFoundError:
    __version__ = "0.1.0-dev"

__author__ = "EON Framework Team"
__email__ = "contact@eonframework.dev"
__description__ = "Plataforma Simbiótica de Meta-Governança para Agentes IA"

# Expor principais componentes
from arkitect.core import (
    QuantumCore,
    SymbioticEngine,
    ConsciousnessLayer,
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

# Configurar logging padrão
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

# Verificar disponibilidade dos componentes Rust
try:
    from arkitect._core import (
        quantum_bridge,
        symbiotic_processor,
        consciousness_matrix,
    )
    RUST_AVAILABLE = True
    logger.info("Componentes Rust carregados com sucesso", version=__version__)
except ImportError as e:
    RUST_AVAILABLE = False
    logger.warning(
        "Componentes Rust não disponíveis", 
        error=str(e),
        fallback="usando implementação Python pura"
    )

__all__ = [
    "__version__",
    "__author__", 
    "__email__",
    "__description__",
    "QuantumCore",
    "SymbioticEngine",
    "ConsciousnessLayer",
    "BaseAgent",
    "EvolutionaryAgent", 
    "MetaGovernanceAgent",
    "APIServer",
    "create_app",
    "RUST_AVAILABLE",
    "logger",
]


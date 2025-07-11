"""Módulo Agents do ARKITECT.

Agent system with evolutionary and meta-governance capabilities.
"""

from arkitect.agents.base import BaseAgent
from arkitect.agents.evolutionary import EvolutionaryAgent
from arkitect.agents.governance import MetaGovernanceAgent

__all__ = [
    "BaseAgent",
    "EvolutionaryAgent",
    "MetaGovernanceAgent",
]


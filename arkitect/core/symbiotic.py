"""
Symbiotic Engine Module - Agent Collaboration System

This module manages symbiotic relationships between agents,
enabling collaborative task execution and resource sharing.
"""

from typing import Dict, List, Set, Optional, Any
from enum import Enum
import logging
from datetime import datetime

logger = logging.getLogger(__name__)


class SymbiosisType(Enum):
    """Types of symbiotic relationships."""
    MUTUALISM = "mutualism"  # Both benefit
    COMMENSALISM = "commensalism"  # One benefits, other unaffected
    PARASITISM = "parasitism"  # One benefits at expense of other
    COMPETITION = "competition"  # Compete for resources


class SymbioticRelationship:
    """Represents a symbiotic relationship between two agents."""
    
    def __init__(
        self,
        agent_a: str,
        agent_b: str,
        relationship_type: SymbiosisType,
        strength: float = 0.5
    ):
        self.agent_a = agent_a
        self.agent_b = agent_b
        self.relationship_type = relationship_type
        self.strength = min(max(strength, 0.0), 1.0)  # Clamp to [0, 1]
        self.created_at = datetime.utcnow()
        self.interactions = 0
    
    def evolve(self, outcome: float) -> None:
        """
        Evolve relationship based on interaction outcome.
        
        Args:
            outcome: Interaction outcome (-1.0 to 1.0)
        """
        self.interactions += 1
        # Adjust strength based on outcome
        delta = outcome * 0.1
        self.strength = min(max(self.strength + delta, 0.0), 1.0)


class SymbioticEngine:
    """
    Symbiotic Engine for managing agent collaboration.
    
    Handles relationship formation, evolution, and resource
    sharing between agents in the ecosystem.
    """
    
    def __init__(self):
        """Initialize the Symbiotic Engine."""
        self.agents: Set[str] = set()
        self.relationships: Dict[tuple, SymbioticRelationship] = {}
        self.resource_pool: Dict[str, float] = {}
        logger.info("SymbioticEngine initialized")
    
    def register_agent(self, agent_id: str) -> bool:
        """
        Register a new agent in the ecosystem.
        
        Args:
            agent_id: Unique identifier for the agent
            
        Returns:
            True if registration successful
        """
        if agent_id in self.agents:
            logger.warning(f"Agent {agent_id} already registered")
            return False
        
        self.agents.add(agent_id)
        self.resource_pool[agent_id] = 1.0  # Initial resource allocation
        logger.info(f"Agent {agent_id} registered")
        return True
    
    def establish_connection(
        self,
        agent_a: str,
        agent_b: str,
        relationship_type: SymbiosisType = SymbiosisType.MUTUALISM
    ) -> bool:
        """
        Establish a symbiotic connection between two agents.
        
        Args:
            agent_a: First agent ID
            agent_b: Second agent ID
            relationship_type: Type of symbiotic relationship
            
        Returns:
            True if connection established
        """
        # Ensure both agents are registered
        if agent_a not in self.agents:
            self.register_agent(agent_a)
        if agent_b not in self.agents:
            self.register_agent(agent_b)
        
        # Create relationship key (sorted for consistency)
        key = tuple(sorted([agent_a, agent_b]))
        
        if key in self.relationships:
            logger.warning(f"Relationship between {agent_a} and {agent_b} already exists")
            return False
        
        # Create new relationship
        relationship = SymbioticRelationship(
            agent_a, agent_b, relationship_type
        )
        self.relationships[key] = relationship
        
        logger.info(
            f"Established {relationship_type.value} relationship "
            f"between {agent_a} and {agent_b}"
        )
        return True
    
    def get_connections(self, agent_id: str) -> List[Dict[str, Any]]:
        """
        Get all symbiotic connections for an agent.
        
        Args:
            agent_id: Agent identifier
            
        Returns:
            List of connection details
        """
        connections = []
        
        for key, relationship in self.relationships.items():
            if agent_id in key:
                partner = key[0] if key[1] == agent_id else key[1]
                connections.append({
                    "partner": partner,
                    "type": relationship.relationship_type.value,
                    "strength": relationship.strength,
                    "interactions": relationship.interactions
                })
        
        return connections
    
    def share_resources(
        self,
        donor: str,
        recipient: str,
        amount: float
    ) -> bool:
        """
        Share resources between connected agents.
        
        Args:
            donor: Agent giving resources
            recipient: Agent receiving resources
            amount: Amount to transfer
            
        Returns:
            True if transfer successful
        """
        # Check if agents are connected
        key = tuple(sorted([donor, recipient]))
        if key not in self.relationships:
            logger.error(f"No relationship between {donor} and {recipient}")
            return False
        
        # Check donor has sufficient resources
        if self.resource_pool.get(donor, 0) < amount:
            logger.error(f"Insufficient resources for {donor}")
            return False
        
        # Transfer resources
        self.resource_pool[donor] -= amount
        self.resource_pool[recipient] = self.resource_pool.get(recipient, 0) + amount
        
        # Evolve relationship positively
        self.relationships[key].evolve(0.5)
        
        logger.info(f"Transferred {amount} resources from {donor} to {recipient}")
        return True
    
    def calculate_network_health(self) -> float:
        """
        Calculate overall health of the symbiotic network.
        
        Returns:
            Health score (0.0 to 1.0)
        """
        if not self.relationships:
            return 0.0
        
        # Average relationship strength
        total_strength = sum(r.strength for r in self.relationships.values())
        avg_strength = total_strength / len(self.relationships)
        
        # Factor in resource distribution
        if self.resource_pool:
            resource_variance = np.var(list(self.resource_pool.values()))
            # Lower variance = better distribution
            distribution_score = 1.0 / (1.0 + resource_variance)
        else:
            distribution_score = 0.0
        
        # Combined health score
        health = (avg_strength * 0.7) + (distribution_score * 0.3)
        
        return health
    
    def get_ecosystem_stats(self) -> Dict[str, Any]:
        """
        Get statistics about the symbiotic ecosystem.
        
        Returns:
            Dictionary of ecosystem statistics
        """
        relationship_types = {}
        for rel in self.relationships.values():
            rel_type = rel.relationship_type.value
            relationship_types[rel_type] = relationship_types.get(rel_type, 0) + 1
        
        return {
            "total_agents": len(self.agents),
            "total_relationships": len(self.relationships),
            "relationship_types": relationship_types,
            "network_health": self.calculate_network_health(),
            "total_resources": sum(self.resource_pool.values())
        }


# Import numpy for variance calculation
import numpy as np

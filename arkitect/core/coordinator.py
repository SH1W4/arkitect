"""
Agent Coordinator Module - Multi-Agent Collaboration

This module manages collaboration and coordination between agents,
enabling efficient resource sharing and distributed task execution.
"""

from typing import Dict, List, Set, Optional, Any
from enum import Enum
import logging
from datetime import datetime

logger = logging.getLogger(__name__)


class CoordinationType(Enum):
    """Types of agent coordination."""
    COLLABORATIVE = "collaborative"  # Agents work together
    INDEPENDENT = "independent"  # Agents work separately  
    DELEGATED = "delegated"  # One agent delegates to others
    COMPETITIVE = "competitive"  # Agents compete


class AgentConnection:
    """Represents a connection between two agents."""
    
    def __init__(
        self,
        agent_a: str,
        agent_b: str,
        coordination_type: CoordinationType,
        trust_score: float = 0.5
    ):
        self.agent_a = agent_a
        self.agent_b = agent_b
        self.coordination_type = coordination_type
        self.trust_score = min(max(trust_score, 0.0), 1.0)  # Clamp to [0, 1]
        self.created_at = datetime.utcnow()
        self.interactions = 0
    
    def update_trust(self, outcome: float) -> None:
        """
        Update trust score based on interaction outcome.
        
        Args:
            outcome: Interaction outcome (-1.0 to 1.0)
        """
        self.interactions += 1
        # Adjust trust based on outcome
        delta = outcome * 0.1
        self.trust_score = min(max(self.trust_score + delta, 0.0), 1.0)


class AgentCoordinator:
    """
    Agent Coordinator for managing multi-agent collaboration.
    
    Handles connection formation, resource sharing, and
    coordination between agents in the system.
    """
    
    def __init__(self):
        """Initialize the Agent Coordinator."""
        self.agents: Set[str] = set()
        self.connections: Dict[tuple, AgentConnection] = {}
        self.resource_pool: Dict[str, float] = {}
        logger.info("AgentCoordinator initialized")
    
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
    
    def connect_agents(
        self,
        agent_a: str,
        agent_b: str,
        coordination_type: CoordinationType = CoordinationType.COLLABORATIVE
    ) -> bool:
        """
        Connect two agents for coordination.
        
        Args:
            agent_a: First agent ID
            agent_b: Second agent ID
            coordination_type: Type of coordination
            
        Returns:
            True if connection established
        """
        # Ensure both agents are registered
        if agent_a not in self.agents:
            self.register_agent(agent_a)
        if agent_b not in self.agents:
            self.register_agent(agent_b)
        
        # Create connection key (sorted for consistency)
        key = tuple(sorted([agent_a, agent_b]))
        
        if key in self.connections:
            logger.warning(f"Connection between {agent_a} and {agent_b} already exists")
            return False
        
        # Create new connection
        connection = AgentConnection(
            agent_a, agent_b, coordination_type
        )
        self.connections[key] = connection
        
        logger.info(
            f"Established {coordination_type.value} connection "
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

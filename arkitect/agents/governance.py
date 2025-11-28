"""
Meta-Governance Agent Module - System Governance

This module provides an agent specialized in meta-governance,
coordinating other agents and managing system-level decisions.
"""

from .base import BaseAgent
from typing import Dict, List, Any, Optional
import logging

logger = logging.getLogger(__name__)


class GovernancePolicy:
    """Represents a governance policy."""
    
    def __init__(self, name: str, rules: Dict[str, Any]):
        self.name = name
        self.rules = rules
        self.enforcement_count = 0
    
    def enforce(self) -> bool:
        """Enforce the policy."""
        self.enforcement_count += 1
        return True


class MetaGovernanceAgent(BaseAgent):
    """
    Agent specialized in meta-governance and coordination.
    
    Manages policies, coordinates other agents, and makes
    system-level decisions for optimal ecosystem health.
    """
    
    def __init__(
        self,
        id: Optional[str] = None,
        quantum_core=None,
        symbiotic_engine=None
    ):
        """
        Initialize the Meta-Governance Agent.
        
        Args:
            id: Unique agent identifier
            quantum_core: Optional QuantumCore instance
            symbiotic_engine: Optional SymbioticEngine instance
        """
        super().__init__(id, quantum_core, symbiotic_engine)
        self.policies: Dict[str, GovernancePolicy] = {}
        self.managed_agents: List[str] = []
        self.decisions_made = 0
        self.consensus_threshold = 0.6
        
        # Add governance capabilities
        self.add_capability("governance")
        self.add_capability("coordination")
        self.add_capability("policy_enforcement")
        self.add_capability("consensus_building")
        
        # Initialize default policies
        self._initialize_default_policies()
        
        logger.info(f"MetaGovernanceAgent {self.id} initialized")
    
    def _initialize_default_policies(self) -> None:
        """Initialize default governance policies."""
        self.add_policy("resource_fairness", {
            "type": "resource_allocation",
            "min_allocation": 0.1,
            "max_allocation": 0.5
        })
        
        self.add_policy("task_priority", {
            "type": "task_management",
            "priority_levels": ["critical", "high", "medium", "low"]
        })
    
    def add_policy(self, name: str, rules: Dict[str, Any]) -> bool:
        """
        Add a new governance policy.
        
        Args:
            name: Policy name
            rules: Policy rules dictionary
            
        Returns:
            True if policy added successfully
        """
        if name in self.policies:
            logger.warning(f"Policy {name} already exists")
            return False
        
        policy = GovernancePolicy(name, rules)
        self.policies[name] = policy
        logger.info(f"Agent {self.id} added policy: {name}")
        return True
    
    def enforce_policy(self, policy_name: str, context: Dict[str, Any]) -> bool:
        """
        Enforce a governance policy.
        
        Args:
            policy_name: Name of policy to enforce
            context: Context for policy enforcement
            
        Returns:
            True if policy enforced successfully
        """
        if policy_name not in self.policies:
            logger.error(f"Policy {policy_name} not found")
            return False
        
        policy = self.policies[policy_name]
        result = policy.enforce()
        
        logger.info(f"Agent {self.id} enforced policy: {policy_name}")
        return result
    
    def coordinate_agents(self, agent_ids: List[str], task: Dict[str, Any]) -> Dict[str, Any]:
        """
        Coordinate multiple agents for a task.
        
        Args:
            agent_ids: List of agent IDs to coordinate
            task: Task to coordinate
            
        Returns:
            Coordination result
        """
        if not self.symbiotic_engine:
            logger.warning("No symbiotic engine for coordination")
            return {"success": False, "error": "No symbiotic engine"}
        
        # Ensure all agents are registered
        for agent_id in agent_ids:
            if agent_id not in self.managed_agents:
                self.managed_agents.append(agent_id)
        
        # Create coordination plan
        plan = {
            "coordinator": self.id,
            "agents": agent_ids,
            "task": task,
            "strategy": "parallel" if len(agent_ids) > 1 else "single"
        }
        
        logger.info(
            f"Agent {self.id} coordinating {len(agent_ids)} agents for task"
        )
        
        return {
            "success": True,
            "plan": plan,
            "managed_agents": len(self.managed_agents)
        }
    
    def make_decision(
        self,
        decision_type: str,
        options: List[Dict[str, Any]],
        criteria: Dict[str, float]
    ) -> Dict[str, Any]:
        """
        Make a governance decision.
        
        Args:
            decision_type: Type of decision
            options: Available options
            criteria: Decision criteria with weights
            
        Returns:
            Decision result
        """
        self.decisions_made += 1
        
        # Score each option based on criteria
        scores = []
        for option in options:
            score = 0.0
            for criterion, weight in criteria.items():
                option_value = option.get(criterion, 0.0)
                score += option_value * weight
            scores.append(score)
        
        # Select best option
        best_index = scores.index(max(scores))
        best_option = options[best_index]
        
        decision = {
            "type": decision_type,
            "selected_option": best_option,
            "score": scores[best_index],
            "decision_number": self.decisions_made
        }
        
        logger.info(f"Agent {self.id} made decision: {decision_type}")
        
        # Store decision in memory
        self.store_memory(f"decision_{self.decisions_made}", decision)
        
        return decision
    
    def build_consensus(
        self,
        agent_votes: Dict[str, bool]
    ) -> Dict[str, Any]:
        """
        Build consensus from agent votes.
        
        Args:
            agent_votes: Dictionary of agent_id -> vote (True/False)
            
        Returns:
            Consensus result
        """
        if not agent_votes:
            return {"consensus": False, "reason": "no_votes"}
        
        total_votes = len(agent_votes)
        positive_votes = sum(1 for vote in agent_votes.values() if vote)
        consensus_ratio = positive_votes / total_votes
        
        consensus_reached = consensus_ratio >= self.consensus_threshold
        
        result = {
            "consensus": consensus_reached,
            "ratio": consensus_ratio,
            "threshold": self.consensus_threshold,
            "total_votes": total_votes,
            "positive_votes": positive_votes
        }
        
        logger.info(
            f"Consensus {'reached' if consensus_reached else 'not reached'}: "
            f"{consensus_ratio:.2%}"
        )
        
        return result
    
    def get_governance_stats(self) -> Dict[str, Any]:
        """
        Get governance statistics.
        
        Returns:
            Dictionary of governance stats
        """
        return {
            "total_policies": len(self.policies),
            "managed_agents": len(self.managed_agents),
            "decisions_made": self.decisions_made,
            "consensus_threshold": self.consensus_threshold,
            "active_policies": list(self.policies.keys())
        }

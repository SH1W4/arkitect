"""
Evolutionary Agent Module - Self-Improving Agent

This module provides an agent with evolutionary capabilities,
allowing it to adapt and improve through experience.
"""

from .base import BaseAgent
from typing import Dict, List, Any, Optional
import logging
import random

logger = logging.getLogger(__name__)


class EvolutionaryAgent(BaseAgent):
    """
    Agent with evolutionary and adaptive capabilities.
    
    Can learn from experience, mutate strategies, and
    evolve to become more effective over time.
    """
    
    def __init__(
        self,
        id: Optional[str] = None,
        task_scheduler=None,
        agent_coordinator=None,
        mutation_rate: float = 0.1
    ):
        """
        Initialize the Evolutionary Agent.
        
        Args:
            id: Unique agent identifier
            task_scheduler: Optional TaskScheduler instance
            agent_coordinator: Optional AgentCoordinator instance
            mutation_rate: Probability of strategy mutation (0.0 to 1.0)
        """
        super().__init__(id, task_scheduler, agent_coordinator)
        self.mutation_rate = min(max(mutation_rate, 0.0), 1.0)
        self.generation = 1
        self.fitness_score = 0.0
        self.strategies: List[str] = ["default"]
        self.current_strategy = "default"
        self.strategy_performance: Dict[str, List[float]] = {"default": []}
        
        # Add evolutionary capabilities
        self.add_capability("evolution")
        self.add_capability("adaptation")
        self.add_capability("learning")
        
        logger.info(f"EvolutionaryAgent {self.id} initialized (gen {self.generation})")
    
    def execute_task(self, task: Dict[str, Any]) -> Dict[str, Any]:
        """
        Execute task with evolutionary learning.
        
        Args:
            task: Task to execute
            
        Returns:
            Task execution result with fitness update
        """
        # Execute using parent method
        result = super().execute_task(task)
        
        # Update fitness based on result
        if result['success']:
            self.fitness_score += 10.0
            self.strategy_performance[self.current_strategy].append(1.0)
        else:
            self.fitness_score -= 5.0
            self.strategy_performance[self.current_strategy].append(0.0)
        
        # Check for evolution opportunity
        if self._should_evolve():
            self.evolve()
        
        # Add evolutionary metadata to result
        result['evolution_metadata'] = {
            "generation": self.generation,
            "fitness": self.fitness_score,
            "strategy": self.current_strategy
        }
        
        return result
    
    def _should_evolve(self) -> bool:
        """Check if agent should evolve."""
        # Evolve after every 10 tasks
        total_tasks = self.tasks_completed + self.tasks_failed
        return total_tasks > 0 and total_tasks % 10 == 0
    
    def evolve(self) -> None:
        """
        Evolve the agent to the next generation.
        
        Evaluates strategies, potentially mutates, and advances generation.
        """
        old_generation = self.generation
        self.generation += 1
        
        # Evaluate current strategy
        self._evaluate_strategies()
        
        # Potentially mutate
        if random.random() < self.mutation_rate:
            self._mutate()
        
        logger.info(
            f"Agent {self.id} evolved from gen {old_generation} to {self.generation}"
        )
        
        # Store evolution event in memory
        self.store_memory(f"evolution_{self.generation}", {
            "from_generation": old_generation,
            "fitness": self.fitness_score,
            "strategies": self.strategies.copy()
        })
    
    def _evaluate_strategies(self) -> None:
        """Evaluate and select best performing strategy."""
        if len(self.strategies) == 1:
            return
        
        # Calculate average performance for each strategy
        strategy_scores = {}
        for strategy, performances in self.strategy_performance.items():
            if performances:
                strategy_scores[strategy] = sum(performances) / len(performances)
            else:
                strategy_scores[strategy] = 0.0
        
        # Select best strategy
        best_strategy = max(strategy_scores, key=strategy_scores.get)
        
        if best_strategy != self.current_strategy:
            logger.info(
                f"Agent {self.id} switching strategy: "
                f"{self.current_strategy} -> {best_strategy}"
            )
            self.current_strategy = best_strategy
    
    def _mutate(self) -> None:
        """
        Mutate agent by adding or modifying strategies.
        """
        mutation_types = ["add_strategy", "modify_capability"]
        mutation = random.choice(mutation_types)
        
        if mutation == "add_strategy":
            new_strategy = f"strategy_{len(self.strategies) + 1}"
            self.strategies.append(new_strategy)
            self.strategy_performance[new_strategy] = []
            logger.debug(f"Agent {self.id} mutated: added {new_strategy}")
        
        elif mutation == "modify_capability":
            new_capability = f"evolved_capability_{self.generation}"
            self.add_capability(new_capability)
            logger.debug(f"Agent {self.id} mutated: gained {new_capability}")
    
    def get_evolution_stats(self) -> Dict[str, Any]:
        """
        Get evolutionary statistics.
        
        Returns:
            Dictionary of evolution stats
        """
        return {
            "generation": self.generation,
            "fitness_score": self.fitness_score,
            "mutation_rate": self.mutation_rate,
            "strategies": self.strategies,
            "current_strategy": self.current_strategy,
            "strategy_count": len(self.strategies)
        }

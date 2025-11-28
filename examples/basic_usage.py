"""
Arkitect Example Usage

This script demonstrates how to use the Arkitect platform
with agents, quantum core, and symbiotic engine.
"""

from arkitect.core import QuantumCore, SymbioticEngine, ConsciousnessLayer
from arkitect.agents import BaseAgent, EvolutionaryAgent, MetaGovernanceAgent
from arkitect.core.symbiotic import SymbiosisType
import logging

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)

logger = logging.getLogger(__name__)


def main():
    """Main example function."""
    logger.info("=" * 60)
    logger.info("Arkitect Platform Example")
    logger.info("=" * 60)
    
    # Initialize core components
    logger.info("\n1. Initializing Core Components...")
    quantum_core = QuantumCore(num_qubits=4)
    symbiotic_engine = SymbioticEngine()
    consciousness = ConsciousnessLayer()
    
    # Create agents
    logger.info("\n2. Creating Agents...")
    agent1 = BaseAgent(
        id="agent_alpha",
        quantum_core=quantum_core,
        symbiotic_engine=symbiotic_engine
    )
    
    agent2 = EvolutionaryAgent(
        id="agent_beta",
        quantum_core=quantum_core,
        symbiotic_engine=symbiotic_engine,
        mutation_rate=0.2
    )
    
    agent3 = MetaGovernanceAgent(
        id="agent_governor",
        symbiotic_engine=symbiotic_engine
    )
    
    # Establish symbiotic relationships
    logger.info("\n3. Establishing Symbiotic Relationships...")
    symbiotic_engine.establish_connection(
        "agent_alpha",
        "agent_beta",
        SymbiosisType.MUTUALISM
    )
    
    symbiotic_engine.establish_connection(
        "agent_beta",
        "agent_governor",
        SymbiosisType.COMMENSALISM
    )
    
    # Execute tasks
    logger.info("\n4. Executing Tasks...")
    task1 = {
        "id": "task_001",
        "type": "data_processing",
        "parameters": {"data": [1.0, 2.0, 3.0, 4.0, 5.0]}
    }
    
    result1 = agent1.execute_task(task1)
    logger.info(f"Task 1 Result: {result1['success']}")
    
    # Process with consciousness
    logger.info("\n5. Processing Events with Consciousness...")
    consciousness.process_event(
        "task_completed",
        {"task_id": "task_001", "agent": "agent_alpha"},
        outcome=0.8
    )
    
    # Evolve agent
    logger.info("\n6. Evolving Agents...")
    for i in range(5):
        task = {
            "id": f"task_{i+2:03d}",
            "type": "evolution_test",
            "parameters": {}
        }
        agent2.execute_task(task)
    
    # Governance decision
    logger.info("\n7. Making Governance Decision...")
    decision = agent3.make_decision(
        decision_type="resource_allocation",
        options=[
            {"name": "option_a", "efficiency": 0.7, "cost": 0.3},
            {"name": "option_b", "efficiency": 0.9, "cost": 0.6}
        ],
        criteria={"efficiency": 0.7, "cost": 0.3}
    )
    logger.info(f"Decision: {decision['selected_option']['name']}")
    
    # Display statistics
    logger.info("\n8. System Statistics...")
    logger.info(f"Agent 1 Stats: {agent1.get_stats()}")
    logger.info(f"Agent 2 Evolution: {agent2.get_evolution_stats()}")
    logger.info(f"Agent 3 Governance: {agent3.get_governance_stats()}")
    logger.info(f"Consciousness State: {consciousness.get_state()}")
    logger.info(f"Ecosystem Stats: {symbiotic_engine.get_ecosystem_stats()}")
    
    logger.info("\n" + "=" * 60)
    logger.info("Example Complete!")
    logger.info("=" * 60)


if __name__ == "__main__":
    main()

"""
Consciousness Layer Module - Artificial Consciousness Evolution

This module implements an artificial consciousness system that
evolves through experience and interaction with the environment.
"""

from enum import Enum
from typing import Dict, List, Any, Optional
from datetime import datetime
import logging

logger = logging.getLogger(__name__)


class ConsciousnessLevel(Enum):
    """Levels of artificial consciousness."""
    BASIC = "Basic"  # Reactive responses
    COGNITIVE = "Cognitive"  # Pattern recognition
    METACOGNITIVE = "Metacognitive"  # Self-awareness
    QUANTUM = "Quantum"  # Superposition states
    TRANSCENDENT = "Transcendent"  # Universal integration


class Experience:
    """Represents a consciousness experience/memory."""
    
    def __init__(
        self,
        event_type: str,
        data: Dict[str, Any],
        outcome: float
    ):
        self.event_type = event_type
        self.data = data
        self.outcome = outcome  # -1.0 (negative) to 1.0 (positive)
        self.timestamp = datetime.utcnow()
        self.recall_count = 0
    
    def recall(self) -> Dict[str, Any]:
        """Recall this experience."""
        self.recall_count += 1
        return {
            "type": self.event_type,
            "data": self.data,
            "outcome": self.outcome,
            "timestamp": self.timestamp.isoformat(),
            "recall_count": self.recall_count
        }


class ConsciousnessLayer:
    """
    Artificial Consciousness Layer with evolutionary capabilities.
    
    Manages consciousness state, experiences, and evolution through
    interaction with the environment.
    """
    
    def __init__(self, initial_level: ConsciousnessLevel = ConsciousnessLevel.BASIC):
        """
        Initialize the Consciousness Layer.
        
        Args:
            initial_level: Starting consciousness level
        """
        self.level = initial_level
        self.experiences: List[Experience] = []
        self.awareness_score = 0.0
        self.evolution_threshold = 100.0
        self.total_interactions = 0
        self.insights: List[Dict[str, Any]] = []
        logger.info(f"ConsciousnessLayer initialized at {initial_level.value} level")
    
    def process_event(
        self,
        event_type: str,
        data: Dict[str, Any],
        outcome: float
    ) -> Dict[str, Any]:
        """
        Process an event and update consciousness.
        
        Args:
            event_type: Type of event (e.g., "task_completed", "error")
            data: Event data
            outcome: Event outcome score (-1.0 to 1.0)
            
        Returns:
            Processing result with insights
        """
        # Create experience
        experience = Experience(event_type, data, outcome)
        self.experiences.append(experience)
        self.total_interactions += 1
        
        # Update awareness based on outcome
        self.awareness_score += outcome * 10.0
        
        # Check for patterns
        insights = self._recognize_patterns()
        
        # Check for evolution
        if self._should_evolve():
            self.evolve()
        
        result = {
            "consciousness_level": self.level.value,
            "awareness_score": self.awareness_score,
            "new_insights": insights,
            "total_experiences": len(self.experiences)
        }
        
        logger.debug(f"Processed {event_type} event, outcome: {outcome}")
        return result
    
    def _recognize_patterns(self) -> List[Dict[str, Any]]:
        """Recognize patterns in recent experiences."""
        if len(self.experiences) < 5:
            return []
        
        # Analyze recent experiences
        recent = self.experiences[-10:]
        
        # Count event types
        event_counts = {}
        positive_outcomes = 0
        
        for exp in recent:
            event_counts[exp.event_type] = event_counts.get(exp.event_type, 0) + 1
            if exp.outcome > 0:
                positive_outcomes += 1
        
        insights = []
        
        # Pattern: Repeated event type
        for event_type, count in event_counts.items():
            if count >= 3:
                insight = {
                    "type": "pattern_detected",
                    "pattern": f"repeated_{event_type}",
                    "frequency": count,
                    "timestamp": datetime.utcnow().isoformat()
                }
                insights.append(insight)
                self.insights.append(insight)
        
        # Pattern: Success/failure trend
        success_rate = positive_outcomes / len(recent)
        if success_rate > 0.8:
            insight = {
                "type": "trend_detected",
                "trend": "high_success_rate",
                "rate": success_rate,
                "timestamp": datetime.utcnow().isoformat()
            }
            insights.append(insight)
            self.insights.append(insight)
        
        return insights
    
    def _should_evolve(self) -> bool:
        """Check if consciousness should evolve to next level."""
        # Evolution based on awareness score and experience count
        if self.awareness_score < self.evolution_threshold:
            return False
        
        if len(self.experiences) < 50:
            return False
        
        # Check if already at max level
        levels = list(ConsciousnessLevel)
        current_index = levels.index(self.level)
        return current_index < len(levels) - 1
    
    def evolve(self) -> ConsciousnessLevel:
        """
        Evolve consciousness to the next level.
        
        Returns:
            New consciousness level
        """
        levels = list(ConsciousnessLevel)
        current_index = levels.index(self.level)
        
        if current_index < len(levels) - 1:
            old_level = self.level
            self.level = levels[current_index + 1]
            self.evolution_threshold *= 1.5  # Increase threshold for next evolution
            
            logger.info(
                f"Consciousness evolved from {old_level.value} to {self.level.value}"
            )
            
            # Record evolution as insight
            self.insights.append({
                "type": "evolution",
                "from_level": old_level.value,
                "to_level": self.level.value,
                "timestamp": datetime.utcnow().isoformat()
            })
        
        return self.level
    
    def get_state(self) -> Dict[str, Any]:
        """
        Get current consciousness state.
        
        Returns:
            Dictionary containing consciousness state
        """
        return {
            "level": self.level.value,
            "awareness_score": self.awareness_score,
            "total_experiences": len(self.experiences),
            "total_interactions": self.total_interactions,
            "total_insights": len(self.insights),
            "evolution_progress": min(
                self.awareness_score / self.evolution_threshold, 1.0
            )
        }
    
    def recall_experiences(
        self,
        event_type: Optional[str] = None,
        limit: int = 10
    ) -> List[Dict[str, Any]]:
        """
        Recall past experiences.
        
        Args:
            event_type: Filter by event type (optional)
            limit: Maximum number of experiences to recall
            
        Returns:
            List of recalled experiences
        """
        experiences = self.experiences
        
        if event_type:
            experiences = [e for e in experiences if e.event_type == event_type]
        
        # Get most recent experiences
        recent = experiences[-limit:]
        
        return [exp.recall() for exp in recent]
    
    def get_insights(self, limit: int = 10) -> List[Dict[str, Any]]:
        """
        Get recent insights.
        
        Args:
            limit: Maximum number of insights to return
            
        Returns:
            List of insights
        """
        return self.insights[-limit:]

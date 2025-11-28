"""
Task Scheduler Module - Intelligent Task Scheduling

This module provides advanced task scheduling and prioritization
for the Arkitect platform.
"""

from typing import List, Dict, Any, Optional
from enum import Enum
import logging
from dataclasses import dataclass
from datetime import datetime

logger = logging.getLogger(__name__)


class SchedulingStrategy(Enum):
    """Task scheduling strategies."""
    FIFO = "fifo"  # First In First Out
    PRIORITY = "priority"  # Priority-based
    SHORTEST_JOB_FIRST = "sjf"  # Shortest job first
    ROUND_ROBIN = "round_robin"  # Round robin
    DEADLINE = "deadline"  # Deadline-based


@dataclass
class TaskMetrics:
    """Metrics for task execution."""
    estimated_duration: float = 0.0
    actual_duration: float = 0.0
    cpu_usage: float = 0.0
    memory_usage: float = 0.0
    success_rate: float = 1.0


class TaskScheduler:
    """
    Intelligent Task Scheduler.
    
    Provides advanced scheduling algorithms for optimal
    task execution and resource utilization.
    """
    
    def __init__(self, strategy: SchedulingStrategy = SchedulingStrategy.PRIORITY):
        """
        Initialize the Task Scheduler.
        
        Args:
            strategy: Scheduling strategy to use
        """
        self.strategy = strategy
        self.task_queue: List[Dict[str, Any]] = []
        self.completed_tasks: List[Dict[str, Any]] = []
        self.task_metrics: Dict[str, TaskMetrics] = {}
        logger.info(f"TaskScheduler initialized with {strategy.value} strategy")
    
    def schedule_task(self, task: Dict[str, Any]) -> int:
        """
        Schedule a task for execution.
        
        Args:
            task: Task dictionary with id, priority, etc.
            
        Returns:
            Position in queue
        """
        task_id = task.get('id', f'task_{len(self.task_queue)}')
        task['scheduled_at'] = datetime.utcnow().isoformat()
        
        # Initialize metrics for task
        if task_id not in self.task_metrics:
            self.task_metrics[task_id] = TaskMetrics()
        
        # Add to queue based on strategy
        if self.strategy == SchedulingStrategy.PRIORITY:
            self._schedule_by_priority(task)
        elif self.strategy == SchedulingStrategy.DEADLINE:
            self._schedule_by_deadline(task)
        else:
            self.task_queue.append(task)
        
        position = self._find_task_position(task_id)
        logger.info(f"Scheduled task {task_id} at position {position}")
        return position
    
    def _schedule_by_priority(self, task: Dict[str, Any]) -> None:
        """Schedule task by priority."""
        priority_map = {'critical': 0, 'high': 1, 'medium': 2, 'low': 3}
        task_priority = priority_map.get(task.get('priority', 'medium'), 2)
        
        # Insert task in priority order
        inserted = False
        for i, queued_task in enumerate(self.task_queue):
            queued_priority = priority_map.get(queued_task.get('priority', 'medium'), 2)
            if task_priority < queued_priority:
                self.task_queue.insert(i, task)
                inserted = True
                break
        
        if not inserted:
            self.task_queue.append(task)
    
    def _schedule_by_deadline(self, task: Dict[str, Any]) -> None:
        """Schedule task by deadline."""
        deadline = task.get('deadline')
        if not deadline:
            self.task_queue.append(task)
            return
        
        # Insert task in deadline order
        inserted = False
        for i, queued_task in enumerate(self.task_queue):
            queued_deadline = queued_task.get('deadline')
            if queued_deadline and deadline < queued_deadline:
                self.task_queue.insert(i, task)
                inserted = True
                break
        
        if not inserted:
            self.task_queue.append(task)
    
    def _find_task_position(self, task_id: str) -> int:
        """Find position of task in queue."""
        for i, task in enumerate(self.task_queue):
            if task.get('id') == task_id:
                return i
        return -1
    
    def get_next_task(self) -> Optional[Dict[str, Any]]:
        """
        Get next task to execute.
        
        Returns:
            Next task or None if queue is empty
        """
        if not self.task_queue:
            return None
        
        task = self.task_queue.pop(0)
        task['started_at'] = datetime.utcnow().isoformat()
        logger.debug(f"Retrieved task {task.get('id')} from queue")
        return task
    
    def mark_completed(self, task_id: str, success: bool = True, duration: float = 0.0) -> None:
        """
        Mark task as completed.
        
        Args:
            task_id: Task identifier
            success: Whether task succeeded
            duration: Actual execution duration
        """
        if task_id in self.task_metrics:
            metrics = self.task_metrics[task_id]
            metrics.actual_duration = duration
            if not success:
                metrics.success_rate *= 0.9  # Decrease success rate
        
        completed_task = {
            'id': task_id,
            'completed_at': datetime.utcnow().isoformat(),
            'success': success,
            'duration': duration
        }
        self.completed_tasks.append(completed_task)
        logger.info(f"Task {task_id} marked as {'completed' if success else 'failed'}")
    
    def get_queue_stats(self) -> Dict[str, Any]:
        """
        Get queue statistics.
        
        Returns:
            Dictionary of queue stats
        """
        priority_counts = {}
        for task in self.task_queue:
            priority = task.get('priority', 'medium')
            priority_counts[priority] = priority_counts.get(priority, 0) + 1
        
        return {
            'total_queued': len(self.task_queue),
            'total_completed': len(self.completed_tasks),
            'by_priority': priority_counts,
            'strategy': self.strategy.value
        }
    
    def optimize_schedule(self, tasks: List[Dict[str, Any]]) -> List[int]:
        """
        Optimize task execution order.
        
        Args:
            tasks: List of tasks to optimize
            
        Returns:
            Optimized order (indices)
        """
        if not tasks:
            return []
        
        # Optimize based on priority and estimated duration
        scored_tasks = []
        for i, task in enumerate(tasks):
            priority_map = {'critical': 4, 'high': 3, 'medium': 2, 'low': 1}
            priority_score = priority_map.get(task.get('priority', 'medium'), 2)
            
            # Favor shorter tasks with higher priority
            duration = self.task_metrics.get(
                task.get('id', ''), TaskMetrics()
            ).estimated_duration or 1.0
            
            score = priority_score / max(duration, 0.1)
            scored_tasks.append((i, score))
        
        # Sort by score (descending)
        scored_tasks.sort(key=lambda x: x[1], reverse=True)
        optimized_order = [i for i, _ in scored_tasks]
        
        logger.info(f"Optimized schedule for {len(tasks)} tasks")
        return optimized_order

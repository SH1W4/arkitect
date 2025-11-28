"""
Metrics Collector Module - System and Task Metrics

This module provides comprehensive metrics collection and
monitoring for the Arkitect platform.
"""

from typing import Dict, List, Any, Optional
from datetime import datetime, timedelta
from collections import deque
import logging

logger = logging.getLogger(__name__)


class MetricType:
    """Metric type constants."""
    COUNTER = "counter"
    GAUGE = "gauge"
    HISTOGRAM = "histogram"
    SUMMARY = "summary"


class MetricsCollector:
    """
    Metrics Collector for system and task monitoring.
    
    Collects, stores, and analyzes metrics for performance
    monitoring and optimization.
    """
    
    def __init__(self, retention_hours: int = 24):
        """
        Initialize the Metrics Collector.
        
        Args:
            retention_hours: How long to retain metrics (default: 24 hours)
        """
        self.retention_hours = retention_hours
        self.metrics: Dict[str, List[Dict[str, Any]]] = {}
        self.aggregated_stats: Dict[str, Dict[str, float]] = {}
        self.total_events = 0
        logger.info(f"MetricsCollector initialized (retention: {retention_hours}h)")
    
    def record_metric(
        self,
        metric_name: str,
        value: float,
        labels: Optional[Dict[str, str]] = None,
        metric_type: str = MetricType.GAUGE
    ) -> None:
        """
        Record a metric value.
        
        Args:
            metric_name: Name of the metric
            value: Metric value
            labels: Optional labels for the metric
            metric_type: Type of metric
        """
        if metric_name not in self.metrics:
            self.metrics[metric_name] = []
        
        metric_entry = {
            'value': value,
            'timestamp': datetime.utcnow(),
            'labels': labels or {},
            'type': metric_type
        }
        
        self.metrics[metric_name].append(metric_entry)
        self.total_events += 1
        
        # Clean old metrics
        self._cleanup_old_metrics(metric_name)
        
        # Update aggregated stats
        self._update_aggregated_stats(metric_name)
        
        logger.debug(f"Recorded metric {metric_name}: {value}")
    
    def _cleanup_old_metrics(self, metric_name: str) -> None:
        """Remove metrics older than retention period."""
        cutoff_time = datetime.utcnow() - timedelta(hours=self.retention_hours)
        
        self.metrics[metric_name] = [
            m for m in self.metrics[metric_name]
            if m['timestamp'] > cutoff_time
        ]
    
    def _update_aggregated_stats(self, metric_name: str) -> None:
        """Update aggregated statistics for a metric."""
        values = [m['value'] for m in self.metrics[metric_name]]
        
        if not values:
            return
        
        self.aggregated_stats[metric_name] = {
            'count': len(values),
            'sum': sum(values),
            'avg': sum(values) / len(values),
            'min': min(values),
            'max': max(values),
            'last': values[-1]
        }
    
    def get_metric(self, metric_name: str, last_n: Optional[int] = None) -> List[Dict[str, Any]]:
        """
        Get metric values.
        
        Args:
            metric_name: Name of the metric
            last_n: Get only last N values (optional)
            
        Returns:
            List of metric entries
        """
        if metric_name not in self.metrics:
            return []
        
        metrics = self.metrics[metric_name]
        
        if last_n:
            return metrics[-last_n:]
        
        return metrics
    
    def get_aggregated_stats(self, metric_name: str) -> Dict[str, float]:
        """
        Get aggregated statistics for a metric.
        
        Args:
            metric_name: Name of the metric
            
        Returns:
            Dictionary of aggregated stats
        """
        return self.aggregated_stats.get(metric_name, {})
    
    def get_all_metrics(self) -> Dict[str, Any]:
        """
        Get all metrics and their stats.
        
        Returns:
            Dictionary of all metrics and stats
        """
        return {
            'metric_names': list(self.metrics.keys()),
            'total_events': self.total_events,
            'retention_hours': self.retention_hours,
            'stats': self.aggregated_stats
        }
    
    def record_task_execution(
        self,
        task_id: str,
        duration: float,
        success: bool,
        resource_usage: Optional[Dict[str, float]] = None
    ) -> None:
        """
        Record task execution metrics.
        
        Args:
            task_id: Task identifier
            duration: Execution duration in seconds
            success: Whether task succeeded
            resource_usage: Optional resource usage metrics
        """
        # Record duration
        self.record_metric(
            'task_duration_seconds',
            duration,
            labels={'task_id': task_id, 'status': 'success' if success else 'failed'}
        )
        
        # Record success/failure
        self.record_metric(
            'task_status',
            1.0 if success else 0.0,
            labels={'task_id': task_id},
            metric_type=MetricType.COUNTER
        )
        
        # Record resource usage if provided
        if resource_usage:
            for resource, value in resource_usage.items():
                self.record_metric(
                    f'task_{resource}',
                    value,
                    labels={'task_id': task_id}
                )
        
        logger.info(f"Recorded execution metrics for task {task_id}")
    
    def get_system_health(self) -> Dict[str, Any]:
        """
        Get overall system health metrics.
        
        Returns:
            Dictionary of health metrics
        """
        # Calculate success rate
        task_statuses = self.get_metric('task_status', last_n=100)
        if task_statuses:
            success_count = sum(1 for m in task_statuses if m['value'] == 1.0)
            success_rate = success_count / len(task_statuses)
        else:
            success_rate = 1.0
        
        # Get average duration
        duration_stats = self.get_aggregated_stats('task_duration_seconds')
        avg_duration = duration_stats.get('avg', 0.0)
        
        return {
            'success_rate': success_rate,
            'avg_task_duration': avg_duration,
            'total_tasks': len(task_statuses),
            'health_score': success_rate * 100,
            'timestamp': datetime.utcnow().isoformat()
        }
    
    def get_performance_metrics(self, time_window_hours: int = 1) -> Dict[str, Any]:
        """
        Get performance metrics for a time window.
        
        Args:
            time_window_hours: Time window in hours
            
        Returns:
            Dictionary of performance metrics
        """
        cutoff = datetime.utcnow() - timedelta(hours=time_window_hours)
        
        # Filter metrics within time window
        recent_tasks = [
            m for m in self.get_metric('task_status')
            if m['timestamp'] > cutoff
        ]
        
        if not recent_tasks:
            return {'tasks_executed': 0, 'time_window_hours': time_window_hours}
        
        success_count = sum(1 for m in recent_tasks if m['value'] == 1.0)
        
        return {
            'time_window_hours': time_window_hours,
            'tasks_executed': len(recent_tasks),
            'tasks_succeeded': success_count,
            'tasks_failed': len(recent_tasks) - success_count,
            'success_rate': success_count / len(recent_tasks) if recent_tasks else 0.0,
            'throughput_per_hour': len(recent_tasks) / time_window_hours
        }

"""
ARKITECT Development Orchestrator
Sistema de orquestração do desenvolvimento integrado
"""

import asyncio
from typing import Dict, List, Any
import logging
from datetime import datetime
import uuid

from task_mesh.executor import Task, TaskExecutor
from integration_layer.connector import create_connector, EONFrameworkConnector

class DevelopmentTask:
    """Representa uma tarefa de desenvolvimento com metadados e capacidade evolutiva"""
    def __init__(self, name: str, task_type: str, priority: int = 1):
        self.id = str(uuid.uuid4())
        self.name = name
        self.type = task_type
        self.priority = priority
        self.status = "created"
        self.created_at = datetime.now()
        self.metadata: Dict[str, Any] = {}
        self.dependencies: List[str] = []
        self.task_metrics = {
            'execution_time': [],        # Tempo de execução em ms
            'memory_usage': [],          # Uso de memória em MB
            'cpu_usage': [],             # Uso de CPU em %
            'error_count': [],           # Número de erros
            'dependency_wait_time': []    # Tempo esperando dependências em ms
        }
        self.improvement_data = {
            'performance_bottlenecks': [],  # Gargalos identificados
            'resource_constraints': [],     # Limitações de recursos
            'optimization_history': []      # Histórico de otimizações
        }
        
    def add_dependency(self, task_id: str):
        """Adiciona uma dependência à tarefa"""
        if task_id not in self.dependencies:
            self.dependencies.append(task_id)
            
    def update_status(self, status: str):
        """Atualiza o status da tarefa"""
        self.status = status
        
    def to_task_mesh(self) -> Task:
        """Converte para formato do Task Mesh"""
        return Task(
            id=self.id,
            name=self.name,
            type=self.type,
            priority=self.priority,
            dependencies=self.dependencies,
            metadata=self.metadata
        )

from typing import List, Tuple

class DevelopmentOrchestrator:
    """Orquestrador principal do desenvolvimento"""
    
    def __init__(self):
        self.logger = logging.getLogger("development.orchestrator")
        self.task_executor = TaskExecutor()
        self.eon_connector: EONFrameworkConnector = create_connector("eon_framework")
        self.tasks: Dict[str, DevelopmentTask] = {}
        self.setup_logging()

    def setup_logging(self):
        """Configura logging detalhado"""
        logging.basicConfig(
            format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            level=logging.INFO
        )
        
        # Adiciona handler para métricas
        metrics_handler = logging.FileHandler('metrics.log')
        metrics_handler.setLevel(logging.DEBUG)
        self.logger.addHandler(metrics_handler)

    def calculate_trend(self, metrics: List[float]) -> Dict[str, float]:
        """Calcula tendência de uma série de métricas"""
        if not metrics or len(metrics) < 2:
            return {'slope': 0.0, 'average': 0.0, 'variance': 0.0}
        
        avg = sum(metrics) / len(metrics)
        variance = sum((x - avg) ** 2 for x in metrics) / len(metrics)
        
        # Calcula tendência linear simples
        if len(metrics) > 1:
            slope = (metrics[-1] - metrics[0]) / len(metrics)
        else:
            slope = 0.0
        
        return {
            'slope': slope,
            'average': avg,
            'variance': variance
        }

    def calculate_average(self, metrics: List[float]) -> float:
        """Calcula a média de uma série de métricas"""
        return sum(metrics) / len(metrics) if metrics else 0.0

    async def analyze_metric_trends(self, task: DevelopmentTask) -> Dict[str, Any]:
        """Analisa tendências das métricas ao longo do tempo"""
        return {
            'execution_trend': self.calculate_trend(task.task_metrics['execution_time']),
            'memory_trend': self.calculate_trend(task.task_metrics['memory_usage']),
            'cpu_trend': self.calculate_trend(task.task_metrics['cpu_usage']),
            'error_trend': self.calculate_trend(task.task_metrics['error_count'])
        }

    async def check_execution_thresholds(self, task: DevelopmentTask) -> List[Tuple[str, float]]:
        """Verifica limiares de execução e gera alertas"""
        thresholds = {
            'memory_limit': 85.0,  # % de uso de memória
            'cpu_limit': 90.0,     # % de uso de CPU
            'error_limit': 5,      # Número de erros consecutivos
            'execution_limit': 300  # Tempo máximo em segundos
        }
        
        alerts = []
        if task.task_metrics['memory_usage'] and task.task_metrics['memory_usage'][-1] > thresholds['memory_limit']:
            alerts.append(('HIGH_MEMORY_USAGE', task.task_metrics['memory_usage'][-1]))
        
        if task.task_metrics['cpu_usage'] and task.task_metrics['cpu_usage'][-1] > thresholds['cpu_limit']:
            alerts.append(('HIGH_CPU_USAGE', task.task_metrics['cpu_usage'][-1]))

        if len(task.task_metrics['error_count']) >= thresholds['error_limit']:
            alerts.append(('HIGH_ERROR_RATE', len(task.task_metrics['error_count'])))

        for alert, value in alerts:
            self.logger.warning(f"Alert {alert} for task {task.name}: {value}")
            
        return alerts

    def merge_metrics(self, local_metrics: Dict, eon_metrics: Dict) -> Dict:
        """Combina métricas locais com métricas do EON de forma inteligente"""
        merged = {}
        
        for key in set(local_metrics.keys()) | set(eon_metrics.keys()):
            if key in local_metrics and key in eon_metrics:
                # Usa a média para métricas numéricas
                if isinstance(local_metrics[key], (int, float)):
                    merged[key] = (local_metrics[key] + eon_metrics[key]) / 2
                else:
                    # Para outros tipos, prioriza dados locais
                    merged[key] = local_metrics[key]
            else:
                # Usa o valor disponível
                merged[key] = local_metrics.get(key, eon_metrics.get(key))
        
        return merged

    async def start(self):
        """Inicia o orquestrador"""
        self.logger.info("Iniciando orquestrador de desenvolvimento")
        await self.eon_connector.connect()
        await self.task_executor.start()
        
    async def stop(self):
        """Para o orquestrador"""
        self.logger.info("Parando orquestrador de desenvolvimento")
        await self.task_executor.stop()
        await self.eon_connector.disconnect()
        
    async def create_development_task(self, 
                                    name: str, 
                                    task_type: str, 
                                    priority: int = 1,
                                    dependencies: List[str] = None) -> DevelopmentTask:
        """Cria uma nova tarefa de desenvolvimento"""
        task = DevelopmentTask(name, task_type, priority)
        if dependencies:
            for dep_id in dependencies:
                task.add_dependency(dep_id)
                
        self.tasks[task.id] = task
        self.logger.info(f"Nova tarefa de desenvolvimento criada: {task.name} ({task.id})")
        
        # Sincroniza com EON-Framework
        await self.sync_task_with_eon(task)
        
        return task
        
    async def submit_task(self, task: DevelopmentTask):
        """Submete uma tarefa para execução"""
        # Converte e adiciona ao executor
        mesh_task = task.to_task_mesh()
        self.task_executor.add_task(mesh_task)
        
        # Atualiza status
        task.update_status("submitted")
        self.logger.info(f"Tarefa submetida para execução: {task.name}")
        
    async def sync_task_with_eon(self, task: DevelopmentTask):
        """Sincroniza uma tarefa com o EON-Framework"""
        try:
            sync_data = {
                "task_id": task.id,
                "name": task.name,
                "type": task.type,
                "status": task.status,
                "metadata": task.metadata
            }
            
            await self.eon_connector.sync_development_state(sync_data)
            self.logger.info(f"Tarefa sincronizada com EON-Framework: {task.name}")
            
        except Exception as e:
            self.logger.error(f"Erro ao sincronizar tarefa com EON-Framework: {str(e)}")
            raise
            
    async def monitor_task_execution(self, task_id: str):
        """Monitora execução com métricas específicas"""
        task = self.tasks.get(task_id)
        if not task:
            raise ValueError(f"Tarefa não encontrada: {task_id}")
            
        while task.status not in ["completed", "failed"]:
            # Coleta métricas do sistema
            system_metrics = await self.task_executor.get_system_metrics(task.id)
            
            # Coleta métricas de execução
            execution_metrics = await self.collect_execution_metrics(task)
            
            # Analisa tendências
            trends = await self.analyze_metric_trends(task)
            self.logger.info(f"Metric trends for task {task.name}: {trends}")
            
            # Verifica alertas
            alerts = await self.check_execution_thresholds(task)
            if alerts:
                self.logger.warning(f"Alerts detected for task {task.name}: {alerts}")
            
            # Analisa dependências
            dependency_metrics = await self.analyze_dependency_chain(task)
            
            # Atualiza métricas
            self.update_task_metrics(task, {
                'system': system_metrics,
                'execution': execution_metrics,
                'dependencies': dependency_metrics
            })
            
            # Verifica limiares e alertas
            await self.check_execution_thresholds(task)
            
            self.logger.info(f"Status da tarefa {task.name}: {task.status}")
            self.logger.debug(f"Métricas atualizadas para {task.name}")
            
            # Registra histórico de otimização se houver mudanças
            if execution_metrics.get('optimization_applied'):
                task.improvement_data['optimization_history'].append({
                    'timestamp': datetime.now(),
                    'type': execution_metrics['optimization_type'],
                    'impact': execution_metrics['optimization_impact']
                })
            
            await asyncio.sleep(5)  # Intervalo de polling
            
        # Análise final após conclusão
        await self.perform_task_retrospective(task)
        return task.status
        
    async def collect_task_metrics(self, task: DevelopmentTask) -> Dict[str, float]:
        """Coleta métricas detalhadas de uma tarefa"""
        metrics = {
            'performance': 0.0,
            'resources': 0.0,
            'completion': 0.0,
            'errors': 0.0
        }
        
        # Coleta métricas do executor
        executor_metrics = await self.task_executor.get_task_metrics(task.id)
        if executor_metrics:
            metrics.update(executor_metrics)
            
        # Integra com métricas do EON-Framework
        eon_metrics = await self.eon_connector.get_task_metrics(task.id)
        if eon_metrics:
            # Combina métricas locais e do framework
            metrics = self.merge_metrics(metrics, eon_metrics)
            
        return metrics
        
    def analyze_execution_times(self, times: List[float]) -> Dict[str, Any]:
        """Analisa tempos de execução para identificar padrões"""
        if not times:
            return {}
        
        return {
            'min': min(times),
            'max': max(times),
            'avg': sum(times) / len(times),
            'trend': self.calculate_trend(times)
        }

    def analyze_resource_usage(self, memory: List[float], cpu: List[float]) -> Dict[str, Any]:
        """Analisa uso de recursos para identificar padrões"""
        return {
            'memory': {
                'peak': max(memory) if memory else 0,
                'trend': self.calculate_trend(memory)
            },
            'cpu': {
                'peak': max(cpu) if cpu else 0,
                'trend': self.calculate_trend(cpu)
            }
        }

    def identify_performance_bottlenecks(self, execution_trend: Dict, 
                                    resource_usage: Dict,
                                    wait_times: List[float]) -> List[Dict]:
        """Identifica gargalos específicos de performance"""
        bottlenecks = []
        
        # Analisa tendência de execução
        if execution_trend.get('slope', 0) > 0.1:
            bottlenecks.append({
                'type': 'EXECUTION_DEGRADATION',
                'severity': 'HIGH',
                'trend': execution_trend['slope']
            })
        
        # Analisa uso de recursos
        if resource_usage['memory']['peak'] > 90:
            bottlenecks.append({
                'type': 'MEMORY_PRESSURE',
                'severity': 'HIGH',
                'peak': resource_usage['memory']['peak']
            })
        
        # Analisa tempos de espera
        if wait_times and max(wait_times) > 1000:  # mais de 1 segundo
            bottlenecks.append({
                'type': 'DEPENDENCY_DELAY',
                'severity': 'MEDIUM',
                'max_wait': max(wait_times)
            })
        
        return bottlenecks

    def analyze_resource_constraints(self, memory_usage: List[float], cpu_usage: List[float]) -> List[Dict]:
        """Analisa restrições de recursos do sistema"""
        constraints = []
        
        if memory_usage:
            peak_memory = max(memory_usage)
            if peak_memory > 80:
                constraints.append({
                    'type': 'MEMORY_CONSTRAINT',
                    'severity': 'HIGH' if peak_memory > 90 else 'MEDIUM',
                    'value': peak_memory
                })
                
        if cpu_usage:
            peak_cpu = max(cpu_usage)
            if peak_cpu > 85:
                constraints.append({
                    'type': 'CPU_CONSTRAINT',
                    'severity': 'HIGH' if peak_cpu > 95 else 'MEDIUM',
                    'value': peak_cpu
                })
                
        return constraints

    async def analyze_execution_patterns(self, task: DevelopmentTask, metrics: Dict[str, float]):
        """Analisa padrões concretos de execução"""
        if len(task.task_metrics['execution_time']) > 0:
            # Análise de tempo de execução
            execution_trend = self.analyze_execution_times(
                task.task_metrics['execution_time']
            )
            
            # Análise de uso de recursos
            resource_usage = self.analyze_resource_usage(
                task.task_metrics['memory_usage'],
                task.task_metrics['cpu_usage']
            )
            
            # Identifica gargalos específicos
            bottlenecks = self.identify_performance_bottlenecks(
                execution_trend,
                resource_usage,
                task.task_metrics['dependency_wait_time']
            )
            
            if bottlenecks:
                task.improvement_data['performance_bottlenecks'].extend(bottlenecks)
                self.logger.info(f"Gargalos identificados para {task.name}: {bottlenecks}")
            
            # Análise de restrições de recursos
            constraints = self.analyze_resource_constraints(
                task.task_metrics['memory_usage'],
                task.task_metrics['cpu_usage']
            )
            
            if constraints:
                task.improvement_data['resource_constraints'].extend(constraints)
                self.logger.info(f"Restrições de recursos para {task.name}: {constraints}")
                
    async def perform_task_retrospective(self, task: DevelopmentTask):
        """Realiza análise retrospectiva detalhada da tarefa"""
        # Métricas básicas de execução
        execution_metrics = {
            'avg_time': self.calculate_average(task.task_metrics['execution_time']),
            'peak_memory': max(task.task_metrics['memory_usage']) if task.task_metrics['memory_usage'] else 0,
            'avg_cpu': self.calculate_average(task.task_metrics['cpu_usage']),
            'total_errors': sum(task.task_metrics['error_count']),
            'total_wait_time': sum(task.task_metrics['dependency_wait_time'])
        }
        
        # Análise de tendências
        trends = await self.analyze_metric_trends(task)
        
        # Análise detalhada de recursos
        resource_analysis = {
            'memory_efficiency': self.analyze_memory_efficiency(task.task_metrics['memory_usage']),
            'cpu_utilization': self.analyze_cpu_utilization(task.task_metrics['cpu_usage']),
            'bottlenecks': task.improvement_data['performance_bottlenecks'],
            'constraints': task.improvement_data['resource_constraints']
        }
        
        # Análise de melhorias
        improvements = {
            'optimization_history': task.improvement_data['optimization_history'],
            'success_patterns': self.identify_success_patterns(task),
            'recommendations': self.generate_improvement_recommendations(task),
            'future_optimizations': self.suggest_future_optimizations(task)
        }
        
        analysis = {
            'execution_metrics': execution_metrics,
            'trends': trends,
            'resource_analysis': resource_analysis,
            'improvements': improvements,
            'summary': {
                'overall_performance': self.calculate_performance_score(execution_metrics),
                'efficiency_score': self.calculate_efficiency_score(resource_analysis),
                'improvement_potential': self.calculate_improvement_potential(improvements)
            }
        }
        
        # Registra aprendizados
        await self.eon_connector.store_task_learnings(task.id, analysis)
        self.logger.info(f"Retrospectiva completa para tarefa {task.name}")
        return analysis

    def analyze_memory_efficiency(self, memory_usage: List[float]) -> Dict[str, float]:
        """Analisa eficiência do uso de memória"""
        if not memory_usage:
            return {'efficiency': 0.0, 'stability': 0.0}
        
        avg_usage = sum(memory_usage) / len(memory_usage)
        peak_usage = max(memory_usage)
        stability = 1.0 - (peak_usage - avg_usage) / peak_usage if peak_usage > 0 else 0
        
        return {
            'efficiency': 1.0 - (avg_usage / 100),  # assume percentual
            'stability': stability
        }

    def analyze_cpu_utilization(self, cpu_usage: List[float]) -> Dict[str, float]:
        """Analisa utilização de CPU"""
        if not cpu_usage:
            return {'utilization': 0.0, 'consistency': 0.0}
        
        avg_usage = sum(cpu_usage) / len(cpu_usage)
        variance = sum((x - avg_usage) ** 2 for x in cpu_usage) / len(cpu_usage)
        consistency = 1.0 - (variance / 10000)  # normaliza variância
        
        return {
            'utilization': avg_usage / 100,  # assume percentual
            'consistency': max(0.0, min(1.0, consistency))
        }

    def calculate_performance_score(self, metrics: Dict) -> float:
        """Calcula pontuação geral de performance"""
        weights = {
            'avg_time': 0.3,
            'peak_memory': 0.2,
            'avg_cpu': 0.2,
            'total_errors': 0.2,
            'total_wait_time': 0.1
        }
        
        normalized_metrics = {
            'avg_time': 1.0 - min(1.0, metrics['avg_time'] / 1000),
            'peak_memory': 1.0 - min(1.0, metrics['peak_memory'] / 100),
            'avg_cpu': 1.0 - min(1.0, metrics['avg_cpu'] / 100),
            'total_errors': 1.0 / (1.0 + metrics['total_errors']),
            'total_wait_time': 1.0 - min(1.0, metrics['total_wait_time'] / 5000)
        }
        
        return sum(normalized_metrics[k] * weights[k] for k in weights)

    def calculate_efficiency_score(self, analysis: Dict) -> float:
        """Calcula pontuação de eficiência"""
        memory_score = analysis['memory_efficiency']['efficiency'] * 0.5 + \
                      analysis['memory_efficiency']['stability'] * 0.5
                      
        cpu_score = analysis['cpu_utilization']['utilization'] * 0.4 + \
                    analysis['cpu_utilization']['consistency'] * 0.6
                    
        return (memory_score + cpu_score) / 2

    def calculate_improvement_potential(self, improvements: Dict) -> float:
        """Calcula potencial de melhoria baseado em análises"""
        num_recommendations = len(improvements['recommendations'])
        optimization_impact = sum(opt.get('impact', 0) 
                                for opt in improvements['optimization_history'])
        
        # Normaliza pontuação
        potential = min(1.0, (num_recommendations * 0.1) + (optimization_impact * 0.05))
        return potential

    def identify_success_patterns(self, task: DevelopmentTask) -> List[Dict]:
        """Identifica padrões de sucesso na execução"""
        patterns = []
        
        # Analisa execuções bem-sucedidas
        if task.task_metrics['execution_time']:
            avg_time = sum(task.task_metrics['execution_time']) / len(task.task_metrics['execution_time'])
            fastest_times = sorted(task.task_metrics['execution_time'])[:3]  # Top 3 melhores tempos
            
            if avg_time < 1000:  # Menos de 1 segundo
                patterns.append({
                    'type': 'FAST_EXECUTION',
                    'avg_time': avg_time,
                    'best_times': fastest_times
                })
        
        # Analisa eficiência de recursos
        if task.task_metrics['memory_usage'] and task.task_metrics['cpu_usage']:
            avg_memory = sum(task.task_metrics['memory_usage']) / len(task.task_metrics['memory_usage'])
            avg_cpu = sum(task.task_metrics['cpu_usage']) / len(task.task_metrics['cpu_usage'])
            
            if avg_memory < 50 and avg_cpu < 60:  # Uso moderado de recursos
                patterns.append({
                    'type': 'RESOURCE_EFFICIENT',
                    'avg_memory': avg_memory,
                    'avg_cpu': avg_cpu
                })
        
        return patterns

    def update_task_metrics(self, task: DevelopmentTask, new_metrics: Dict[str, Any]):
        """Atualiza métricas da tarefa com novos dados"""
        # Atualiza métricas do sistema
        if 'system' in new_metrics:
            task.task_metrics['memory_usage'].append(new_metrics['system'].get('memory_percent', 0))
            task.task_metrics['cpu_usage'].append(new_metrics['system'].get('cpu_percent', 0))
        
        # Atualiza métricas de execução
        if 'execution' in new_metrics:
            task.task_metrics['execution_time'].append(new_metrics['execution'].get('elapsed_time', 0))
            if new_metrics['execution'].get('errors'):
                task.task_metrics['error_count'].append(len(new_metrics['execution']['errors']))
        
        # Atualiza métricas de dependência
        if 'dependencies' in new_metrics:
            task.task_metrics['dependency_wait_time'].append(
                new_metrics['dependencies'].get('total_wait_time', 0)
            )
        
        # Registra otimizações se houver
        if new_metrics.get('optimization_applied'):
            task.improvement_data['optimization_history'].append({
                'timestamp': datetime.now(),
                'type': new_metrics.get('optimization_type', 'unknown'),
                'impact': new_metrics.get('optimization_impact', 0)
            })
            
        self.logger.debug(f"Métricas atualizadas para tarefa {task.name}")

    async def monitor_task_progress(self, task_id: str):
        """Monitora o progresso de uma tarefa específica"""
        task = self.tasks.get(task_id)
        if not task:
            raise ValueError(f"Tarefa não encontrada: {task_id}")
        
        start_time = datetime.now()
        self.logger.info(f"Iniciando monitoramento da tarefa {task.name}")
        
        try:
            while task.status not in ["completed", "failed"]:
                # Coleta métricas atuais
                metrics = await self.collect_task_metrics(task)
                
                # Analisa padrões de execução
                await self.analyze_execution_patterns(task, metrics)
                
                # Verifica gargalos
                bottlenecks = self.identify_performance_bottlenecks(
                    execution_trend=self.calculate_trend(task.task_metrics['execution_time']),
                    resource_usage={
                        'memory': {'peak': max(task.task_metrics['memory_usage']) if task.task_metrics['memory_usage'] else 0},
                        'cpu': {'peak': max(task.task_metrics['cpu_usage']) if task.task_metrics['cpu_usage'] else 0}
                    },
                    wait_times=task.task_metrics['dependency_wait_time']
                )
                
                if bottlenecks:
                    self.logger.warning(f"Gargalos detectados para {task.name}: {bottlenecks}")
                
                # Identifica padrões de sucesso
                success_patterns = self.identify_success_patterns(task)
                if success_patterns:
                    self.logger.info(f"Padrões de sucesso identificados para {task.name}: {success_patterns}")
                
                # Calcula eficiência atual
                efficiency_score = self.calculate_efficiency_score({
                    'memory_efficiency': self.analyze_memory_efficiency(task.task_metrics['memory_usage']),
                    'cpu_utilization': self.analyze_cpu_utilization(task.task_metrics['cpu_usage'])
                })
                
                self.logger.info(f"Eficiência atual da tarefa {task.name}: {efficiency_score:.2f}")
                
                # Atualiza métricas e aguarda próximo ciclo
                await asyncio.sleep(5)
            
            # Análise final após conclusão
            elapsed_time = (datetime.now() - start_time).total_seconds()
            self.logger.info(f"Monitoramento finalizado para {task.name}. Tempo total: {elapsed_time:.2f}s")
            
            await self.perform_task_retrospective(task)
            
        except Exception as e:
            self.logger.error(f"Erro no monitoramento da tarefa {task.name}: {str(e)}")
            task.status = "failed"
            raise

# Exemplo de uso
async def example_workflow():
    orchestrator = DevelopmentOrchestrator()
    await orchestrator.start()
    
    try:
        # Cria tarefas de desenvolvimento
        setup_task = await orchestrator.create_development_task(
            name="Setup Ambiente",
            task_type="environment_setup"
        )
        
        build_task = await orchestrator.create_development_task(
            name="Build Componentes",
            task_type="component_build",
            dependencies=[setup_task.id]
        )
        
        # Submete tarefas para execução
        await orchestrator.submit_task(setup_task)
        await orchestrator.submit_task(build_task)
        
        # Monitora progresso
        await orchestrator.monitor_task_progress(build_task.id)
        
    finally:
        await orchestrator.stop()


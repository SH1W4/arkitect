"""
ARKITECT Main Controller
Sistema principal de controle e integração do ARKITECT
"""

import asyncio
import logging
import os
import signal
import sys
from pathlib import Path
from typing import List, Dict
from prometheus_client import start_http_server, Counter, Gauge
import uvicorn
from uvicorn.config import Config
from uvicorn.server import Server
from fastapi import FastAPI

from task_mesh.config import TaskMeshConfig, DEFAULT_CONFIG
from task_mesh.executor import TaskExecutor
from integration_layer.connector import create_connector
from development_orchestration.orchestrator import DevelopmentOrchestrator

# Configuração de logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(sys.stdout),
        logging.FileHandler('arkitect.log')
    ]
)

logger = logging.getLogger("arkitect.main")

# Métricas Prometheus
TASK_COUNTER = Counter('arkitect_tasks_total', 'Total de tarefas processadas')
ACTIVE_TASKS = Gauge('arkitect_active_tasks', 'Tarefas atualmente em execução')
SYSTEM_HEALTH = Gauge('arkitect_system_health', 'Status de saúde do sistema')

# API FastAPI para health check e métricas adicionais
app = FastAPI()

@app.get("/health")
async def health_check():
    return {"status": "healthy", "mode": "standalone" if os.getenv("STANDALONE_MODE") == "true" else "integrated"}

class ArkitectController:
    """Controlador principal do sistema ARKITECT"""
    
    def __init__(self):
        self.logger = logger
        self.dev_orchestrator = DevelopmentOrchestrator()
        self.running = False
        self.shutdown_event = asyncio.Event()
        self.tasks = []
        self.server = None
        
    async def start(self):
        """Inicia todos os componentes do sistema"""
        self.logger.info("Iniciando sistema ARKITECT")
        self.running = True
        SYSTEM_HEALTH.set(1)  # Sistema saudável
        
        # Configura evento de shutdown
        self._shutdown_event = asyncio.Event()
        
        try:
            # Inicia servidor de métricas
            metrics_port = int(os.getenv("METRICS_PORT", "9090"))
            start_http_server(metrics_port)
            self.logger.info(f"Servidor de métricas iniciado na porta {metrics_port}")
            
            # Configura e inicia servidor API
            config = Config(
                app=app,
                host="localhost",
                port=8000,
                log_level="info",
                loop="none"  # Usa o loop de eventos atual
            )
            
            server = Server(config=config)
            self.server = server
            
            # Inicia o servidor em uma task separada
            api_task = asyncio.create_task(server.serve())
            self.tasks.append(api_task)
            
            # Inicia orquestrador de desenvolvimento
            await self.dev_orchestrator.start()
            
            # Configura tarefas iniciais do sistema
            await self.setup_initial_tasks()
            
            # Mantém sistema rodando até receber sinal de shutdown
            await self._shutdown_event.wait()
            
        except Exception as e:
            self.logger.error(f"Erro durante execução do sistema: {str(e)}")
            SYSTEM_HEALTH.set(0)  # Sistema com problema
            raise
            
    async def stop(self):
        """Para todos os componentes do sistema de forma controlada"""
        self.logger.info("Iniciando graceful shutdown do ARKITECT")
        self.running = False
        SYSTEM_HEALTH.set(0)  # Sistema em processo de shutdown
        
        # Sinaliza evento de shutdown
        self.shutdown_event.set()
        
        # Para o servidor Uvicorn
        if self.server:
            self.logger.info("Parando servidor API...")
            self.server.should_exit = True
            await asyncio.sleep(0.1)  # Permite que o servidor processe o sinal de saída
        
        # Para todos os componentes de forma ordenada
        await self.dev_orchestrator.stop()
        
        # Cancela tarefas pendentes
        for task in self.tasks:
            if not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass
        
        self.logger.info("Sistema ARKITECT finalizado com sucesso")
        
    async def setup_initial_tasks(self):
        """Configura tarefas iniciais do sistema"""
        tasks = [
            {
                "name": "Análise de Código",
                "type": "code_analysis",
                "priority": 1
            },
            {
                "name": "Execução de Testes",
                "type": "test_execution",
                "priority": 2
            },
            {
                "name": "Build de Componentes",
                "type": "build",
                "priority": 3
            }
        ]
        
        for task_info in tasks:
            task = await self.dev_orchestrator.create_development_task(
                name=task_info["name"],
                task_type=task_info["type"],
                priority=task_info["priority"]
            )
            await self.dev_orchestrator.submit_task(task)
            
class GracefulExit(SystemExit):
    """Exceção customizada para graceful shutdown"""
    pass

def handle_shutdown(signum, frame):
    """Handler para sinais de shutdown"""
    logger.info(f"Sinal de shutdown recebido: {signum}")
    raise GracefulExit()

async def run_arkitect():
    """Executa o ARKITECT com tratamento adequado de shutdown"""
    controller = ArkitectController()
    
    try:
        # Configura handlers de sinal
        for sig in (signal.SIGTERM, signal.SIGINT):
            signal.signal(sig, handle_shutdown)
        
        # Inicia o controlador
        await controller.start()
        
    except GracefulExit:
        logger.info("Iniciando graceful shutdown")
    except Exception as e:
        logger.error(f"Erro fatal: {str(e)}")
        raise
    finally:
        logger.info("Finalizando sistema...")
        await controller.stop()

async def main():
    """Função principal de execução"""
    try:
        await run_arkitect()
    except GracefulExit:
        logger.info("Sistema finalizado corretamente")
    except Exception as e:
        logger.error(f"Erro durante execução: {str(e)}")
        raise

if __name__ == "__main__":
    try:
        if os.name == 'nt':  # Windows
            # Configura evento de shutdown para Windows
            asyncio.set_event_loop_policy(asyncio.WindowsProactorEventLoopPolicy())
        
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("\nSistema finalizado pelo usuário")
    except GracefulExit:
        logger.info("Sistema finalizado via sinal")
    except Exception as e:
        logger.error(f"Erro fatal durante execução: {str(e)}")
        sys.exit(1)
    finally:
        logger.info("Sistema ARKITECT encerrado")


"""
ARKITECT Integration Connector
Sistema de integração entre ARKITECT e EON-Framework
"""

import aiohttp
import asyncio
from typing import Dict, Any, Optional
import logging
from dataclasses import dataclass
import json

@dataclass
class ServiceConfig:
    """Configuração de serviço para integração"""
    name: str
    host: str
    port: int
    timeout: int
    endpoints: Dict[str, str]

class IntegrationConnector:
    """Conector principal para integração entre sistemas"""
    
    def __init__(self, service_config: ServiceConfig):
        self.config = service_config
        self.logger = logging.getLogger(f"integration.{service_config.name}")
        self.session: Optional[aiohttp.ClientSession] = None
        self.base_url = f"http://{self.config.host}:{self.config.port}"
        
    async def connect(self):
        """Estabelece conexão com o serviço"""
        if not self.session:
            self.session = aiohttp.ClientSession(
                timeout=aiohttp.ClientTimeout(total=self.config.timeout)
            )
        self.logger.info(f"Conectado ao serviço: {self.config.name}")
        
    async def disconnect(self):
        """Encerra conexão com o serviço"""
        if self.session:
            await self.session.close()
            self.session = None
        self.logger.info(f"Desconectado do serviço: {self.config.name}")
        
    async def request(self, 
                     endpoint: str, 
                     method: str = "GET", 
                     data: Optional[Dict] = None,
                     params: Optional[Dict] = None) -> Dict:
        """Realiza requisição ao serviço"""
        if not self.session:
            await self.connect()
            
        url = f"{self.base_url}{endpoint}"
        
        try:
            async with self.session.request(
                method=method,
                url=url,
                json=data,
                params=params
            ) as response:
                response.raise_for_status()
                return await response.json()
                
        except aiohttp.ClientError as e:
            self.logger.error(f"Erro na requisição para {url}: {str(e)}")
            raise
            
class EONFrameworkConnector(IntegrationConnector):
    """Conector específico para integração com EON-Framework"""
    
    async def submit_task(self, task_data: Dict[str, Any]) -> Dict[str, Any]:
        """Submete uma tarefa para o EON-Framework"""
        return await self.request(
            endpoint="/tasks/submit",
            method="POST",
            data=task_data
        )
        
    async def get_task_status(self, task_id: str) -> Dict[str, Any]:
        """Obtém status de uma tarefa no EON-Framework"""
        return await self.request(
            endpoint=f"/tasks/{task_id}/status",
            method="GET"
        )
        
    async def sync_development_state(self, state_data: Dict[str, Any]) -> Dict[str, Any]:
        """Sincroniza estado de desenvolvimento com EON-Framework"""
        return await self.request(
            endpoint="/development/sync",
            method="POST",
            data=state_data
        )

# Configuração padrão para EON-Framework
EON_CONFIG = ServiceConfig(
    name="eon_framework",
    host="localhost",
    port=8001,
    timeout=30,
    endpoints={
        "submit_task": "/tasks/submit",
        "task_status": "/tasks/{task_id}/status",
        "sync_state": "/development/sync"
    }
)

# Factory para criar conectores
def create_connector(service_type: str, config: Optional[ServiceConfig] = None) -> IntegrationConnector:
    """Cria um conector baseado no tipo de serviço"""
    if service_type == "eon_framework":
        return EONFrameworkConnector(config or EON_CONFIG)
    else:
        raise ValueError(f"Tipo de serviço não suportado: {service_type}")


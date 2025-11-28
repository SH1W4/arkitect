"""
API Server Module - Server Management

This module provides the APIServer class for managing
the FastAPI server lifecycle.
"""

import uvicorn
import logging
from typing import Optional

logger = logging.getLogger(__name__)


class APIServer:
    """
    API Server wrapper for managing FastAPI application.
    
    Provides convenient methods for starting and stopping
    the API server with proper configuration.
    """
    
    def __init__(
        self,
        app=None,
        host: str = "0.0.0.0",
        port: int = 8000,
        reload: bool = False
    ):
        """
        Initialize the API Server.
        
        Args:
            app: FastAPI application instance
            host: Host to bind to
            port: Port to listen on
            reload: Enable auto-reload for development
        """
        self.app = app
        self.host = host
        self.port = port
        self.reload = reload
        self.server: Optional[uvicorn.Server] = None
        logger.info(f"APIServer initialized on {host}:{port}")
    
    def run(self, **kwargs) -> None:
        """
        Run the API server.
        
        Args:
            **kwargs: Additional uvicorn configuration options
        """
        if not self.app:
            from .app import create_app
            self.app = create_app()
        
        config = {
            "app": self.app,
            "host": self.host,
            "port": self.port,
            "reload": self.reload,
            **kwargs
        }
        
        logger.info(f"Starting API server on {self.host}:{self.port}")
        uvicorn.run(**config)
    
    async def start(self) -> None:
        """Start the server asynchronously."""
        if not self.app:
            from .app import create_app
            self.app = create_app()
        
        config = uvicorn.Config(
            app=self.app,
            host=self.host,
            port=self.port,
            reload=self.reload
        )
        
        self.server = uvicorn.Server(config)
        logger.info(f"Starting async API server on {self.host}:{self.port}")
        await self.server.serve()
    
    async def stop(self) -> None:
        """Stop the server asynchronously."""
        if self.server:
            logger.info("Stopping API server")
            self.server.should_exit = True

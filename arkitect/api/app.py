"""
API App Module - FastAPI Application Factory

This module provides the application factory for creating
configured FastAPI instances.
"""

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
import logging

logger = logging.getLogger(__name__)


def create_app(
    title: str = "Arkitect API",
    description: str = "High-Performance Agent Orchestration Platform",
    version: str = "0.2.0"
) -> FastAPI:
    """
    Create and configure a FastAPI application.
    
    Args:
        title: API title
        description: API description
        version: API version
        
    Returns:
        Configured FastAPI application
    """
    app = FastAPI(
        title=title,
        description=description,
        version=version,
        docs_url="/docs",
        redoc_url="/redoc"
    )
    
    # Add CORS middleware
    app.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],  # Configure appropriately for production
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )
    
    # Health check endpoint
    @app.get("/health")
    async def health_check():
        """Health check endpoint."""
        return {
            "status": "healthy",
            "service": "arkitect-api",
            "version": version
        }
    
    # Root endpoint
    @app.get("/")
    async def root():
        """Root endpoint with API information."""
        return {
            "name": title,
            "version": version,
            "docs": "/docs",
            "health": "/health"
        }
    
    logger.info(f"FastAPI app created: {title} v{version}")
    
    return app

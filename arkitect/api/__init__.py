"""Módulo API do ARKITECT.

FastAPI-based REST API for ARKITECT platform.
"""

from arkitect.api.app import create_app
from arkitect.api.server import APIServer

__all__ = [
    "create_app",
    "APIServer",
]


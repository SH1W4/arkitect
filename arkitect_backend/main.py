from fastapi import FastAPI, Request, HTTPException
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
import logging
from typing import Dict, Any

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

# Initialize FastAPI application
app = FastAPI(
    title="ARKITECT Backend",
    description="Symbiotic AI Core System with Quantum-based Memory and Consciousness Evolution",
    version="1.0.0"
)

# CORS Middleware Configuration
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Update with specific origins in production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global error handler
@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    logger.error(f"Global error handler caught: {exc}", exc_info=True)
    return JSONResponse(
        status_code=500,
        content={"detail": "Internal server error", "error_type": str(type(exc).__name__)}
    )

# Health check endpoint
@app.get("/health")
async def health_check() -> Dict[str, str]:
    return {"status": "operational", "service": "ARKITECT Backend"}

# Import and include routers
from arkitect_backend.routers import (
    symbiotic_core,
    quantum_memory,
    consciousness,
    eon_framework,
    task_management
)

# Register routers
app.include_router(symbiotic_core.router, prefix="/api/v1/core", tags=["Symbiotic Core"])
app.include_router(quantum_memory.router, prefix="/api/v1/memory", tags=["Quantum Memory"])
app.include_router(consciousness.router, prefix="/api/v1/consciousness", tags=["Consciousness"])
app.include_router(eon_framework.router, prefix="/api/v1/eon", tags=["EON Framework"])
app.include_router(task_management.router, prefix="/api/v1/tasks", tags=["Task Management"])

if __name__ == "__main__":
    import uvicorn
    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=True)


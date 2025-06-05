import uvicorn
from arkitect_backend.config import settings

def main():
    """Entry point for running the ARKITECT Backend server."""
    uvicorn.run(
        "arkitect_backend.main:app",
        host=settings.ARKITECT_HOST,
        port=settings.ARKITECT_PORT,
        reload=True,
        workers=settings.ARKITECT_WORKERS,
        log_level=settings.ARKITECT_LOG_LEVEL.lower()
    )

if __name__ == "__main__":
    main()


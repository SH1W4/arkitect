from fastapi import Request, status
from fastapi.responses import JSONResponse
import logging
from typing import Dict, Any

logger = logging.getLogger(__name__)

class ErrorHandler:
    @staticmethod
    async def handle_exception(request: Request, exc: Exception) -> JSONResponse:
        error_id = "ERR-" + str(hash(str(exc)))[:8]
        logger.error(f"Error {error_id}: {str(exc)}", exc_info=True)
        
        error_response = {
            "error_id": error_id,
            "detail": str(exc),
            "type": exc.__class__.__name__
        }
        
        if hasattr(exc, "status_code"):
            status_code = exc.status_code
        else:
            status_code = status.HTTP_500_INTERNAL_SERVER_ERROR
            
        return JSONResponse(
            status_code=status_code,
            content=error_response
        )

class ARKITECTException(Exception):
    def __init__(self, detail: str, status_code: int = status.HTTP_500_INTERNAL_SERVER_ERROR):
        self.detail = detail
        self.status_code = status_code
        super().__init__(self.detail)


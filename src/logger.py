import sys
from loguru import logger
from pathlib import Path
from .config import LOG_LEVEL

# Configure logging
log_file = Path(__file__).parent.parent / 'logs' / 'app.log'

logger.configure(
    handlers=[
        {"sink": sys.stdout, "level": LOG_LEVEL},
        {"sink": str(log_file), "level": LOG_LEVEL, "rotation": "10 MB"},
    ]
)


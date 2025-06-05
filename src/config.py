import os
from dotenv import load_dotenv
from pathlib import Path

# Load environment variables
env_path = Path(__file__).parent.parent / 'config' / '.env'
load_dotenv(env_path)

# Application settings
ENV = os.getenv('ENV', 'development')
DEBUG = os.getenv('DEBUG', 'true').lower() == 'true'
LOG_LEVEL = os.getenv('LOG_LEVEL', 'INFO')

# Redis settings
REDIS_URL = os.getenv('REDIS_URL', 'redis://localhost:6379/0')

# Security
SECRET_KEY = os.getenv('SECRET_KEY', 'your-secret-key-here')


from pydantic_settings import BaseSettings
from typing import List, Optional
from functools import lru_cache
from pydantic import ConfigDict

class Settings(BaseSettings):
    # Core Settings
    PROJECT_NAME: str = "ARKITECT Backend"
    VERSION: str = "1.0.0"
    API_V1_STR: str = "/api/v1"
    
    # ARKITECT Configuration
    ARKITECT_ENV: str = "development"
    ARKITECT_API_KEY: str = "dev_eon_framework_key"
    ARKITECT_DB_URL: str = "sqlite:///./arkitect.db"
    ARKITECT_LOG_LEVEL: str = "DEBUG"
    ARKITECT_PORT: int = 8000
    ARKITECT_HOST: str = "localhost"
    ARKITECT_WORKERS: int = 4
    
    # Environment
    ENVIRONMENT: str = "development"
    DEBUG: bool = True
    
    # Security
    SECRET_KEY: str = "your-secret-key-here"
    ALGORITHM: str = "HS256"
    ACCESS_TOKEN_EXPIRE_MINUTES: int = 30
    ENABLE_SSL: bool = False
    SSL_CERT_PATH: Optional[str] = None
    SSL_KEY_PATH: Optional[str] = None
    
    # CORS and Access
    BACKEND_CORS_ORIGINS: List[str] = ["*"]
    ALLOWED_HOSTS: str = "localhost,127.0.0.1"
    CORS_ORIGINS: str = "http://localhost:*"
    
    # Redis Configuration
    REDIS_URL: str = "redis://localhost:6379"
    REDIS_PASSWORD: str = "development_password"
    REDIS_DB: int = 0
    
    # EON Framework Integration
    EON_FRAMEWORK_REQUIRED: bool = False
    EON_FRAMEWORK_HOST: str = "localhost"
    EON_FRAMEWORK_PORT: int = 8001
    EON_FRAMEWORK_TIMEOUT: int = 30
    EON_FRAMEWORK_API_KEY: str = "dev_eon_framework_key"
    STANDALONE_MODE: bool = True
    
    # Task Management
    TASK_MESH_WORKERS: int = 4
    TASK_MESH_QUEUE_SIZE: int = 1000
    TASK_MESH_RETRY_LIMIT: int = 3
    
    # Monitoring and Metrics
    ENABLE_METRICS: bool = True
    METRICS_PORT: int = 9090
    
    # Logging Configuration
    LOG_FORMAT: str = "json"
    LOG_FILE: str = "logs/arkitect.log"
    LOG_ROTATION: str = "1d"
    LOG_RETENTION: str = "30d"
    ENABLE_CONSOLE_LOG: bool = True
    
    # Development Settings
    DEVELOPMENT_MODE: bool = True
    ENABLE_DEBUG_ENDPOINTS: bool = True
    ENABLE_PERFORMANCE_LOGGING: bool = True
    ENABLE_DETAILED_ERRORS: bool = True
    
    # Local Development Settings
    LOCAL_PROCESSING: bool = True
    MOCK_EON_RESPONSES: bool = True
    ENABLE_LOCAL_CACHE: bool = True
    MAX_LOCAL_TASKS: int = 100
    LOCAL_TASK_TIMEOUT: int = 300
    
    model_config = ConfigDict(
        case_sensitive=True,
        env_file=".env",
        env_file_encoding="utf-8",
        extra="allow"
    )

@lru_cache()
def get_settings() -> Settings:
    return Settings()

settings = get_settings()

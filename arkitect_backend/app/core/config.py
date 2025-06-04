"""
Módulo de configuração e validação de ambiente do ARKITECT.
Implementa verificação robusta de variáveis de ambiente e logging detalhado.
"""

import os
import logging
from pathlib import Path
from typing import Dict, List, Optional
from dotenv import load_dotenv

# Configuração do logger
logger = logging.getLogger("arkitect.startup")

class EnvValidator:
    """Gerenciador de validação de variáveis de ambiente."""
    
    def __init__(self):
        self.required_vars: List[str] = [
            "ARKITECT_API_KEY",
            "ARKITECT_DB_URL",
            "ARKITECT_ENV",
            "ARKITECT_LOG_LEVEL"
        ]
        self.optional_vars: Dict[str, str] = {
            "ARKITECT_PORT": "8000",
            "ARKITECT_HOST": "localhost",
            "ARKITECT_WORKERS": "4"
        }
        self.env_values: Dict[str, str] = {}

    def load_env_file(self) -> bool:
        """
        Carrega e valida o arquivo .env
        
        Returns:
            bool: True se arquivo foi carregado com sucesso
        """
        try:
            logger.info("Verificando Arquivo .env...")
            env_path = Path(".env")
            
            if not env_path.exists():
                logger.warning(".env não encontrado. Criando arquivo com configurações padrão...")
                self._create_default_env()
                return True
                
            load_dotenv()
            logger.info(".env carregado com sucesso")
            return True
            
        except Exception as e:
            logger.error(f"Erro ao carregar .env: {str(e)}")
            return False

    def validate_env(self) -> bool:
        """
        Valida todas as variáveis de ambiente necessárias.
        
        Returns:
            bool: True se todas as variáveis estão válidas
        """
        valid = True
        
        # Verifica variáveis obrigatórias
        for var in self.required_vars:
            value = os.getenv(var)
            if not value:
                logger.error(f"Variável de ambiente obrigatória não encontrada: {var}")
                valid = False
            else:
                self.env_values[var] = value
                logger.debug(f"Variável {var} validada com sucesso")

        # Configura variáveis opcionais com valores padrão
        for var, default in self.optional_vars.items():
            value = os.getenv(var, default)
            self.env_values[var] = value
            logger.debug(f"Variável opcional {var} configurada: {value}")

        return valid

    def _create_default_env(self):
        """Cria um arquivo .env com configurações padrão."""
        default_content = """
# Configurações do ARKITECT
ARKITECT_ENV=development
ARKITECT_API_KEY=dev_key_change_me
ARKITECT_DB_URL=sqlite:///./arkitect.db
ARKITECT_LOG_LEVEL=INFO

# Configurações opcionais
ARKITECT_PORT=8000
ARKITECT_HOST=localhost
ARKITECT_WORKERS=4
"""
        with open(".env", "w") as f:
            f.write(default_content.strip())
        load_dotenv()
        logger.info("Arquivo .env padrão criado")

    def get_env_value(self, key: str) -> Optional[str]:
        """
        Retorna o valor de uma variável de ambiente.
        
        Args:
            key: Nome da variável
            
        Returns:
            Optional[str]: Valor da variável ou None se não existir
        """
        return self.env_values.get(key)

class Settings:
    """Gerenciador de configurações do ARKITECT."""
    
    def __init__(self):
        self.env_validator = EnvValidator()
        if not self.env_validator.load_env_file():
            raise RuntimeError("Falha ao carregar configurações de ambiente")
            
        if not self.env_validator.validate_env():
            raise RuntimeError("Configurações de ambiente inválidas")
            
        # Configura atributos baseados nas variáveis de ambiente
        self.api_key = self.env_validator.get_env_value("ARKITECT_API_KEY")
        self.db_url = self.env_validator.get_env_value("ARKITECT_DB_URL")
        self.environment = self.env_validator.get_env_value("ARKITECT_ENV")
        self.log_level = self.env_validator.get_env_value("ARKITECT_LOG_LEVEL")
        self.port = int(self.env_validator.get_env_value("ARKITECT_PORT"))
        self.host = self.env_validator.get_env_value("ARKITECT_HOST")
        self.workers = int(self.env_validator.get_env_value("ARKITECT_WORKERS"))

    @property
    def is_development(self) -> bool:
        """Verifica se está em ambiente de desenvolvimento."""
        return self.environment.lower() == "development"

# Instância global das configurações
settings = Settings()

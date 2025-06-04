"""
ARKITECT Startup Script
Script de inicialização com verificação de dependências
"""

import sys
import asyncio
import subprocess
from pathlib import Path
import logging
import os
import traceback
from typing import List, Tuple
import socket
import importlib
from pkg_resources import parse_version
from dotenv import load_dotenv

# Configuração avançada de logging
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
    handlers=[
        logging.StreamHandler(),
        logging.FileHandler('startup.log')
    ]
)
logger = logging.getLogger("arkitect.startup")

class StartupCheck:
    """Verifica e prepara ambiente para execução"""
    
    @staticmethod
    def check_python_version() -> bool:
        """Verifica versão do Python"""
        version = sys.version_info
        if version.major < 3 or (version.major == 3 and version.minor < 10):
            logger.error("Python 3.10+ é necessário")
            return False
        return True
        
    @staticmethod
    def check_dependencies() -> bool:
        """Verifica se dependências estão instaladas e suas versões mínimas exigidas"""
        required = {
            "aiohttp": "3.8.0",
            "fastapi": "0.68.0",
            "uvicorn": "0.15.0",
            "redis": "4.0.0",
            "pydantic": "1.8.0",
            "prometheus_client": "0.12.0"
        }
        
        missing = []
        version_mismatch = []
        
        for package, min_version in required.items():
            try:
                module = importlib.import_module(package)
                version = getattr(module, '__version__', 'unknown')
                logger.debug(f"Verificando {package}: versão instalada {version}, mínima {min_version}")
                
                if version != 'unknown' and parse_version(version) < parse_version(min_version):
                    version_mismatch.append(f"{package} (atual: {version}, necessária: {min_version})")
            except ImportError as e:
                logger.debug(f"Erro ao importar {package}: {str(e)}")
                missing.append(package)
            except Exception as e:
                logger.warning(f"Erro ao verificar versão de {package}: {str(e)}")
                
        if missing or version_mismatch:
            if missing:
                logger.error(f"Dependências faltando: {', '.join(missing)}")
            if version_mismatch:
                logger.error(f"Versões incompatíveis: {', '.join(version_mismatch)}")
            return False
        
        logger.info("Todas as dependências verificadas com sucesso")
        return True
        
    @staticmethod
    def check_redis() -> bool:
        """Verifica conexão com Redis"""
        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(1)
            s.connect(('localhost', 6379))
            s.close()
            return True
        except:
            logger.error("Redis não está acessível")
            return False
            
    @staticmethod
    def check_eon_framework() -> bool:
        """Verifica conexão com EON-Framework"""
        if os.getenv('STANDALONE_MODE', 'false').lower() == 'true':
            if os.getenv('EON_FRAMEWORK_REQUIRED', 'true').lower() == 'false':
                logger.info("Modo standalone ativo, ignorando verificação do EON-Framework")
                return True

        try:
            s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            s.settimeout(1)
            s.connect(('localhost', 8001))
            s.close()
            return True
        except:
            if os.getenv('STANDALONE_MODE', 'false').lower() == 'true':
                logger.warning("EON-Framework não acessível, continuando em modo standalone")
                return True
            logger.error("EON-Framework não está acessível e modo standalone não está ativo")
            return False
            
    @staticmethod
    def check_env_file() -> bool:
        """Verifica arquivo .env e carrega variáveis"""
        if not Path('.env').exists():
            if Path('.env.example').exists():
                logger.warning("Arquivo .env não encontrado, copiando de .env.example")
                import shutil
                shutil.copy('.env.example', '.env')
            else:
                logger.error("Arquivo .env não encontrado")
                return False
        
        # Carrega variáveis de ambiente
        load_dotenv()
        
        # Verifica modo standalone
        if os.getenv('STANDALONE_MODE', 'false').lower() == 'true':
            logger.info("Iniciando em modo standalone")
            if os.getenv('DEVELOPMENT_MODE', 'false').lower() != 'true':
                logger.warning("Modo standalone recomendado apenas para desenvolvimento")
        
        return True
        
    @staticmethod
    def create_directories() -> None:
        """Cria diretórios necessários"""
        dirs = ['logs', 'data', 'temp']
        for d in dirs:
            Path(d).mkdir(exist_ok=True)
            
    def run_checks(self) -> bool:
        """Executa todas as verificações"""
        checks = [
            (self.check_python_version, "Versão do Python"),
            (self.check_dependencies, "Dependências"),
            (self.check_redis, "Conexão Redis"),
            (self.check_eon_framework, "Conexão EON-Framework"),
            (self.check_env_file, "Arquivo .env")
        ]
        
        success = True
        for check, name in checks:
            logger.info(f"Verificando {name}...")
            if not check():
                success = False
                
        if success:
            self.create_directories()
            logger.info("Todas as verificações passaram com sucesso")
        else:
            logger.error("Algumas verificações falharam")
            
        return success

async def main():
    """Função principal de inicialização"""
    # Carrega variáveis de ambiente antes das verificações
    load_dotenv()
    
    # Configura modo de execução
    if os.getenv('STANDALONE_MODE', 'false').lower() == 'true':
        logger.info("Iniciando ARKITECT em modo standalone")
        if os.getenv('DEVELOPMENT_MODE', 'false').lower() == 'true':
            logger.info("Modo de desenvolvimento ativo")
    
    checker = StartupCheck()
    if not checker.run_checks():
        sys.exit(1)
        
    logger.info("Iniciando ARKITECT...")
    try:
        logger.debug("Tentando importar e iniciar o módulo principal")
        from main import main as arkitect_main
        await arkitect_main()
    except ImportError as e:
        logger.error(f"Erro ao importar módulo principal: {str(e)}")
        logger.debug(f"Traceback completo:\n{traceback.format_exc()}")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Erro ao iniciar ARKITECT: {str(e)}")
        logger.error("Detalhes do erro:")
        logger.error(traceback.format_exc())
        sys.exit(1)

def verify_system_requirements():
    """Verifica requisitos adicionais do sistema"""
    try:
        # Verifica permissões de diretório
        dirs_to_check = ['logs', 'data', 'temp']
        for dir_name in dirs_to_check:
            path = Path(dir_name)
            if path.exists():
                if not os.access(path, os.W_OK):
                    logger.error(f"Sem permissão de escrita no diretório {dir_name}")
                    return False
            else:
                try:
                    path.mkdir(exist_ok=True)
                except Exception as e:
                    logger.error(f"Erro ao criar diretório {dir_name}: {str(e)}")
                    return False
        
        # Verifica portas necessárias
        ports_to_check = [8000, 9090]  # API e métricas
        for port in ports_to_check:
            sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            try:
                sock.bind(('localhost', port))
                sock.close()
            except socket.error:
                logger.error(f"Porta {port} já está em uso")
                return False
        
        return True
    except Exception as e:
        logger.error(f"Erro ao verificar requisitos do sistema: {str(e)}")
        return False

if __name__ == "__main__":
    try:
        # Verifica requisitos do sistema antes de iniciar
        if not verify_system_requirements():
            logger.error("Verificação de requisitos do sistema falhou")
            sys.exit(1)
            
        asyncio.run(main())
    except KeyboardInterrupt:
        logger.info("\nInterrompido pelo usuário")
    except Exception as e:
        logger.error(f"Erro fatal durante execução: {str(e)}")
        logger.error("Detalhes do erro:")
        logger.error(traceback.format_exc())
        sys.exit(1)


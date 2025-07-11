#!/usr/bin/env python3
"""
Script de inicialização do OrchestratorAPI
Configura ambiente e inicia servidor com opções avançadas
"""

import os
import sys
import subprocess
import argparse
from pathlib import Path

def setup_environment():
    """Configura ambiente de desenvolvimento"""
    print("🔧 Setting up environment...")
    
    # Verifica Python version
    if sys.version_info < (3, 9):
        print("❌ Python 3.9+ is required")
        sys.exit(1)
    
    # Cria diretórios necessários
    dirs = ["logs", "data", "config"]
    for dir_name in dirs:
        Path(dir_name).mkdir(exist_ok=True)
    
    # Verifica arquivo .env
    if not Path(".env").exists():
        print("⚠️  .env file not found, copying from .env.example")
        if Path(".env.example").exists():
            subprocess.run(["cp", ".env.example", ".env"])
        else:
            print("❌ .env.example not found, creating minimal .env")
            with open(".env", "w") as f:
                f.write("DEBUG=true\nLOG_LEVEL=DEBUG\n")
    
    print("✅ Environment setup complete")

def install_dependencies():
    """Instala dependências Python"""
    print("📦 Installing dependencies...")
    
    try:
        subprocess.run([sys.executable, "-m", "pip", "install", "-r", "requirements.txt"], 
                      check=True, capture_output=True)
        print("✅ Dependencies installed")
    except subprocess.CalledProcessError as e:
        print(f"❌ Failed to install dependencies: {e}")
        sys.exit(1)

def start_redis():
    """Inicia Redis se não estiver rodando"""
    print("🔴 Checking Redis...")
    
    try:
        # Verifica se Redis está rodando
        result = subprocess.run(["redis-cli", "ping"], 
                               capture_output=True, text=True, timeout=5)
        if result.returncode == 0 and "PONG" in result.stdout:
            print("✅ Redis is running")
            return
    except (subprocess.TimeoutExpired, FileNotFoundError):
        pass
    
    print("⚠️  Redis not detected, trying to start...")
    
    try:
        # Tenta iniciar Redis
        subprocess.Popen(["redis-server"], 
                        stdout=subprocess.DEVNULL, 
                        stderr=subprocess.DEVNULL)
        print("🔴 Redis started")
    except FileNotFoundError:
        print("⚠️  Redis not installed, API will run without Redis")

def start_api(args):
    """Inicia a API"""
    print("🚀 Starting ARKITECT OrchestratorAPI...")
    
    # Importa e roda a aplicação
    import uvicorn
    from main import app, global_settings
    
    # Configurações do uvicorn
    config = {
        "app": "main:app" if args.reload else app,
        "host": args.host or global_settings.host,
        "port": args.port or global_settings.port,
        "reload": args.reload,
        "log_level": args.log_level.lower() if args.log_level else global_settings.log_level.lower(),
        "access_log": args.access_log,
        "workers": 1 if args.reload else (args.workers or global_settings.workers)
    }
    
    if args.ssl_cert and args.ssl_key:
        config["ssl_certfile"] = args.ssl_cert
        config["ssl_keyfile"] = args.ssl_key
        print("🔒 SSL enabled")
    
    print(f"🌐 Server will start on {config['host']}:{config['port']}")
    print(f"📚 Docs available at: http://{config['host']}:{config['port']}/docs")
    
    uvicorn.run(**config)

def main():
    """Função principal"""
    parser = argparse.ArgumentParser(description="ARKITECT OrchestratorAPI Starter")
    
    # Argumentos do servidor
    parser.add_argument("--host", help="Host to bind", default=None)
    parser.add_argument("--port", type=int, help="Port to bind", default=None)
    parser.add_argument("--workers", type=int, help="Number of workers", default=None)
    parser.add_argument("--reload", action="store_true", help="Enable auto-reload")
    parser.add_argument("--log-level", help="Log level", 
                       choices=["DEBUG", "INFO", "WARNING", "ERROR", "CRITICAL"])
    parser.add_argument("--access-log", action="store_true", help="Enable access log")
    
    # SSL
    parser.add_argument("--ssl-cert", help="SSL certificate file")
    parser.add_argument("--ssl-key", help="SSL key file")
    
    # Opções de setup
    parser.add_argument("--skip-setup", action="store_true", help="Skip environment setup")
    parser.add_argument("--skip-deps", action="store_true", help="Skip dependency installation")
    parser.add_argument("--skip-redis", action="store_true", help="Skip Redis check")
    
    # Comandos especiais
    parser.add_argument("--test", action="store_true", help="Run API tests")
    parser.add_argument("--docker", action="store_true", help="Start with Docker Compose")
    
    args = parser.parse_args()
    
    print("🏗️  ARKITECT OrchestratorAPI Starter")
    print("=" * 40)
    
    # Comando especial: Docker
    if args.docker:
        print("🐳 Starting with Docker Compose...")
        try:
            subprocess.run(["docker-compose", "up", "-d"], check=True)
            print("✅ Docker services started")
            print("🌐 API available at: http://localhost:8000")
            print("📊 Grafana available at: http://localhost:3000")
            print("🔍 Prometheus available at: http://localhost:9090")
        except subprocess.CalledProcessError as e:
            print(f"❌ Docker Compose failed: {e}")
        return
    
    # Comando especial: Teste
    if args.test:
        print("🧪 Running API tests...")
        try:
            subprocess.run([sys.executable, "test_api.py"], check=True)
        except subprocess.CalledProcessError as e:
            print(f"❌ Tests failed: {e}")
        return
    
    # Setup padrão
    if not args.skip_setup:
        setup_environment()
    
    if not args.skip_deps:
        install_dependencies()
    
    if not args.skip_redis:
        start_redis()
    
    # Inicia API
    try:
        start_api(args)
    except KeyboardInterrupt:
        print("\n👋 Shutting down...")
    except Exception as e:
        print(f"❌ Failed to start API: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()


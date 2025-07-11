#!/usr/bin/env python3
"""
Quick Start Script para ARKITECT OrchestratorAPI
Script simplificado que verifica dependências e inicia a API
"""

import sys
import subprocess
import importlib.util

def check_and_install_package(package_name, import_name=None):
    """Verifica se um pacote está instalado e instala se necessário"""
    if import_name is None:
        import_name = package_name
    
    try:
        if import_name == "pydantic_settings":
            from pydantic_settings import BaseSettings
        elif import_name == "fastapi":
            import fastapi
        elif import_name == "uvicorn":
            import uvicorn
        elif import_name == "psutil":
            import psutil
        elif import_name == "pydantic":
            import pydantic
        else:
            __import__(import_name)
        print(f"✅ {package_name} já instalado")
        return True
    except ImportError:
        print(f"⚠️  {package_name} não encontrado, tentando instalar...")
        try:
            subprocess.check_call([sys.executable, "-m", "pip", "install", package_name])
            print(f"✅ {package_name} instalado com sucesso")
            return True
        except subprocess.CalledProcessError:
            print(f"❌ Falha ao instalar {package_name}")
            return False

def check_dependencies():
    """Verifica e instala dependências básicas"""
    print("🔍 Verificando dependências...")
    
    required_packages = [
        ("fastapi", "fastapi"),
        ("uvicorn", "uvicorn"),
        ("pydantic", "pydantic"),
        ("pydantic-settings", "pydantic_settings"),
        ("psutil", "psutil")
    ]
    
    all_installed = True
    for package, import_name in required_packages:
        if not check_and_install_package(package, import_name):
            all_installed = False
    
    return all_installed

def run_simple_api():
    """Executa uma versão simplificada da API para teste"""
    print("🚀 Iniciando ARKITECT OrchestratorAPI (modo simplificado)...")
    
    try:
        # Import dinâmico para evitar erros antes da instalação
        import uvicorn
        from fastapi import FastAPI
        from fastapi.responses import JSONResponse
        from datetime import datetime
        import psutil
        
        # Cria app simplificado
        app = FastAPI(
            title="ARKITECT OrchestratorAPI",
            description="API de Orquestração Simbiótica - Demo",
            version="1.0.0"
        )
        
        @app.get("/")
        async def root():
            return {
                "message": "ARKITECT OrchestratorAPI está funcionando!",
                "timestamp": datetime.utcnow().isoformat(),
                "version": "1.0.0",
                "status": "operational"
            }
        
        @app.get("/health")
        async def health():
            return {
                "status": "healthy",
                "timestamp": datetime.utcnow().isoformat(),
                "version": "1.0.0",
                "environment": "quickstart"
            }
        
        @app.get("/metrics/simple")
        async def simple_metrics():
            try:
                cpu_percent = psutil.cpu_percent(interval=1)
                memory = psutil.virtual_memory()
                
                return {
                    "cpu_usage": cpu_percent,
                    "memory_usage": memory.percent,
                    "timestamp": datetime.utcnow().isoformat()
                }
            except Exception as e:
                return {"error": str(e), "timestamp": datetime.utcnow().isoformat()}
        
        print("✅ API criada com sucesso")
        print("🌐 Acesse: http://localhost:8000")
        print("📚 Docs: http://localhost:8000/docs")
        print("🩺 Health: http://localhost:8000/health")
        print("📊 Métricas: http://localhost:8000/metrics/simple")
        print("\n⏹️  Pressione Ctrl+C para parar")
        
        # Inicia servidor
        uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")
        
    except KeyboardInterrupt:
        print("\n👋 API finalizada pelo usuário")
    except Exception as e:
        print(f"❌ Erro ao iniciar API: {e}")
        return False
    
    return True

def main():
    """Função principal"""
    print("🏗️  ARKITECT OrchestratorAPI - Quick Start")
    print("=" * 45)
    
    # Verifica Python version
    if sys.version_info < (3, 8):
        print("❌ Python 3.8+ é necessário")
        sys.exit(1)
    
    print(f"✅ Python {sys.version_info.major}.{sys.version_info.minor}.{sys.version_info.micro}")
    
    # Verifica dependências
    if not check_dependencies():
        print("❌ Falha ao instalar dependências necessárias")
        print("💡 Tente executar: pip install fastapi uvicorn pydantic pydantic-settings psutil")
        sys.exit(1)
    
    print("\n🎯 Todas as dependências verificadas!")
    print("\n" + "=" * 45)
    
    # Executa API
    if not run_simple_api():
        sys.exit(1)

if __name__ == "__main__":
    main()


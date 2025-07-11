#!/usr/bin/env python3
"""
Script de Setup do GitTaskAgent

Script automatizado para configurar e testar o GitTaskAgent
em um ambiente de desenvolvimento.
"""

import os
import sys
import subprocess
import json
from pathlib import Path
import tempfile
import shutil

def check_dependencies():
    """Verifica dependências necessárias"""
    print("🔍 Verificando dependências...")
    
    # Verificar Python
    try:
        python_version = sys.version_info
        if python_version.major < 3 or python_version.minor < 8:
            print("❌ Python 3.8+ necessário")
            return False
        print(f"✓ Python {python_version.major}.{python_version.minor} OK")
    except:
        print("❌ Erro ao verificar Python")
        return False
    
    # Verificar Git
    try:
        result = subprocess.run(["git", "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            print(f"✓ {result.stdout.strip()} OK")
        else:
            print("❌ Git não encontrado")
            return False
    except:
        print("❌ Git não instalado")
        return False
    
    return True

def install_python_packages():
    """Instala pacotes Python necessários"""
    print("\n📦 Instalando pacotes Python...")
    
    packages = [
        "fastapi",
        "uvicorn[standard]",
        "pytest",
        "pytest-asyncio",
        "pydantic",
    ]
    
    for package in packages:
        print(f"  Instalando {package}...")
        try:
            subprocess.run([sys.executable, "-m", "pip", "install", package], 
                         check=True, capture_output=True)
            print(f"  ✓ {package} instalado")
        except subprocess.CalledProcessError as e:
            print(f"  ❌ Erro ao instalar {package}: {e}")
            return False
    
    return True

def setup_test_repository():
    """Configura repositório de teste"""
    print("\n📁 Configurando repositório de teste...")
    
    # Criar diretório de teste
    test_dir = Path.cwd() / "git_agent_test"
    
    if test_dir.exists():
        print(f"  Removendo repositório existente: {test_dir}")
        shutil.rmtree(test_dir)
    
    test_dir.mkdir()
    
    try:
        # Inicializar Git
        subprocess.run(["git", "init"], cwd=test_dir, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "GitAgent Test"], 
                      cwd=test_dir, check=True)
        subprocess.run(["git", "config", "user.email", "test@arkitect.dev"], 
                      cwd=test_dir, check=True)
        
        # Criar arquivo inicial
        readme = test_dir / "README.md"
        readme.write_text("# GitTaskAgent Test Repository\n\nRepositório para testes do GitTaskAgent.")
        
        # Commit inicial
        subprocess.run(["git", "add", "README.md"], cwd=test_dir, check=True)
        subprocess.run(["git", "commit", "-m", "chore: initial test setup"], 
                      cwd=test_dir, check=True)
        
        print(f"  ✓ Repositório de teste criado em: {test_dir}")
        return test_dir
        
    except subprocess.CalledProcessError as e:
        print(f"  ❌ Erro ao configurar repositório: {e}")
        return None

def test_basic_functionality(test_repo):
    """Testa funcionalidade básica do agente"""
    print("\n🧪 Testando funcionalidade básica...")
    
    try:
        # Importar agente
        sys.path.insert(0, str(Path.cwd()))
        from agents.git_task_agent import GitTaskAgent, TaskNode
        
        # Testar inicialização
        agent = GitTaskAgent(repository_path=str(test_repo))
        print("  ✓ GitTaskAgent inicializado")
        
        # Testar TaskNode
        task = TaskNode(
            id="test_001",
            type="git",
            description="Testar funcionalidade básica",
            scope="test"
        )
        print("  ✓ TaskNode criado")
        
        # Testar análise de tipo
        commit_type = agent._analyze_commit_type("Implementar nova feature")
        print(f"  ✓ Análise de tipo: {commit_type.value}")
        
        # Testar geração de mensagem
        message = agent._generate_conventional_message(
            commit_type, "Implementar nova feature", "test"
        )
        print(f"  ✓ Mensagem gerada: {message}")
        
        # Testar estado de consciência
        consciousness = agent.get_consciousness_state()
        print(f"  ✓ Consciência: evolução={consciousness['evolution_level']:.3f}")
        
        return True
        
    except Exception as e:
        print(f"  ❌ Erro no teste: {e}")
        import traceback
        traceback.print_exc()
        return False

def run_dry_run_test(test_repo):
    """Executa teste de dry-run"""
    print("\n🎨 Executando teste de dry-run...")
    
    try:
        import asyncio
        from agents.git_task_agent import GitTaskAgent, TaskNode
        
        async def dry_run_test():
            agent = GitTaskAgent(repository_path=str(test_repo))
            
            # Criar arquivo de teste
            test_file = test_repo / "test_feature.py"
            test_file.write_text("# Arquivo de teste\ndef test_function():\n    return True")
            
            # Criar TaskNode
            task = TaskNode(
                id="dry_run_test",
                type="git",
                description="Adicionar arquivo de teste",
                files=["test_feature.py"],
                scope="test"
            )
            
            # Observar e executar dry-run
            operations = await agent.observe_task_nodes([task])
            result = await agent.execute_operation(operations[0], dry_run=True)
            
            return result
        
        result = asyncio.run(dry_run_test())
        
        if result:
            print("  ✓ Dry-run executado com sucesso")
            return True
        else:
            print("  ❌ Falha no dry-run")
            return False
            
    except Exception as e:
        print(f"  ❌ Erro no teste de dry-run: {e}")
        return False

def create_config_file():
    """Cria arquivo de configuração"""
    print("\n⚙️ Criando arquivo de configuração...")
    
    config = {
        "git_agent": {
            "default_repository": ".",
            "auto_push": False,
            "dry_run_first": True,
            "consciousness_export_interval": 10,
            "max_files_per_operation": 10,
            "quantum_coherence_threshold": 0.7
        },
        "conventional_commits": {
            "default_scope": None,
            "enforce_scope": False,
            "max_description_length": 50,
            "allow_breaking_changes": True
        },
        "logging": {
            "level": "INFO",
            "format": "%(asctime)s - [GitTask] - %(levelname)s - %(message)s",
            "file": "git_agent.log"
        }
    }
    
    config_file = Path.cwd() / "git_agent_config.json"
    
    try:
        with open(config_file, 'w', encoding='utf-8') as f:
            json.dump(config, f, indent=2, ensure_ascii=False)
        
        print(f"  ✓ Configuração salva em: {config_file}")
        return config_file
        
    except Exception as e:
        print(f"  ❌ Erro ao criar configuração: {e}")
        return None

def create_startup_script():
    """Cria script de inicialização"""
    print("\n🚀 Criando script de inicialização...")
    
    startup_script = '''
#!/usr/bin/env python3
"""
Script de Inicialização do GitTaskAgent

Use este script para iniciar rapidamente o GitTaskAgent
com configurações padrão.
"""

import asyncio
import json
from pathlib import Path
from agents.git_task_agent import GitTaskAgent, TaskNode

def load_config():
    """Carrega configuração"""
    config_file = Path("git_agent_config.json")
    if config_file.exists():
        with open(config_file) as f:
            return json.load(f)
    return {}

async def main():
    """Função principal"""
    print("🤖 Iniciando GitTaskAgent...")
    
    # Carregar configuração
    config = load_config()
    git_config = config.get("git_agent", {})
    
    # Inicializar agente
    repo_path = git_config.get("default_repository", ".")
    agent = GitTaskAgent(repository_path=repo_path)
    
    print(f"✓ Agente inicializado para: {Path(repo_path).resolve()}")
    
    # Mostrar estado
    consciousness = agent.get_consciousness_state()
    print(f"Consciência: Evolução={consciousness['evolution_level']:.3f}, "
          f"Coerência={consciousness['quantum_coherence']:.3f}")
    
    # Exemplo de uso
    print("\n📚 Exemplo de uso:")
    print("from agents.git_task_agent import GitTaskAgent, TaskNode")
    print("agent = GitTaskAgent()")
    print("task = TaskNode(id='exemplo', type='git', description='Implementar feature')")
    print("operations = await agent.observe_task_nodes([task])")
    print("result = await agent.execute_operation(operations[0], dry_run=True)")
    
    return agent

if __name__ == "__main__":
    asyncio.run(main())
'''
    
    script_file = Path.cwd() / "start_git_agent.py"
    
    try:
        with open(script_file, 'w', encoding='utf-8') as f:
            f.write(startup_script)
        
        # Tornar executável no Unix
        if os.name != 'nt':
            os.chmod(script_file, 0o755)
        
        print(f"  ✓ Script criado em: {script_file}")
        return script_file
        
    except Exception as e:
        print(f"  ❌ Erro ao criar script: {e}")
        return None

def run_api_test():
    """Testa API REST"""
    print("\n🌐 Testando API REST...")
    
    try:
        # Verificar se FastAPI funciona
        from fastapi import FastAPI
        from agents.git_task_api import router
        
        app = FastAPI()
        app.include_router(router)
        
        print("  ✓ API REST configurada")
        print("  💡 Para testar a API, execute:")
        print("     uvicorn agents.git_task_api:router --reload --port 8000")
        
        return True
        
    except Exception as e:
        print(f"  ❌ Erro na configuração da API: {e}")
        return False

def generate_summary_report(test_repo, config_file, script_file):
    """Gera relatório de resumo"""
    print("\n📈 RELATÓRIO DE SETUP")
    print("=" * 50)
    
    print(f"📁 Repositório de teste: {test_repo}")
    print(f"⚙️ Arquivo de configuração: {config_file}")
    print(f"🚀 Script de inicialização: {script_file}")
    
    print("\n📚 PRÓXIMOS PASSOS:")
    print("1. Execute os testes: pytest tests/test_git_task_agent.py -v")
    print("2. Teste a demonstração: python examples/git_agent_demo.py")
    print("3. Inicie a API: uvicorn agents.git_task_api:router --reload")
    print("4. Use o script: python start_git_agent.py")
    
    print("\n📜 DOCUMENTAÇÃO:")
    print("- Guia completo: docs/git_task_agent_guide.md")
    print("- Exemplos: examples/git_agent_demo.py")
    print("- Testes: tests/test_git_task_agent.py")
    
    print("\n✨ GitTaskAgent configurado com sucesso!")

def main():
    """Função principal do setup"""
    print("🚀 SETUP DO GITTASKAGENT")
    print("=" * 40)
    print("Configurador automático do sistema de commits simbiótico")
    
    # 1. Verificar dependências
    if not check_dependencies():
        print("\n❌ Setup cancelado devido a dependências")
        return 1
    
    # 2. Instalar pacotes
    if not install_python_packages():
        print("\n❌ Setup cancelado devido a erro na instalação")
        return 1
    
    # 3. Configurar repositório de teste
    test_repo = setup_test_repository()
    if not test_repo:
        print("\n❌ Setup cancelado devido a erro no repositório")
        return 1
    
    # 4. Testar funcionalidade básica
    if not test_basic_functionality(test_repo):
        print("\n❌ Falha nos testes básicos")
        return 1
    
    # 5. Teste de dry-run
    if not run_dry_run_test(test_repo):
        print("\n❌ Falha no teste de dry-run")
        return 1
    
    # 6. Criar configuração
    config_file = create_config_file()
    if not config_file:
        print("\n❌ Falha ao criar configuração")
        return 1
    
    # 7. Criar script de inicialização
    script_file = create_startup_script()
    if not script_file:
        print("\n❌ Falha ao criar script")
        return 1
    
    # 8. Testar API
    run_api_test()
    
    # 9. Gerar relatório
    generate_summary_report(test_repo, config_file, script_file)
    
    return 0

if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)


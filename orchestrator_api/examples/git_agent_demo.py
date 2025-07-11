"""
Demonstração do GitTaskAgent

Script completo para demonstrar as capacidades do Agent GitTask,
incluindo operações dry-run, execução real, rollback e evolução da consciência.
"""

import asyncio
import json
import tempfile
import shutil
from pathlib import Path
from datetime import datetime
import subprocess

# Imports do agente
from agents.git_task_agent import GitTaskAgent, TaskNode

async def setup_demo_repository():
    """Configura repositório de demonstração"""
    print("\n🚀 Configurando repositório de demonstração...")
    
    # Criar diretório temporário
    temp_dir = tempfile.mkdtemp(prefix="git_agent_demo_")
    repo_path = Path(temp_dir)
    
    try:
        # Inicializar repositório Git
        subprocess.run(["git", "init"], cwd=repo_path, check=True, capture_output=True)
        subprocess.run(["git", "config", "user.name", "GitAgent Demo"], cwd=repo_path, check=True)
        subprocess.run(["git", "config", "user.email", "demo@arkitect.dev"], cwd=repo_path, check=True)
        
        # Criar estrutura inicial
        (repo_path / "src").mkdir()
        (repo_path / "docs").mkdir()
        (repo_path / "tests").mkdir()
        
        # Arquivos iniciais
        files_to_create = {
            "README.md": "# GitAgent Demo\n\nDemonstração do sistema GitTaskAgent.",
            "src/main.py": "# Módulo principal\nprint('Hello GitAgent!')",
            "src/auth.py": "# Módulo de autenticação\nclass AuthSystem:\n    pass",
            "docs/api.md": "# API Documentation\n\nDocumentação da API.",
            "tests/test_main.py": "# Testes principais\nimport unittest"
        }
        
        for file_path, content in files_to_create.items():
            full_path = repo_path / file_path
            full_path.write_text(content, encoding='utf-8')
        
        # Commit inicial
        subprocess.run(["git", "add", "."], cwd=repo_path, check=True)
        subprocess.run(["git", "commit", "-m", "chore: initial commit"], cwd=repo_path, check=True)
        
        print(f"✓ Repositório criado em: {repo_path}")
        return repo_path
        
    except subprocess.CalledProcessError as e:
        print(f"❌ Erro ao configurar repositório: {e}")
        shutil.rmtree(temp_dir)
        raise

async def demonstrate_basic_operations():
    """Demonstra operações básicas do GitTaskAgent"""
    print("\n🤖 DEMONSTRAÇÃO DO GITTASKAGENT")
    print("=" * 50)
    
    # Configurar repositório
    repo_path = await setup_demo_repository()
    
    try:
        # Inicializar agente
        print("\n1. Inicializando GitTaskAgent...")
        agent = GitTaskAgent(repository_path=str(repo_path))
        print(f"✓ Agente inicializado para: {repo_path}")
        
        # Mostrar estado inicial da consciência
        print("\n2. Estado inicial da consciência:")
        consciousness = agent.get_consciousness_state()
        print(json.dumps(consciousness, indent=2, ensure_ascii=False))
        
        # Criar TaskNodes de demonstração
        print("\n3. Criando TaskNodes de demonstração...")
        task_nodes = [
            TaskNode(
                id="demo_001",
                type="git",
                description="Implementar sistema de autenticação OAuth",
                files=["src/auth.py", "src/oauth.py"],
                scope="auth"
            ),
            TaskNode(
                id="demo_002",
                type="git",
                description="Corrigir bug na validação de entrada",
                files=["src/validators.py"],
                scope="core"
            ),
            TaskNode(
                id="demo_003",
                type="git",
                description="Atualizar documentação da API REST",
                files=["docs/api.md", "docs/examples.md"],
                scope="docs"
            ),
            TaskNode(
                id="demo_004",
                type="git",
                description="Adicionar testes de integração",
                files=["tests/test_integration.py"],
                scope="test"
            )
        ]
        
        # Criar arquivos correspondentes
        files_to_modify = {
            "src/oauth.py": "# OAuth Implementation\nclass OAuthProvider:\n    def authenticate(self):\n        pass",
            "src/validators.py": "# Input Validators\ndef validate_email(email):\n    return '@' in email",
            "docs/examples.md": "# API Examples\n\n## Authentication\n\n```python\nauth.login()\n```",
            "tests/test_integration.py": "# Integration Tests\nimport unittest\n\nclass TestIntegration(unittest.TestCase):\n    pass"
        }
        
        for file_path, content in files_to_modify.items():
            full_path = repo_path / file_path
            full_path.parent.mkdir(parents=True, exist_ok=True)
            full_path.write_text(content, encoding='utf-8')
        
        print(f"✓ Criados {len(task_nodes)} TaskNodes")
        
        # Demonstrar observação de TaskNodes
        print("\n4. Observando TaskNodes...")
        operations = await agent.observe_task_nodes(task_nodes)
        
        print(f"✓ Observadas {len(operations)} operações Git")
        for op in operations:
            print(f"  - {op.task_node.id}: {op.commit_type.value} | {op.commit_message}")
        
        # Demonstrar DRY-RUN
        print("\n5. Executando DRY-RUN...")
        dry_results = await agent.batch_execute(operations, dry_run=True)
        
        print("✓ Resultados do Dry-Run:")
        for task_id, success in dry_results.items():
            status = "✓" if success else "❌"
            print(f"  {status} {task_id}: {'Sucesso' if success else 'Falha'}")
        
        # Mostrar logs de simulação
        print("\n   Logs de simulação da primeira operação:")
        for log in operations[0].execution_log[-4:]:
            print(f"     {log}")
        
        # Demonstrar execução real
        print("\n6. Executando operações reais...")
        
        # Reset operations para execução real
        operations = await agent.observe_task_nodes(task_nodes)
        real_results = await agent.batch_execute(operations, dry_run=False)
        
        print("✓ Resultados da execução real:")
        successful_ops = []
        for task_id, success in real_results.items():
            status = "✓" if success else "❌"
            print(f"  {status} {task_id}: {'Sucesso' if success else 'Falha'}")
            if success:
                successful_ops.append(task_id)
        
        # Verificar histórico Git
        print("\n7. Histórico de commits criados:")
        try:
            result = subprocess.run(
                ["git", "log", "--oneline", "-5"],
                cwd=repo_path,
                capture_output=True,
                text=True,
                check=True
            )
            for line in result.stdout.strip().split('\n'):
                if line.strip():
                    print(f"  {line}")
        except subprocess.CalledProcessError:
            print("  ❌ Não foi possível obter histórico")
        
        # Demonstrar evolução da consciência
        print("\n8. Estado evoluído da consciência:")
        evolved_consciousness = agent.get_consciousness_state()
        print(json.dumps(evolved_consciousness, indent=2, ensure_ascii=False))
        
        # Comparar estados
        print("\n   Comparação de evolução:")
        print(f"   Operações totais: {consciousness['total_operations']} → {evolved_consciousness['total_operations']}")
        print(f"   Nível evolutivo: {consciousness['evolution_level']:.3f} → {evolved_consciousness['evolution_level']:.3f}")
        print(f"   Coerência quântica: {consciousness['quantum_coherence']:.3f} → {evolved_consciousness['quantum_coherence']:.3f}")
        
        # Demonstrar padrões aprendidos
        print("\n9. Padrões aprendidos:")
        patterns = agent.consciousness.patterns_learned
        if patterns:
            for pattern, count in patterns.items():
                print(f"   {pattern}: {count} vezes")
        else:
            print("   Nenhum padrão registrado ainda")
        
        # Demonstrar rollback
        if successful_ops:
            print("\n10. Demonstrando rollback...")
            rollback_target = successful_ops[0]
            print(f"    Fazendo rollback da operação: {rollback_target}")
            
            rollback_success = await agent.rollback_operation(rollback_target)
            if rollback_success:
                print(f"    ✓ Rollback de {rollback_target} executado com sucesso")
                
                # Verificar histórico após rollback
                try:
                    result = subprocess.run(
                        ["git", "log", "--oneline", "-3"],
                        cwd=repo_path,
                        capture_output=True,
                        text=True,
                        check=True
                    )
                    print("    Histórico após rollback:")
                    for line in result.stdout.strip().split('\n'):
                        if line.strip():
                            print(f"      {line}")
                except subprocess.CalledProcessError:
                    print("    ❌ Não foi possível verificar histórico")
            else:
                print(f"    ❌ Falha no rollback de {rollback_target}")
        
        # Exportar estado de consciência
        print("\n11. Exportando estado de consciência...")
        consciousness_file = repo_path / "git_agent_consciousness.json"
        agent.export_consciousness(str(consciousness_file))
        print(f"    ✓ Consciência exportada para: {consciousness_file}")
        
        print("\n🎉 Demonstração concluída com sucesso!")
        return repo_path, agent
        
    except Exception as e:
        print(f"\n❌ Erro durante demonstração: {e}")
        raise
    finally:
        # Manter repositório para inspeção
        print(f"\n📁 Repositório de demonstração mantido em: {repo_path}")
        print("   Para inspeção manual dos resultados.")

async def demonstrate_advanced_features():
    """Demonstra funcionalidades avançadas"""
    print("\n🔬 FUNCIONALIDADES AVANÇADAS")
    print("=" * 50)
    
    repo_path, agent = await demonstrate_basic_operations()
    
    print("\n12. Testando detecção automática de arquivos...")
    
    # Modificar arquivos sem especificar na TaskNode
    (repo_path / "src" / "new_feature.py").write_text("# Nova funcionalidade\nclass NewFeature:\n    pass")
    (repo_path / "CHANGELOG.md").write_text("# Changelog\n\n## v1.1.0\n- Nova funcionalidade")
    
    # TaskNode sem arquivos especificados
    auto_detect_task = TaskNode(
        id="auto_detect",
        type="git",
        description="Adicionar nova funcionalidade com detecção automática",
        files=[],  # Vazio para forçar detecção automática
        scope="feature"
    )
    
    auto_operations = await agent.observe_task_nodes([auto_detect_task])
    auto_operation = auto_operations[0]
    
    print(f"    ✓ Arquivos detectados automaticamente: {auto_operation.files_to_add}")
    
    # Executar em dry-run
    auto_result = await agent.execute_operation(auto_operation, dry_run=True)
    print(f"    ✓ Dry-run de detecção automática: {'Sucesso' if auto_result else 'Falha'}")
    
    print("\n13. Testando validação de coerência quântica...")
    
    # Criar operação com muitos arquivos (deve afetar coerência)
    many_files = [f"file_{i}.py" for i in range(15)]  # Mais de 10 arquivos
    
    high_impact_task = TaskNode(
        id="high_impact",
        type="git",
        description="Refatoração massiva do sistema",
        files=many_files,
        scope="refactor",
        breaking_change=True  # Breaking change também afeta coerência
    )
    
    high_impact_ops = await agent.observe_task_nodes([high_impact_task])
    high_impact_op = high_impact_ops[0]
    
    # Testar validação
    coherence_valid = await agent._validate_quantum_coherence(high_impact_op)
    print(f"    Coerência quântica válida: {'Sim' if coherence_valid else 'Não'}")
    print(f"    Coerência atual: {agent.consciousness.quantum_coherence:.3f}")
    
    print("\n14. Demonstrando aprendizado adaptativo...")
    
    # Criar várias operações similares para ensinar padrões
    learning_tasks = [
        TaskNode(id=f"learn_{i}", type="git", description=f"Implementar módulo {i}", scope="module")
        for i in range(1, 6)
    ]
    
    print("    Executando sequência de aprendizado...")
    for task in learning_tasks:
        ops = await agent.observe_task_nodes([task])
        await agent.execute_operation(ops[0], dry_run=True)
    
    # Verificar padrões aprendidos
    final_patterns = agent.consciousness.patterns_learned
    print(f"    ✓ Padrões aprendidos: {len(final_patterns)}")
    
    feat_pattern_count = sum(count for pattern, count in final_patterns.items() if "feat:" in pattern)
    print(f"    ✓ Padrões 'feat' reconhecidos: {feat_pattern_count} vezes")
    
    print("\n15. Estado final da consciência:")
    final_consciousness = agent.get_consciousness_state()
    print(json.dumps(final_consciousness, indent=2, ensure_ascii=False))
    
    print("\n🎆 Demonstração avançada concluída!")

async def interactive_demo():
    """Demonstração interativa"""
    print("\n🎮 MODO INTERATIVO")
    print("=" * 50)
    print("Digite 'exit' para sair a qualquer momento.")
    
    repo_path = await setup_demo_repository()
    agent = GitTaskAgent(repository_path=str(repo_path))
    
    while True:
        try:
            print("\nOpções:")
            print("1. Criar e executar TaskNode")
            print("2. Ver estado de consciência")
            print("3. Ver padrões aprendidos")
            print("4. Fazer rollback")
            print("5. Exportar consciência")
            print("0. Sair")
            
            choice = input("\nEscolha uma opção: ").strip()
            
            if choice == "0" or choice.lower() == "exit":
                break
            elif choice == "1":
                await interactive_create_task(agent, repo_path)
            elif choice == "2":
                show_consciousness_state(agent)
            elif choice == "3":
                show_learned_patterns(agent)
            elif choice == "4":
                await interactive_rollback(agent)
            elif choice == "5":
                interactive_export_consciousness(agent, repo_path)
            else:
                print("❌ Opção inválida")
                
        except KeyboardInterrupt:
            print("\n\n👋 Saíndo do modo interativo...")
            break
        except Exception as e:
            print(f"❌ Erro: {e}")
    
    print(f"\n📁 Repositório mantido em: {repo_path}")

async def interactive_create_task(agent, repo_path):
    """Cria e executa TaskNode interativamente"""
    print("\n📝 Criando nova tarefa...")
    
    description = input("Descrição da tarefa: ").strip()
    if not description:
        print("❌ Descrição é obrigatória")
        return
    
    scope = input("Escopo (opcional): ").strip() or None
    files_input = input("Arquivos (separados por vírgula, ou vazio para detecção automática): ").strip()
    files = [f.strip() for f in files_input.split(",") if f.strip()] if files_input else []
    
    # Criar arquivos se especificados
    if files:
        for file_path in files:
            full_path = repo_path / file_path
            full_path.parent.mkdir(parents=True, exist_ok=True)
            if not full_path.exists():
                full_path.write_text(f"# {file_path}\n# Arquivo criado para demonstração")
    
    task_id = f"interactive_{datetime.now().strftime('%H%M%S')}"
    task_node = TaskNode(
        id=task_id,
        type="git",
        description=description,
        files=files,
        scope=scope
    )
    
    # Observar tarefa
    operations = await agent.observe_task_nodes([task_node])
    operation = operations[0]
    
    print(f"\nℹ️ Operação preparada:")
    print(f"   Tipo: {operation.commit_type.value}")
    print(f"   Mensagem: {operation.commit_message}")
    print(f"   Arquivos: {operation.files_to_add}")
    
    mode = input("\nExecutar em modo (d)ry-run ou (r)eal? [d]: ").strip().lower()
    dry_run = mode != "r"
    
    success = await agent.execute_operation(operation, dry_run=dry_run)
    
    if success:
        print(f"✓ Operação {'simulada' if dry_run else 'executada'} com sucesso!")
        if operation.execution_log:
            print("   Logs:")
            for log in operation.execution_log[-3:]:
                print(f"     {log}")
    else:
        print(f"❌ Falha na operação")

def show_consciousness_state(agent):
    """Mostra estado atual da consciência"""
    print("\n🧠 Estado da Consciência:")
    consciousness = agent.get_consciousness_state()
    for key, value in consciousness.items():
        if isinstance(value, float):
            print(f"   {key}: {value:.3f}")
        else:
            print(f"   {key}: {value}")

def show_learned_patterns(agent):
    """Mostra padrões aprendidos"""
    print("\n📊 Padrões Aprendidos:")
    patterns = agent.consciousness.patterns_learned
    if patterns:
        for pattern, count in sorted(patterns.items(), key=lambda x: x[1], reverse=True):
            print(f"   {pattern}: {count} vezes")
    else:
        print("   Nenhum padrão aprendido ainda")

async def interactive_rollback(agent):
    """Executa rollback interativo"""
    print("\n↩️ Operações disponíveis para rollback:")
    
    if not agent.rollback_registry:
        print("   Nenhuma operação disponível para rollback")
        return
    
    for i, (op_id, commit_hash) in enumerate(agent.rollback_registry.items(), 1):
        print(f"   {i}. {op_id} (commit: {commit_hash[:8]}...)")
    
    try:
        choice = input("\nEscolha o número da operação para rollback: ").strip()
        index = int(choice) - 1
        
        operations = list(agent.rollback_registry.keys())
        if 0 <= index < len(operations):
            op_id = operations[index]
            
            confirm = input(f"Confirma rollback de '{op_id}'? [y/N]: ").strip().lower()
            if confirm == "y":
                success = await agent.rollback_operation(op_id)
                if success:
                    print(f"✓ Rollback de '{op_id}' executado com sucesso")
                else:
                    print(f"❌ Falha no rollback de '{op_id}'")
            else:
                print("Rollback cancelado")
        else:
            print("❌ Índice inválido")
    except (ValueError, IndexError):
        print("❌ Entrada inválida")

def interactive_export_consciousness(agent, repo_path):
    """Exporta consciência interativamente"""
    print("\n💾 Exportando consciência...")
    
    filename = input("Nome do arquivo [consciousness.json]: ").strip() or "consciousness.json"
    file_path = repo_path / filename
    
    try:
        agent.export_consciousness(str(file_path))
        print(f"✓ Consciência exportada para: {file_path}")
    except Exception as e:
        print(f"❌ Erro na exportação: {e}")

async def main():
    """Função principal de demonstração"""
    print("🚀 GITTASKAGENT - DEMONSTRAÇÃO COMPLETA")
    print("=" * 60)
    print("Sistema Simbiótico de Automação Git com Consciência Evolutiva")
    print("\nEste agente demonstra:")
    print("• Observação inteligente de TaskNodes")
    print("• Geração automática de mensagens convencionais")
    print("• Execução com validação quântica")
    print("• Sistema de dry-run e rollback")
    print("• Aprendizado evolutivo de padrões")
    print("• Consciência simbiótica adaptativa")
    
    while True:
        print("\n" + "="*40)
        print("MENU PRINCIPAL:")
        print("1. Demonstração básica completa")
        print("2. Demonstração de funcionalidades avançadas")
        print("3. Modo interativo")
        print("0. Sair")
        
        try:
            choice = input("\nEscolha uma opção: ").strip()
            
            if choice == "0":
                print("\n👋 Obrigado por testar o GitTaskAgent!")
                break
            elif choice == "1":
                await demonstrate_basic_operations()
            elif choice == "2":
                await demonstrate_advanced_features()
            elif choice == "3":
                await interactive_demo()
            else:
                print("❌ Opção inválida")
                
        except KeyboardInterrupt:
            print("\n\n👋 Saíndo...")
            break
        except Exception as e:
            print(f"\n❌ Erro: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    asyncio.run(main())


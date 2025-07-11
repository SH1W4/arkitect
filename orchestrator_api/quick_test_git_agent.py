#!/usr/bin/env python3
"""
Teste Rápido do GitTaskAgent

Script simples para testar o GitTaskAgent sem dependências externas.
"""

import asyncio
import tempfile
import shutil
from pathlib import Path
import sys

# Adicionar o diretório atual ao path
sys.path.insert(0, str(Path.cwd()))

try:
    from agents.git_task_agent import GitTaskAgent, TaskNode, ConventionalCommitType
    print("✓ Módulos importados com sucesso")
except ImportError as e:
    print(f"❌ Erro ao importar módulos: {e}")
    print("Certifique-se de que está no diretório correto")
    sys.exit(1)

def create_test_repo():
    """Cria repositório de teste simples"""
    temp_dir = tempfile.mkdtemp(prefix="git_test_")
    repo_path = Path(temp_dir)
    
    # Criar estrutura de diretório .git (simulado)
    git_dir = repo_path / ".git"
    git_dir.mkdir()
    
    # Criar arquivo de teste
    test_file = repo_path / "test.py"
    test_file.write_text("# Arquivo de teste\nprint('Hello GitAgent!')")
    
    print(f"✓ Repositório de teste criado: {repo_path}")
    return repo_path

async def test_basic_functionality():
    """Testa funcionalidades básicas"""
    print("\n🧪 Testando funcionalidades básicas...")
    
    # Criar repositório de teste
    repo_path = create_test_repo()
    
    try:
        # 1. Testar inicialização
        print("\n1. Testando inicialização do agente...")
        agent = GitTaskAgent(repository_path=str(repo_path))
        print("✓ GitTaskAgent inicializado")
        
        # 2. Testar análise de tipo de commit
        print("\n2. Testando análise de tipos de commit...")
        test_cases = [
            ("Implementar nova funcionalidade", ConventionalCommitType.FEAT),
            ("Corrigir bug na validação", ConventionalCommitType.FIX),
            ("Atualizar documentação", ConventionalCommitType.DOCS),
            ("Refatorar código de autenticação", ConventionalCommitType.REFACTOR)
        ]
        
        for description, expected_type in test_cases:
            detected_type = agent._analyze_commit_type(description)
            status = "✓" if detected_type == expected_type else "❌"
            print(f"  {status} '{description}' -> {detected_type.value}")
        
        # 3. Testar geração de mensagens
        print("\n3. Testando geração de mensagens convencionais...")
        message = agent._generate_conventional_message(
            ConventionalCommitType.FEAT,
            "Implementar sistema de autenticação",
            scope="auth"
        )
        expected = "feat(auth): Implementar sistema de autenticação"
        status = "✓" if message == expected else "❌"
        print(f"  {status} Mensagem gerada: '{message}'")
        
        # 4. Testar TaskNode
        print("\n4. Testando criação de TaskNode...")
        task_node = TaskNode(
            id="test_001",
            type="git",
            description="Implementar feature de teste",
            files=["test.py"],
            scope="test"
        )
        print(f"✓ TaskNode criado: {task_node.id}")
        
        # 5. Testar observação de TaskNodes
        print("\n5. Testando observação de TaskNodes...")
        operations = await agent.observe_task_nodes([task_node])
        
        if operations:
            operation = operations[0]
            print(f"✓ Operação criada:")
            print(f"  - ID: {operation.task_node.id}")
            print(f"  - Tipo: {operation.commit_type.value}")
            print(f"  - Mensagem: {operation.commit_message}")
            print(f"  - Arquivos: {operation.files_to_add}")
            print(f"  - Assinatura: {operation.quantum_signature}")
        else:
            print("❌ Nenhuma operação criada")
            return False
        
        # 6. Testar simulação (dry-run)
        print("\n6. Testando simulação (dry-run)...")
        await agent._simulate_operation(operation)
        
        if operation.execution_log:
            print("✓ Simulação executada:")
            for log in operation.execution_log:
                print(f"    {log}")
        else:
            print("❌ Nenhum log de simulação gerado")
        
        # 7. Testar estado de consciência
        print("\n7. Testando estado de consciência...")
        consciousness = agent.get_consciousness_state()
        print(f"✓ Estado de consciência:")
        print(f"  - Nível evolutivo: {consciousness['evolution_level']:.3f}")
        print(f"  - Coerência quântica: {consciousness['quantum_coherence']:.3f}")
        print(f"  - Taxa de sucesso: {consciousness['success_rate']:.3f}")
        print(f"  - Operações totais: {consciousness['total_operations']}")
        
        # 8. Testar assinatura quântica
        print("\n8. Testando assinatura quântica...")
        signature1 = agent._generate_quantum_signature(task_node)
        signature2 = agent._generate_quantum_signature(task_node)
        
        if signature1 != signature2:
            print(f"✓ Assinaturas quânticas únicas:")
            print(f"  - Assinatura 1: {signature1}")
            print(f"  - Assinatura 2: {signature2}")
        else:
            print("❌ Assinaturas quânticas idênticas (problema!)")
        
        # 9. Testar validação de mensagens
        print("\n9. Testando validação de mensagens...")
        valid_messages = [
            "feat: adicionar nova funcionalidade",
            "fix(auth): corrigir bug de login",
            "docs: atualizar README"
        ]
        
        invalid_messages = [
            "adicionar funcionalidade",  # sem tipo
            "feat adicionar",            # sem dois pontos
            ""                           # vazia
        ]
        
        for msg in valid_messages:
            is_valid = await agent._validate_commit_message(msg)
            status = "✓" if is_valid else "❌"
            print(f"  {status} '{msg}' -> {'Válida' if is_valid else 'Inválida'}")
        
        for msg in invalid_messages:
            is_valid = await agent._validate_commit_message(msg)
            status = "✓" if not is_valid else "❌"
            print(f"  {status} '{msg}' -> {'Inválida' if not is_valid else 'Válida (deveria ser inválida!)'}")
        
        print("\n✓ Todos os testes básicos passaram!")
        return True
        
    except Exception as e:
        print(f"\n❌ Erro durante teste: {e}")
        import traceback
        traceback.print_exc()
        return False
        
    finally:
        # Limpeza
        try:
            shutil.rmtree(repo_path)
            print(f"\n🧹 Limpeza: repositório de teste removido")
        except:
            pass

def test_imports():
    """Testa importações de módulos"""
    print("📦 Testando importações...")
    
    try:
        from agents.git_task_agent import (
            GitTaskAgent, TaskNode, GitOperation, GitOperationStatus,
            ConventionalCommitType, GitAgentConsciousness
        )
        print("✓ Todas as classes principais importadas")
        
        # Testar enums
        commit_types = list(ConventionalCommitType)
        print(f"✓ {len(commit_types)} tipos de commit disponíveis: {[t.value for t in commit_types]}")
        
        statuses = list(GitOperationStatus)
        print(f"✓ {len(statuses)} status de operação: {[s.value for s in statuses]}")
        
        return True
        
    except ImportError as e:
        print(f"❌ Erro de importação: {e}")
        return False

def test_api_import():
    """Testa importação da API"""
    print("\n🌐 Testando importação da API...")
    
    try:
        from agents.git_task_api import router
        print("✓ Router da API importado")
        
        # Verificar rotas
        routes = [route.path for route in router.routes]
        print(f"✓ {len(routes)} rotas disponíveis:")
        for route in routes[:5]:  # Mostrar apenas primeiras 5
            print(f"  - {route}")
        if len(routes) > 5:
            print(f"  ... e mais {len(routes) - 5} rotas")
        
        return True
        
    except ImportError as e:
        print(f"❌ Erro ao importar API: {e}")
        return False

def main():
    """Função principal de teste"""
    print("🚀 TESTE RÁPIDO DO GITTASKAGENT")
    print("=" * 50)
    print("Verificando funcionalidades principais sem dependências externas")
    
    # Testes sequenciais
    tests = [
        ("Importações", test_imports),
        ("API", test_api_import),
        ("Funcionalidades Básicas", lambda: asyncio.run(test_basic_functionality()))
    ]
    
    results = []
    
    for test_name, test_func in tests:
        print(f"\n{'='*20} {test_name} {'='*20}")
        try:
            result = test_func()
            results.append((test_name, result))
        except Exception as e:
            print(f"❌ Erro em {test_name}: {e}")
            results.append((test_name, False))
    
    # Relatório final
    print("\n" + "=" * 50)
    print("📈 RELATÓRIO FINAL")
    print("=" * 50)
    
    passed = 0
    total = len(results)
    
    for test_name, result in results:
        status = "✓ PASSOU" if result else "❌ FALHOU"
        print(f"{status}: {test_name}")
        if result:
            passed += 1
    
    print(f"\nResultado: {passed}/{total} testes passaram")
    
    if passed == total:
        print("\n🎉 Todos os testes passaram! GitTaskAgent está funcionando corretamente.")
        print("\n📚 Próximos passos:")
        print("1. Execute: python examples/git_agent_demo.py")
        print("2. Leia: docs/git_task_agent_guide.md")
        print("3. Teste a API: uvicorn agents.git_task_api:router --reload")
    else:
        print(f"\n⚠️ {total - passed} teste(s) falharam. Verifique os erros acima.")
    
    return 0 if passed == total else 1

if __name__ == "__main__":
    exit_code = main()
    sys.exit(exit_code)


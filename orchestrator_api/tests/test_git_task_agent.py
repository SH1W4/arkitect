"""
Testes para o GitTaskAgent

Suite completa de testes para validação do módulo de commits automáticos,
incluindo testes unitários, de integração e simulações evolutivas.
"""

import pytest
import asyncio
import tempfile
import shutil
from pathlib import Path
from datetime import datetime
import json
from unittest.mock import Mock, patch, AsyncMock

# Imports do módulo sendo testado
from agents.git_task_agent import (
    GitTaskAgent, TaskNode, GitOperation, GitOperationStatus,
    ConventionalCommitType, GitAgentConsciousness
)

class TestGitTaskAgent:
    """Testes para a classe GitTaskAgent"""
    
    @pytest.fixture
    def temp_git_repo(self):
        """Cria repositório Git temporário para testes"""
        temp_dir = tempfile.mkdtemp()
        repo_path = Path(temp_dir)
        
        # Criar estrutura de repositório Git
        git_dir = repo_path / ".git"
        git_dir.mkdir()
        
        # Arquivo de teste
        test_file = repo_path / "test.py"
        test_file.write_text("# Arquivo de teste\nprint('Hello World')")
        
        yield repo_path
        
        # Limpeza
        shutil.rmtree(temp_dir)
    
    @pytest.fixture
    def agent(self, temp_git_repo):
        """Cria instância do GitTaskAgent para testes"""
        return GitTaskAgent(repository_path=str(temp_git_repo))
    
    @pytest.fixture
    def sample_task_nodes(self):
        """TaskNodes de exemplo para testes"""
        return [
            TaskNode(
                id="task_001",
                type="git",
                description="Implementar nova funcionalidade de autenticação",
                files=["auth.py", "models.py"],
                scope="auth"
            ),
            TaskNode(
                id="task_002",
                type="git",
                description="Corrigir bug na validação de dados",
                files=["validators.py"],
                scope="core"
            ),
            TaskNode(
                id="task_003",
                type="git",
                description="Atualizar documentação da API",
                files=["README.md", "docs/api.md"],
                scope="docs"
            )
        ]
    
    def test_init_valid_repository(self, temp_git_repo):
        """Testa inicialização com repositório válido"""
        agent = GitTaskAgent(repository_path=str(temp_git_repo))
        
        assert agent.repository_path == temp_git_repo
        assert isinstance(agent.consciousness, GitAgentConsciousness)
        assert agent.consciousness.evolution_level == 0.0
        assert agent.consciousness.quantum_coherence == 1.0
    
    def test_init_invalid_repository(self):
        """Testa inicialização com repositório inválido"""
        with tempfile.TemporaryDirectory() as temp_dir:
            # Diretório sem .git
            with pytest.raises(ValueError, match="não é um repositório Git"):
                GitTaskAgent(repository_path=temp_dir)
    
    def test_analyze_commit_type_feat(self, agent):
        """Testa identificação de commit tipo FEAT"""
        descriptions = [
            "Adicionar nova funcionalidade",
            "Implementar sistema de autenticação",
            "Create new feature",
            "Add user management"
        ]
        
        for desc in descriptions:
            commit_type = agent._analyze_commit_type(desc)
            assert commit_type == ConventionalCommitType.FEAT
    
    def test_analyze_commit_type_fix(self, agent):
        """Testa identificação de commit tipo FIX"""
        descriptions = [
            "Corrigir bug na validação",
            "Resolver problema de memória",
            "Fix authentication error",
            "Solve database issue"
        ]
        
        for desc in descriptions:
            commit_type = agent._analyze_commit_type(desc)
            assert commit_type == ConventionalCommitType.FIX
    
    def test_analyze_commit_type_docs(self, agent):
        """Testa identificação de commit tipo DOCS"""
        descriptions = [
            "Atualizar documentação",
            "Adicionar comentários",
            "Update README",
            "Add documentation"
        ]
        
        for desc in descriptions:
            commit_type = agent._analyze_commit_type(desc)
            assert commit_type == ConventionalCommitType.DOCS
    
    def test_generate_conventional_message(self, agent):
        """Testa geração de mensagem convencional"""
        # Teste básico
        message = agent._generate_conventional_message(
            ConventionalCommitType.FEAT,
            "Implementar autenticação"
        )
        assert message == "feat: Implementar autenticação"
        
        # Teste com escopo
        message = agent._generate_conventional_message(
            ConventionalCommitType.FIX,
            "Corrigir bug de validação",
            scope="auth"
        )
        assert message == "fix(auth): Corrigir bug de validação"
        
        # Teste com breaking change
        message = agent._generate_conventional_message(
            ConventionalCommitType.FEAT,
            "Nova API de autenticação",
            scope="api",
            breaking_change=True
        )
        assert message == "feat(api)!: Nova API de autenticação"
    
    def test_generate_quantum_signature(self, agent):
        """Testa geração de assinatura quântica"""
        task_node = TaskNode(
            id="test_001",
            type="git",
            description="Teste"
        )
        
        signature1 = agent._generate_quantum_signature(task_node)
        signature2 = agent._generate_quantum_signature(task_node)
        
        # Assinaturas devem ser diferentes devido ao timestamp
        assert signature1 != signature2
        assert len(signature1) == 16
        assert len(signature2) == 16
    
    @pytest.mark.asyncio
    async def test_observe_task_nodes(self, agent, sample_task_nodes):
        """Testa observação de TaskNodes"""
        with patch.object(agent, '_detect_modified_files', new_callable=AsyncMock) as mock_detect:
            mock_detect.return_value = ["test.py"]
            
            operations = await agent.observe_task_nodes(sample_task_nodes)
            
            assert len(operations) == 3
            assert all(isinstance(op, GitOperation) for op in operations)
            assert all(op.task_node.id in ["task_001", "task_002", "task_003"] for op in operations)
            
            # Verificar tipos de commit identificados
            commit_types = [op.commit_type for op in operations]
            assert ConventionalCommitType.FEAT in commit_types  # "Implementar"
            assert ConventionalCommitType.FIX in commit_types   # "Corrigir"
            assert ConventionalCommitType.DOCS in commit_types  # "Atualizar documentação"
    
    @pytest.mark.asyncio
    async def test_validate_commit_message(self, agent):
        """Testa validação de mensagens de commit"""
        # Mensagens válidas
        valid_messages = [
            "feat: adicionar nova funcionalidade",
            "fix(auth): corrigir bug de autenticação",
            "docs: atualizar documentação",
            "feat(api)!: nova API com breaking change"
        ]
        
        for message in valid_messages:
            is_valid = await agent._validate_commit_message(message)
            assert is_valid, f"Mensagem deve ser válida: {message}"
        
        # Mensagens inválidas
        invalid_messages = [
            "adicionar nova funcionalidade",  # sem tipo
            "feat adicionar funcionalidade",  # sem dois pontos
            "invalid: tipo inválido",          # tipo não reconhecido
            "",                                 # vazia
        ]
        
        for message in invalid_messages:
            is_valid = await agent._validate_commit_message(message)
            assert not is_valid, f"Mensagem deve ser inválida: {message}"
    
    @pytest.mark.asyncio
    async def test_simulate_operation(self, agent, sample_task_nodes):
        """Testa simulação de operação (dry-run)"""
        with patch.object(agent, '_detect_modified_files', new_callable=AsyncMock) as mock_detect:
            mock_detect.return_value = ["test.py"]
            
            operations = await agent.observe_task_nodes([sample_task_nodes[0]])
            operation = operations[0]
            
            await agent._simulate_operation(operation)
            
            # Verificar logs de simulação
            assert len(operation.execution_log) > 0
            assert any("[DRY-RUN]" in log for log in operation.execution_log)
            assert any("Adicionaria arquivos" in log for log in operation.execution_log)
    
    @pytest.mark.asyncio
    async def test_consciousness_update(self, agent):
        """Testa atualização da consciência evolutiva"""
        task_node = TaskNode(id="test", type="git", description="Test")
        operation = GitOperation(
            task_node=task_node,
            commit_type=ConventionalCommitType.FEAT,
            commit_message="feat: test",
            files_to_add=[]
        )
        
        # Estado inicial
        assert agent.consciousness.total_operations == 0
        assert agent.consciousness.successful_operations == 0
        assert agent.consciousness.evolution_level == 0.0
        
        # Simular operação bem-sucedida
        await agent._update_consciousness(operation, success=True)
        
        assert agent.consciousness.total_operations == 1
        assert agent.consciousness.successful_operations == 1
        assert agent.consciousness.evolution_level > 0.0
        assert agent.consciousness.quantum_coherence > 1.0
        
        # Simular operação falhada
        await agent._update_consciousness(operation, success=False)
        
        assert agent.consciousness.total_operations == 2
        assert agent.consciousness.failed_operations == 1
        assert agent.consciousness.quantum_coherence < 1.0
    
    def test_consciousness_state_export_import(self, agent, temp_git_repo):
        """Testa exportação e importação de estado de consciência"""
        # Modificar estado de consciência
        agent.consciousness.total_operations = 10
        agent.consciousness.successful_operations = 8
        agent.consciousness.evolution_level = 0.8
        agent.consciousness.patterns_learned = {"feat:test": 5}
        agent.rollback_registry = {"op1": "abc123"}
        
        # Exportar
        export_file = temp_git_repo / "consciousness.json"
        agent.export_consciousness(str(export_file))
        
        assert export_file.exists()
        
        # Criar novo agente e importar
        new_agent = GitTaskAgent(repository_path=str(temp_git_repo))
        new_agent.import_consciousness(str(export_file))
        
        # Verificar se estado foi restaurado
        assert new_agent.consciousness.total_operations == 10
        assert new_agent.consciousness.successful_operations == 8
        assert new_agent.consciousness.evolution_level == 0.8
        assert new_agent.consciousness.patterns_learned == {"feat:test": 5}
        assert new_agent.rollback_registry == {"op1": "abc123"}
    
    def test_get_consciousness_state(self, agent):
        """Testa obtenção de estado de consciência"""
        state = agent.get_consciousness_state()
        
        required_fields = [
            "evolution_level", "quantum_coherence", "total_operations",
            "success_rate", "patterns_learned", "last_evolution"
        ]
        
        for field in required_fields:
            assert field in state
        
        assert isinstance(state["evolution_level"], float)
        assert isinstance(state["quantum_coherence"], float)
        assert isinstance(state["total_operations"], int)
        assert isinstance(state["success_rate"], float)
        assert isinstance(state["patterns_learned"], int)
        assert isinstance(state["last_evolution"], str)
    
    @pytest.mark.asyncio
    async def test_batch_execute_dry_run(self, agent, sample_task_nodes):
        """Testa execução em lote com dry-run"""
        with patch.object(agent, '_detect_modified_files', new_callable=AsyncMock) as mock_detect:
            mock_detect.return_value = ["test.py"]
            
            operations = await agent.observe_task_nodes(sample_task_nodes)
            results = await agent.batch_execute(operations, dry_run=True)
            
            # Todos devem ter sucesso em dry-run
            assert len(results) == 3
            assert all(success for success in results.values())
            
            # Verificar que foram marcadas como dry-run
            for operation in operations:
                assert operation.dry_run
                assert operation.status == GitOperationStatus.DRY_RUN

class TestGitTaskAgentIntegration:
    """Testes de integração que requerem Git real (mock quando necessário)"""
    
    @pytest.fixture
    def real_git_repo(self):
        """Cria repositório Git real para testes de integração"""
        import subprocess
        
        temp_dir = tempfile.mkdtemp()
        repo_path = Path(temp_dir)
        
        try:
            # Inicializar repositório Git
            subprocess.run(["git", "init"], cwd=repo_path, check=True)
            subprocess.run(["git", "config", "user.name", "Test User"], cwd=repo_path, check=True)
            subprocess.run(["git", "config", "user.email", "test@example.com"], cwd=repo_path, check=True)
            
            # Commit inicial
            readme = repo_path / "README.md"
            readme.write_text("# Test Repository")
            
            subprocess.run(["git", "add", "README.md"], cwd=repo_path, check=True)
            subprocess.run(["git", "commit", "-m", "Initial commit"], cwd=repo_path, check=True)
            
            yield repo_path
            
        finally:
            # Limpeza
            shutil.rmtree(temp_dir)
    
    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_full_git_workflow(self, real_git_repo):
        """Teste completo do workflow Git (requer Git instalado)"""
        agent = GitTaskAgent(repository_path=str(real_git_repo))
        
        # Criar arquivo para commit
        test_file = real_git_repo / "test_feature.py"
        test_file.write_text("def new_feature():\n    return 'Hello World'")
        
        # Criar TaskNode
        task_node = TaskNode(
            id="integration_test",
            type="git",
            description="Adicionar nova funcionalidade de teste",
            files=["test_feature.py"],
            scope="feature"
        )
        
        # Observar e executar
        operations = await agent.observe_task_nodes([task_node])
        operation = operations[0]
        
        # Executar operação real
        success = await agent.execute_operation(operation, dry_run=False)
        
        assert success
        assert operation.status == GitOperationStatus.COMPLETED
        assert operation.rollback_hash is not None
        
        # Verificar se commit foi criado
        import subprocess
        result = subprocess.run(
            ["git", "log", "--oneline", "-1"],
            cwd=real_git_repo,
            capture_output=True,
            text=True
        )
        
        assert "feat(feature): Adicionar nova funcionalidade de teste" in result.stdout
        
        # Testar rollback
        rollback_success = await agent.rollback_operation("integration_test")
        assert rollback_success

@pytest.mark.asyncio
async def test_pattern_learning():
    """Testa aprendizado de padrões evolutivo"""
    with tempfile.TemporaryDirectory() as temp_dir:
        repo_path = Path(temp_dir)
        git_dir = repo_path / ".git"
        git_dir.mkdir()
        
        agent = GitTaskAgent(repository_path=str(repo_path))
        
        # Simular várias descrições que devem ensinar padrões
        descriptions = [
            "Implementar nova funcionalidade",
            "Adicionar feature de login",
            "Criar sistema de autenticação",
            "Implementar API de usuários"
        ]
        
        for desc in descriptions:
            commit_type = agent._analyze_commit_type(desc)
            # Todos devem ser identificados como FEAT
            assert commit_type == ConventionalCommitType.FEAT
        
        # Verificar que padrões foram aprendidos
        patterns = agent.consciousness.patterns_learned
        assert len(patterns) > 0
        
        # Deve ter registrado o padrão "implementar" várias vezes
        feat_patterns = {k: v for k, v in patterns.items() if k.startswith("feat:")}
        assert len(feat_patterns) > 0

if __name__ == "__main__":
    pytest.main(["-v", __file__])


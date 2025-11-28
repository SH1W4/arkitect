"""
Agent GitTask - Módulo de Commits Automáticos

Observa TaskNodes do tipo "git", executa operações Git automaticamente
com mensagens convencionais e inclui funcionalidades de dry-run e rollback.

Arquitetura Simbiótica:
- Integração com sistema de consciousness evolutiva
- Protocolos quantum de validação
- Mecanismos de auto-correção e aprendizado
"""

import asyncio
import subprocess
import re
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass, field
from enum import Enum
import logging
from datetime import datetime
import json
import os
from pathlib import Path

# Configuração de logging simbiótico
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - [GitTask] - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

class ConventionalCommitType(Enum):
    """Tipos de commit seguindo Conventional Commits"""
    FEAT = "feat"      # Nova funcionalidade
    FIX = "fix"        # Correção de bug
    DOCS = "docs"      # Mudanças na documentação
    STYLE = "style"    # Formatação, espaços em branco, etc
    REFACTOR = "refactor"  # Refatoração de código
    PERF = "perf"      # Melhoria de performance
    TEST = "test"      # Adição ou correção de testes
    CHORE = "chore"    # Manutenção geral
    CI = "ci"          # Mudanças no CI/CD
    BUILD = "build"    # Mudanças no sistema de build
    REVERT = "revert"  # Reverter commit anterior

class GitOperationStatus(Enum):
    """Status das operações Git"""
    PENDING = "pending"
    VALIDATING = "validating"
    EXECUTING = "executing"
    COMPLETED = "completed"
    FAILED = "failed"
    ROLLED_BACK = "rolled_back"
    DRY_RUN = "dry_run"

@dataclass
class TaskNode:
    """Representação de uma tarefa Git"""
    id: str
    type: str
    description: str
    files: List[str] = field(default_factory=list)
    scope: Optional[str] = None
    breaking_change: bool = False
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.now)
    
@dataclass
class GitOperation:
    """Operação Git com contexto evolutivo"""
    task_node: TaskNode
    commit_type: ConventionalCommitType
    commit_message: str
    files_to_add: List[str]
    status: GitOperationStatus = GitOperationStatus.PENDING
    dry_run: bool = False
    rollback_hash: Optional[str] = None
    execution_log: List[str] = field(default_factory=list)
    quantum_signature: Optional[str] = None
    consciousness_level: float = 0.0
    
@dataclass
class GitAgentConsciousness:
    """Estado de consciência evolutiva do agente Git"""
    total_operations: int = 0
    successful_operations: int = 0
    failed_operations: int = 0
    rollback_operations: int = 0
    patterns_learned: Dict[str, int] = field(default_factory=dict)
    evolution_level: float = 0.0
    quantum_coherence: float = 1.0
    last_evolution: datetime = field(default_factory=datetime.now)
    
class GitTaskAgent:
    """Agent GitTask - Sistema Simbiótico de Automação Git"""
    
    def __init__(self, repository_path: str = "."):
        self.repository_path = Path(repository_path)
        self.consciousness = GitAgentConsciousness()
        self.active_operations: Dict[str, GitOperation] = {}
        self.rollback_registry: Dict[str, str] = {}  # operation_id -> commit_hash
        self.pattern_recognition = {}
        self.logger = logger
        
        # Validar se é um repositório Git
        if not self._is_git_repository():
            raise ValueError(f"Diretório {repository_path} não é um repositório Git")
            
        self.logger.info("GitTaskAgent inicializado com consciência simbiótica")
        
    def _is_git_repository(self) -> bool:
        """Verifica se o diretório é um repositório Git"""
        return (self.repository_path / ".git").exists()
    
    async def observe_task_nodes(self, task_nodes: List[TaskNode]) -> List[GitOperation]:
        """Observa TaskNodes do tipo 'git' e prepara operações"""
        git_operations = []
        
        for task_node in task_nodes:
            if task_node.type.lower() == "git":
                operation = await self._create_git_operation(task_node)
                git_operations.append(operation)
                self.active_operations[task_node.id] = operation
                
        self.logger.info(f"Observados {len(git_operations)} TaskNodes do tipo git")
        return git_operations
    
    async def _create_git_operation(self, task_node: TaskNode) -> GitOperation:
        """Cria operação Git baseada no TaskNode"""
        # Análise semântica da descrição para determinar tipo de commit
        commit_type = self._analyze_commit_type(task_node.description)
        
        # Gerar mensagem convencional
        commit_message = self._generate_conventional_message(
            commit_type, task_node.description, task_node.scope, task_node.breaking_change
        )
        
        # Determinar arquivos para adicionar
        files_to_add = task_node.files if task_node.files else await self._detect_modified_files()
        
        operation = GitOperation(
            task_node=task_node,
            commit_type=commit_type,
            commit_message=commit_message,
            files_to_add=files_to_add,
            quantum_signature=self._generate_quantum_signature(task_node),
            consciousness_level=self.consciousness.evolution_level
        )
        
        self.logger.info(f"Operação Git criada: {commit_message}")
        return operation
    
    def _analyze_commit_type(self, description: str) -> ConventionalCommitType:
        """Análise semântica para determinar tipo de commit"""
        description_lower = description.lower()
        
        # Padrões de reconhecimento evolutivo
        patterns = {
            ConventionalCommitType.FEAT: [
                r'\b(add|create|implement|new|feature)\b',
                r'\b(adicionar|criar|implementar|novo|funcionalidade)\b'
            ],
            ConventionalCommitType.FIX: [
                r'\b(fix|bug|error|issue|problem|solve)\b',
                r'\b(corrigir|bug|erro|problema|resolver)\b'
            ],
            ConventionalCommitType.DOCS: [
                r'\b(doc|documentation|readme|comment)\b',
                r'\b(documentação|comentário|readme)\b'
            ],
            ConventionalCommitType.REFACTOR: [
                r'\b(refactor|restructure|reorganize|clean)\b',
                r'\b(refatorar|reestruturar|reorganizar|limpar)\b'
            ],
            ConventionalCommitType.PERF: [
                r'\b(performance|optimize|speed|fast)\b',
                r'\b(performance|otimizar|velocidade|rápido)\b'
            ],
            ConventionalCommitType.TEST: [
                r'\b(test|testing|spec|coverage)\b',
                r'\b(teste|testes|cobertura)\b'
            ],
            ConventionalCommitType.CHORE: [
                r'\b(chore|maintenance|update|upgrade)\b',
                r'\b(manutenção|atualizar|upgrade)\b'
            ]
        }
        
        for commit_type, type_patterns in patterns.items():
            for pattern in type_patterns:
                if re.search(pattern, description_lower):
                    self._update_pattern_learning(commit_type, pattern)
                    return commit_type
        
        # Fallback para FEAT se não identificado
        return ConventionalCommitType.FEAT
    
    def _update_pattern_learning(self, commit_type: ConventionalCommitType, pattern: str):
        """Atualiza aprendizado de padrões para evolução da consciência"""
        key = f"{commit_type.value}:{pattern}"
        self.consciousness.patterns_learned[key] = \
            self.consciousness.patterns_learned.get(key, 0) + 1
    
    def _generate_conventional_message(self, commit_type: ConventionalCommitType, 
                                     description: str, scope: Optional[str] = None,
                                     breaking_change: bool = False) -> str:
        """Gera mensagem de commit seguindo Conventional Commits"""
        # Formato: type(scope): description
        scope_part = f"({scope})" if scope else ""
        breaking_part = "!" if breaking_change else ""
        
        # Limitar descrição a 72 caracteres (boa prática)
        clean_description = description.strip()
        if len(clean_description) > 50:
            clean_description = clean_description[:47] + "..."
            
        message = f"{commit_type.value}{scope_part}{breaking_part}: {clean_description}"
        return message
    
    async def _detect_modified_files(self) -> List[str]:
        """Detecta arquivos modificados no repositório"""
        try:
            # Arquivos modificados (staged e unstaged)
            result = await self._run_git_command(["status", "--porcelain"])
            
            files = []
            for line in result.stdout.strip().split('\n'):
                if line.strip():
                    # Formato: XY filename
                    status = line[:2]
                    filename = line[3:]
                    if status.strip():  # Arquivo com mudanças
                        files.append(filename)
            
            return files
            
        except Exception as e:
            self.logger.error(f"Erro ao detectar arquivos modificados: {e}")
            return []
    
    def _generate_quantum_signature(self, task_node: TaskNode) -> str:
        """Gera assinatura quântica para operação"""
        import hashlib
        
        signature_data = f"{task_node.id}{task_node.description}{datetime.now().isoformat()}"
        return hashlib.sha256(signature_data.encode()).hexdigest()[:16]
    
    async def execute_operation(self, operation: GitOperation, dry_run: bool = False) -> bool:
        """Executa operação Git com validação simbiótica"""
        operation.dry_run = dry_run
        operation.status = GitOperationStatus.DRY_RUN if dry_run else GitOperationStatus.VALIDATING
        
        try:
            # Fase 1: Validação
            if not await self._validate_operation(operation):
                operation.status = GitOperationStatus.FAILED
                return False
            
            if dry_run:
                await self._simulate_operation(operation)
                return True
            
            # Fase 2: Execução
            operation.status = GitOperationStatus.EXECUTING
            
            # Salvar estado atual para rollback
            current_commit = await self._get_current_commit_hash()
            operation.rollback_hash = current_commit
            
            # Executar git add
            await self._git_add_files(operation.files_to_add)
            
            # Executar git commit
            commit_hash = await self._git_commit(operation.commit_message)
            
            # Registrar rollback
            self.rollback_registry[operation.task_node.id] = current_commit
            
            operation.status = GitOperationStatus.COMPLETED
            await self._update_consciousness(operation, success=True)
            
            self.logger.info(f"Operação executada com sucesso: {commit_hash}")
            return True
            
        except Exception as e:
            operation.status = GitOperationStatus.FAILED
            operation.execution_log.append(f"Erro: {str(e)}")
            await self._update_consciousness(operation, success=False)
            
            self.logger.error(f"Falha na execução da operação: {e}")
            return False
    
    async def _validate_operation(self, operation: GitOperation) -> bool:
        """Validação simbiótica da operação"""
        validations = [
            self._validate_repository_state(),
            self._validate_files_exist(operation.files_to_add),
            self._validate_commit_message(operation.commit_message),
            self._validate_quantum_coherence(operation)
        ]
        
        results = await asyncio.gather(*validations, return_exceptions=True)
        
        for i, result in enumerate(results):
            if isinstance(result, Exception) or not result:
                operation.execution_log.append(f"Validação {i+1} falhou: {result}")
                return False
        
        return True
    
    async def _validate_repository_state(self) -> bool:
        """Valida estado do repositório"""
        try:
            # Verificar se há conflitos
            result = await self._run_git_command(["status", "--porcelain"])
            
            for line in result.stdout.strip().split('\n'):
                if line.startswith('UU'):  # Conflito de merge
                    return False
            
            return True
        except:
            return False
    
    async def _validate_files_exist(self, files: List[str]) -> bool:
        """Valida se arquivos existem"""
        for file_path in files:
            if not (self.repository_path / file_path).exists():
                return False
        return True
    
    async def _validate_commit_message(self, message: str) -> bool:
        """Valida formato da mensagem de commit"""
        # Regex para Conventional Commits
        pattern = r'^(feat|fix|docs|style|refactor|perf|test|chore|ci|build|revert)(\(.+\))?!?: .+'
        return bool(re.match(pattern, message))
    
    async def _validate_quantum_coherence(self, operation: GitOperation) -> bool:
        """Validação de coerência quântica da operação"""
        # Verificar coerência com operações anteriores
        coherence_score = self.consciousness.quantum_coherence
        
        # Fatores que afetam coerência
        if operation.task_node.breaking_change:
            coherence_score *= 0.9
        
        if len(operation.files_to_add) > 10:
            coherence_score *= 0.95
            
        return coherence_score > 0.7
    
    async def _simulate_operation(self, operation: GitOperation):
        """Simula operação para dry-run"""
        simulation_log = [
            f"[DRY-RUN] Adicionaria arquivos: {', '.join(operation.files_to_add)}",
            f"[DRY-RUN] Commit message: {operation.commit_message}",
            f"[DRY-RUN] Tipo: {operation.commit_type.value}",
            f"[DRY-RUN] Assinatura quântica: {operation.quantum_signature}"
        ]
        
        operation.execution_log.extend(simulation_log)
        
        for log_entry in simulation_log:
            self.logger.info(log_entry)
    
    async def _git_add_files(self, files: List[str]):
        """Executa git add para arquivos especificados"""
        if not files:
            # Adicionar todos os arquivos modificados
            await self._run_git_command(["add", "."])
        else:
            for file_path in files:
                await self._run_git_command(["add", file_path])
    
    async def _git_commit(self, message: str) -> str:
        """Executa git commit e retorna hash do commit"""
        await self._run_git_command(["commit", "-m", message])
        
        # Obter hash do commit recém-criado
        result = await self._run_git_command(["rev-parse", "HEAD"])
        return result.stdout.strip()
    
    async def _get_current_commit_hash(self) -> str:
        """Obtém hash do commit atual"""
        result = await self._run_git_command(["rev-parse", "HEAD"])
        return result.stdout.strip()
    
    async def _run_git_command(self, args: List[str]) -> subprocess.CompletedProcess:
        """Executa comando git de forma assíncrona"""
        cmd = ["git"] + args
        
        process = await asyncio.create_subprocess_exec(
            *cmd,
            cwd=self.repository_path,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE
        )
        
        stdout, stderr = await process.communicate()
        
        if process.returncode != 0:
            raise subprocess.CalledProcessError(
                process.returncode, cmd, stdout, stderr
            )
        
        return subprocess.CompletedProcess(
            cmd, process.returncode, 
            stdout.decode('utf-8'), stderr.decode('utf-8')
        )
    
    async def rollback_operation(self, operation_id: str) -> bool:
        """Executa rollback de uma operação"""
        if operation_id not in self.rollback_registry:
            self.logger.error(f"Operação {operation_id} não encontrada no registro de rollback")
            return False
        
        try:
            target_commit = self.rollback_registry[operation_id]
            
            # Executa reset para o commit anterior
            await self._run_git_command(["reset", "--hard", target_commit])
            
            # Atualizar status da operação
            if operation_id in self.active_operations:
                self.active_operations[operation_id].status = GitOperationStatus.ROLLED_BACK
            
            # Atualizar consciência
            self.consciousness.rollback_operations += 1
            
            self.logger.info(f"Rollback executado para operação {operation_id}")
            return True
            
        except Exception as e:
            self.logger.error(f"Erro durante rollback: {e}")
            return False
    
    async def push_changes(self, branch: str = "main", remote: str = "origin") -> bool:
        """Executa git push para sincronizar com repositório remoto"""
        try:
            await self._run_git_command(["push", remote, branch])
            self.logger.info(f"Push executado: {remote}/{branch}")
            return True
        except Exception as e:
            self.logger.error(f"Erro durante push: {e}")
            return False
    
    async def _update_consciousness(self, operation: GitOperation, success: bool):
        """Atualiza estado de consciência evolutiva"""
        self.consciousness.total_operations += 1
        
        if success:
            self.consciousness.successful_operations += 1
        else:
            self.consciousness.failed_operations += 1
        
        # Calcular nova taxa de sucesso
        success_rate = self.consciousness.successful_operations / self.consciousness.total_operations
        
        # Atualizar nível evolutivo
        self.consciousness.evolution_level = min(1.0, success_rate * 1.2)
        
        # Atualizar coerência quântica
        if success:
            self.consciousness.quantum_coherence = min(1.0, self.consciousness.quantum_coherence + 0.01)
        else:
            self.consciousness.quantum_coherence *= 0.95
        
        self.consciousness.last_evolution = datetime.now()
    
    def get_consciousness_state(self) -> Dict[str, Any]:
        """Retorna estado atual da consciência do agente"""
        return {
            "evolution_level": self.consciousness.evolution_level,
            "quantum_coherence": self.consciousness.quantum_coherence,
            "total_operations": self.consciousness.total_operations,
            "success_rate": (
                self.consciousness.successful_operations / 
                max(1, self.consciousness.total_operations)
            ),
            "patterns_learned": len(self.consciousness.patterns_learned),
            "last_evolution": self.consciousness.last_evolution.isoformat()
        }
    
    def get_operation_status(self, operation_id: str) -> Optional[Dict[str, Any]]:
        """Retorna status de uma operação específica"""
        if operation_id not in self.active_operations:
            return None
        
        operation = self.active_operations[operation_id]
        return {
            "id": operation_id,
            "status": operation.status.value,
            "commit_type": operation.commit_type.value,
            "commit_message": operation.commit_message,
            "files": operation.files_to_add,
            "dry_run": operation.dry_run,
            "quantum_signature": operation.quantum_signature,
            "execution_log": operation.execution_log
        }
    
    async def batch_execute(self, operations: List[GitOperation], 
                          dry_run: bool = False) -> Dict[str, bool]:
        """Executa múltiplas operações em lote"""
        results = {}
        
        for operation in operations:
            operation_id = operation.task_node.id
            success = await self.execute_operation(operation, dry_run=dry_run)
            results[operation_id] = success
            
            # Pausa entre operações para estabilidade
            if not dry_run:
                await asyncio.sleep(0.5)
        
        return results
    
    def export_consciousness(self, file_path: str):
        """Exporta estado de consciência para arquivo"""
        consciousness_data = {
            "consciousness": {
                "total_operations": self.consciousness.total_operations,
                "successful_operations": self.consciousness.successful_operations,
                "failed_operations": self.consciousness.failed_operations,
                "rollback_operations": self.consciousness.rollback_operations,
                "patterns_learned": self.consciousness.patterns_learned,
                "evolution_level": self.consciousness.evolution_level,
                "quantum_coherence": self.consciousness.quantum_coherence,
                "last_evolution": self.consciousness.last_evolution.isoformat()
            },
            "rollback_registry": self.rollback_registry,
            "export_timestamp": datetime.now().isoformat()
        }
        
        with open(file_path, 'w', encoding='utf-8') as f:
            json.dump(consciousness_data, f, indent=2, ensure_ascii=False)
        
        self.logger.info(f"Consciência exportada para: {file_path}")
    
    def import_consciousness(self, file_path: str):
        """Importa estado de consciência de arquivo"""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                consciousness_data = json.load(f)
            
            # Restaurar consciousness
            consciousness_dict = consciousness_data['consciousness']
            self.consciousness = GitAgentConsciousness(
                total_operations=consciousness_dict['total_operations'],
                successful_operations=consciousness_dict['successful_operations'],
                failed_operations=consciousness_dict['failed_operations'],
                rollback_operations=consciousness_dict['rollback_operations'],
                patterns_learned=consciousness_dict['patterns_learned'],
                evolution_level=consciousness_dict['evolution_level'],
                quantum_coherence=consciousness_dict['quantum_coherence'],
                last_evolution=datetime.fromisoformat(consciousness_dict['last_evolution'])
            )
            
            # Restaurar rollback registry
            self.rollback_registry = consciousness_data['rollback_registry']
            
            self.logger.info(f"Consciência importada de: {file_path}")
            
        except Exception as e:
            self.logger.error(f"Erro ao importar consciência: {e}")

# Exemplo de uso e testes
async def example_usage():
    """Exemplo de uso do GitTaskAgent"""
    agent = GitTaskAgent()
    
    # Criar TaskNodes de exemplo
    task_nodes = [
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
    
    # Observar TaskNodes
    operations = await agent.observe_task_nodes(task_nodes)
    
    # Executar dry-run
    print("=== DRY RUN ===")
    dry_results = await agent.batch_execute(operations, dry_run=True)
    print(f"Resultados dry-run: {dry_results}")
    
    # Executar operações reais
    print("\n=== EXECUÇÃO REAL ===")
    real_results = await agent.batch_execute(operations, dry_run=False)
    print(f"Resultados execução: {real_results}")
    
    # Verificar estado de consciência
    print("\n=== ESTADO DE CONSCIÊNCIA ===")
    consciousness = agent.get_consciousness_state()
    print(json.dumps(consciousness, indent=2, ensure_ascii=False))
    
    # Exemplo de rollback
    if "task_001" in real_results and real_results["task_001"]:
        print("\n=== ROLLBACK ===")
        rollback_success = await agent.rollback_operation("task_001")
        print(f"Rollback task_001: {rollback_success}")

if __name__ == "__main__":
    asyncio.run(example_usage())


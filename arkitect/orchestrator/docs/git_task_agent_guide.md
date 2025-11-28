# GitTaskAgent - Guia Completo

## 🤖 Visão Geral

O **GitTaskAgent** é um sistema simbiótico de automação Git que observa TaskNodes do tipo "git" e executa operações Git automaticamente com mensagens convencionais, incluindo funcionalidades de dry-run e rollback.

### ✨ Características Principais

- **Observação Inteligente**: Identifica automaticamente TaskNodes do tipo "git"
- **Mensagens Convencionais**: Gera mensagens seguindo o padrão Conventional Commits
- **Análise Semântica**: Determina o tipo de commit baseado na descrição da tarefa
- **Dry-Run**: Simula operações sem executar commits reais
- **Rollback**: Sistema completo de desfazer operações
- **Consciência Evolutiva**: Aprende padrões e evolui com o uso
- **Validação Quântica**: Sistema de validação de coerência

## 🛠️ Instalação e Configuração

### Pré-requisitos

```bash
# Git instalado e configurado
git --version

# Python 3.8+
python --version

# Dependências Python
pip install fastapi uvicorn pytest asyncio
```

### Configuração Rápida

```python
from agents.git_task_agent import GitTaskAgent, TaskNode

# Inicializar agente
agent = GitTaskAgent(repository_path="/caminho/para/repositorio")

# Criar TaskNode
task = TaskNode(
    id="exemplo_001",
    type="git",
    description="Implementar nova funcionalidade",
    files=["src/feature.py"],
    scope="feature"
)

# Executar operação
operations = await agent.observe_task_nodes([task])
result = await agent.execute_operation(operations[0], dry_run=True)
```

## 📚 API Reference

### Classe GitTaskAgent

#### Construtor

```python
GitTaskAgent(repository_path: str = ".")
```

**Parâmetros:**
- `repository_path`: Caminho para o repositório Git

#### Métodos Principais

##### observe_task_nodes

```python
async def observe_task_nodes(task_nodes: List[TaskNode]) -> List[GitOperation]
```

Observa TaskNodes do tipo "git" e prepara operações.

**Parâmetros:**
- `task_nodes`: Lista de TaskNodes para processar

**Retorna:**
- Lista de GitOperations preparadas

##### execute_operation

```python
async def execute_operation(operation: GitOperation, dry_run: bool = False) -> bool
```

Executa uma operação Git com validação.

**Parâmetros:**
- `operation`: Operação Git para executar
- `dry_run`: Se deve apenas simular a operação

**Retorna:**
- `True` se sucesso, `False` se falhou

##### batch_execute

```python
async def batch_execute(operations: List[GitOperation], dry_run: bool = False) -> Dict[str, bool]
```

Executa múltiplas operações em lote.

##### rollback_operation

```python
async def rollback_operation(operation_id: str) -> bool
```

Executa rollback de uma operação específica.

##### push_changes

```python
async def push_changes(branch: str = "main", remote: str = "origin") -> bool
```

Executa git push para repositório remoto.

### Classe TaskNode

```python
@dataclass
class TaskNode:
    id: str
    type: str
    description: str
    files: List[str] = field(default_factory=list)
    scope: Optional[str] = None
    breaking_change: bool = False
    metadata: Dict[str, Any] = field(default_factory=dict)
    created_at: datetime = field(default_factory=datetime.now)
```

### Tipos de Commit Suportados

| Tipo | Descrição | Palavras-chave |
|------|-------------|----------------|
| `feat` | Nova funcionalidade | add, create, implement, new, feature |
| `fix` | Correção de bug | fix, bug, error, issue, problem, solve |
| `docs` | Documentação | doc, documentation, readme, comment |
| `style` | Formatação | style, format, whitespace |
| `refactor` | Refatoração | refactor, restructure, reorganize, clean |
| `perf` | Performance | performance, optimize, speed, fast |
| `test` | Testes | test, testing, spec, coverage |
| `chore` | Manutenção | chore, maintenance, update, upgrade |

## 🎨 Exemplos de Uso

### Exemplo Básico

```python
import asyncio
from agents.git_task_agent import GitTaskAgent, TaskNode

async def exemplo_basico():
    # Inicializar agente
    agent = GitTaskAgent()
    
    # Criar tarefas
    tasks = [
        TaskNode(
            id="task_001",
            type="git",
            description="Implementar autenticação OAuth",
            files=["auth.py", "oauth.py"],
            scope="auth"
        ),
        TaskNode(
            id="task_002",
            type="git",
            description="Corrigir bug na validação",
            files=["validators.py"],
            scope="core"
        )
    ]
    
    # Observar e executar
    operations = await agent.observe_task_nodes(tasks)
    
    # Dry-run primeiro
    dry_results = await agent.batch_execute(operations, dry_run=True)
    print(f"Dry-run: {dry_results}")
    
    # Execução real
    if all(dry_results.values()):
        real_results = await agent.batch_execute(operations, dry_run=False)
        print(f"Real: {real_results}")
        
        # Push automático
        await agent.push_changes()

# Executar
asyncio.run(exemplo_basico())
```

### Exemplo com Rollback

```python
async def exemplo_rollback():
    agent = GitTaskAgent()
    
    # Executar operação
    task = TaskNode(
        id="risky_task",
        type="git",
        description="Implementação experimental",
        scope="experimental"
    )
    
    operations = await agent.observe_task_nodes([task])
    success = await agent.execute_operation(operations[0])
    
    if success:
        print("Operação executada, mas vamos desfazer...")
        
        # Rollback
        rollback_success = await agent.rollback_operation("risky_task")
        if rollback_success:
            print("Rollback executado com sucesso!")
```

### Exemplo de Consciência Evolutiva

```python
async def exemplo_consciencia():
    agent = GitTaskAgent()
    
    # Verificar estado inicial
    initial_state = agent.get_consciousness_state()
    print(f"Estado inicial: {initial_state}")
    
    # Executar várias operações para evolução
    tasks = [
        TaskNode(id=f"learn_{i}", type="git", description=f"Implementar módulo {i}")
        for i in range(5)
    ]
    
    for task in tasks:
        ops = await agent.observe_task_nodes([task])
        await agent.execute_operation(ops[0], dry_run=True)
    
    # Verificar evolução
    evolved_state = agent.get_consciousness_state()
    print(f"Estado evoluído: {evolved_state}")
    
    # Exportar conhecimento
    agent.export_consciousness("agent_knowledge.json")
```

## 🌐 API REST

O GitTaskAgent também fornece uma API REST completa:

### Endpoints Principais

#### Configurar Agente
```http
POST /git-agent/configure
Content-Type: application/json

{
    "repository_path": "/caminho/para/repo"
}
```

#### Observar TaskNodes
```http
POST /git-agent/observe
Content-Type: application/json

[
    {
        "id": "task_001",
        "type": "git",
        "description": "Implementar nova feature",
        "files": ["feature.py"],
        "scope": "feature"
    }
]
```

#### Executar em Lote
```http
POST /git-agent/execute
Content-Type: application/json

{
    "task_nodes": [...],
    "dry_run": false,
    "auto_push": true,
    "branch": "main",
    "remote": "origin"
}
```

#### Rollback
```http
POST /git-agent/rollback
Content-Type: application/json

{
    "operation_id": "task_001"
}
```

#### Status da Consciência
```http
GET /git-agent/consciousness
```

#### Verificar Saúde
```http
GET /git-agent/health
```

## 🧪 Testes

### Executar Testes

```bash
# Todos os testes
pytest tests/test_git_task_agent.py -v

# Apenas testes unitários
pytest tests/test_git_task_agent.py -v -m "not integration"

# Testes de integração (requer Git)
pytest tests/test_git_task_agent.py -v -m integration
```

### Demonstração Interativa

```bash
# Executar demonstração completa
python examples/git_agent_demo.py
```

## 🔍 Monitoramento e Debug

### Logs

O agente utiliza logging estruturado:

```python
import logging
logging.basicConfig(level=logging.DEBUG)

# Logs incluem:
# - Operações observadas
# - Validações executadas
# - Commits realizados
# - Evolução da consciência
```

### Métricas de Consciência

```python
# Verificar métricas
state = agent.get_consciousness_state()
print(f"Nível evolutivo: {state['evolution_level']}")
print(f"Coerência quântica: {state['quantum_coherence']}")
print(f"Taxa de sucesso: {state['success_rate']}")
print(f"Padrões aprendidos: {state['patterns_learned']}")
```

### Padrões Aprendidos

```python
# Ver padrões detalhados
patterns = agent.consciousness.patterns_learned
for pattern, count in patterns.items():
    print(f"{pattern}: usado {count} vezes")
```

## ⚠️ Segurança e Boas Práticas

### Validações

1. **Estado do Repositório**: Verifica conflitos antes de operar
2. **Existência de Arquivos**: Confirma que arquivos existem
3. **Formato de Mensagem**: Valida Conventional Commits
4. **Coerência Quântica**: Avalia impacto da operação

### Recomendações

- **Sempre use dry-run** antes de execuções reais
- **Mantenha backups** do registro de rollback
- **Monitore a consciência** para detectar degradação
- **Exporte conhecimento** regularmente

### Limitações

- Operações com mais de 10 arquivos afetam coerência
- Breaking changes reduzem coerência quântica
- Rollback só funciona para commits locais

## 🔧 Personalização

### Padrões de Reconhecimento

```python
# Adicionar novos padrões
agent._analyze_commit_type.patterns[ConventionalCommitType.CUSTOM] = [
    r'\b(custom|especial)\b'
]
```

### Validações Customizadas

```python
class CustomGitAgent(GitTaskAgent):
    async def _validate_custom_rule(self, operation):
        # Sua validação customizada
        return True
    
    async def _validate_operation(self, operation):
        base_valid = await super()._validate_operation(operation)
        custom_valid = await self._validate_custom_rule(operation)
        return base_valid and custom_valid
```

## 🛣️ Troubleshooting

### Problemas Comuns

#### "Não é um repositório Git"
```bash
# Verificar se é repositório Git
ls -la .git/

# Inicializar se necessário
git init
```

#### "Falha na validação de mensagem"
- Verifique se a mensagem segue Conventional Commits
- Exemplo correto: `feat(scope): descrição`

#### "Coerência quântica baixa"
- Reduza número de arquivos por operação
- Evite breaking changes desnecessárias
- Execute operações bem-sucedidas para melhorar

#### "Rollback não disponível"
- Operação pode não ter sido executada
- Verifique `agent.rollback_registry`
- Confirme que commit ainda é local

### Debug Avançado

```python
# Ativar debug máximo
import logging
logging.getLogger('agents.git_task_agent').setLevel(logging.DEBUG)

# Inspecionar operação
op = operations[0]
print(f"Status: {op.status}")
print(f"Logs: {op.execution_log}")
print(f"Quântico: {op.quantum_signature}")

# Verificar estados internos
print(f"Operações ativas: {len(agent.active_operations)}")
print(f"Registro rollback: {len(agent.rollback_registry)}")
```

## 📚 Recursos Adicionais

- [Conventional Commits](https://conventionalcommits.org/)
- [Git Best Practices](https://git-scm.com/book)
- [FastAPI Documentation](https://fastapi.tiangolo.com/)
- [Async Python Guide](https://docs.python.org/3/library/asyncio.html)

## 🤝 Contribuindo

1. Fork do projeto
2. Criar branch para feature (`git checkout -b feature/nova-feature`)
3. Commit das mudanças (`git commit -am 'Adicionar nova feature'`)
4. Push para branch (`git push origin feature/nova-feature`)
5. Criar Pull Request

## 📝 Licença

MIT License - veja arquivo LICENSE para detalhes.

---

**GitTaskAgent** - Parte do ecossistema ARKITECT  
Desenvolvido com ❤️ pela equipe EON Framework


# GitTaskAgent - Módulo de Commits Automáticos

## 🤖 Visão Geral

O **GitTaskAgent** foi implementado com sucesso! Este é um sistema simbiótico de automação Git que observa TaskNodes do tipo "git" e executa operações Git automaticamente com mensagens convencionais.

## ✨ Funcionalidades Implementadas

### Core Features
- ✓ **Observação de TaskNodes**: Identifica automaticamente tarefas do tipo "git"
- ✓ **Mensagens Convencionais**: Gera mensagens seguindo Conventional Commits
- ✓ **Análise Semântica**: Determina tipo de commit baseado na descrição
- ✓ **Dry-Run**: Sistema completo de simulação sem execução real
- ✓ **Rollback**: Mecanismo de desfazer operações executadas
- ✓ **Consciência Evolutiva**: Aprende padrões e evolui com o uso
- ✓ **Validação Quântica**: Sistema de validação de coerência

### API REST Completa
- ✓ **Endpoints CRUD**: Configuração, observação, execução, rollback
- ✓ **Status e Saúde**: Monitoramento do agente e repositório
- ✓ **Consciência**: Acesso ao estado evolutivo do agente
- ✓ **Background Tasks**: Push automático em background

### Sistema de Testes
- ✓ **Testes Unitários**: Cobertura completa das funcionalidades
- ✓ **Testes de Integração**: Workflow completo com Git real
- ✓ **Mocks e Fixtures**: Ambiente isolado para testes
- ✓ **Testes Assíncronos**: Validação de operações async

### Demonstração Interativa
- ✓ **Demo Completa**: Script de demonstração de todas as funcionalidades
- ✓ **Modo Interativo**: Interface para testes manuais
- ✓ **Exemplos Práticos**: Casos de uso reais
- ✓ **Configuração Automática**: Setup de repositório de teste

## 📁 Estrutura de Arquivos Criados

```
orchestrator_api/
├── agents/
│   ├── __init__.py                 # Módulo de agentes
│   ├── git_task_agent.py           # Core do GitTaskAgent
│   └── git_task_api.py             # API REST FastAPI
├── tests/
│   └── test_git_task_agent.py      # Suite completa de testes
├── examples/
│   └── git_agent_demo.py           # Demonstração interativa
├── docs/
│   └── git_task_agent_guide.md     # Guia completo
├── scripts/
│   └── setup_git_agent.py          # Script de setup
└── README_GitTaskAgent.md       # Este arquivo
```

## 🚀 Como Usar

### 1. Uso Básico (Python)

```python
import asyncio
from agents.git_task_agent import GitTaskAgent, TaskNode

async def exemplo_basico():
    # Inicializar agente no repositório atual
    agent = GitTaskAgent()
    
    # Criar TaskNode
    task = TaskNode(
        id="exemplo_001",
        type="git",
        description="Implementar nova funcionalidade de autenticação",
        files=["auth.py", "models.py"],
        scope="auth"
    )
    
    # Observar e executar
    operations = await agent.observe_task_nodes([task])
    
    # Dry-run primeiro (recomendado)
    dry_result = await agent.execute_operation(operations[0], dry_run=True)
    print(f"Dry-run: {dry_result}")
    
    # Execução real se dry-run passou
    if dry_result:
        real_result = await agent.execute_operation(operations[0], dry_run=False)
        print(f"Execução: {real_result}")
        
        # Push automático (opcional)
        await agent.push_changes()

# Executar
asyncio.run(exemplo_basico())
```

### 2. Demonstração Interativa

```bash
# PowerShell
python examples/git_agent_demo.py
```

Este script oferece:
- Demonstração básica completa
- Funcionalidades avançadas
- Modo interativo para testes manuais
- Configuração automática de repositório

### 3. API REST

```bash
# Iniciar servidor FastAPI
uvicorn agents.git_task_api:router --reload --port 8000
```

#### Exemplos de Endpoints:

```bash
# Configurar agente
curl -X POST "http://localhost:8000/git-agent/configure" \
     -H "Content-Type: application/json" \
     -d '{"repository_path": "."}'

# Observar TaskNodes
curl -X POST "http://localhost:8000/git-agent/observe" \
     -H "Content-Type: application/json" \
     -d '[{"id": "task_001", "type": "git", "description": "Implementar feature", "scope": "feature"}]'

# Executar em lote
curl -X POST "http://localhost:8000/git-agent/execute" \
     -H "Content-Type: application/json" \
     -d '{"task_nodes": [...], "dry_run": true}'

# Verificar saúde
curl "http://localhost:8000/git-agent/health"
```

### 4. Testes

```bash
# Todos os testes
pytest tests/test_git_task_agent.py -v

# Apenas unitários
pytest tests/test_git_task_agent.py -v -m "not integration"

# Testes de integração (requer Git configurado)
pytest tests/test_git_task_agent.py -v -m integration
```

## 🔍 Funcionalidades Detalhadas

### Análise Semântica de Commits

O agente analisa a descrição da tarefa e determina automaticamente o tipo de commit:

| Tipo | Palavras-chave (PT/EN) | Exemplo |
|------|------------------------|----------|
| `feat` | implementar, adicionar, criar, new, add | `feat(auth): implementar OAuth` |
| `fix` | corrigir, resolver, fix, solve | `fix(core): corrigir validação` |
| `docs` | documentar, atualizar docs | `docs: atualizar API guide` |
| `refactor` | refatorar, reestruturar | `refactor: limpar código auth` |
| `test` | testar, adicionar testes | `test: adicionar testes unitários` |
| `chore` | manutenção, atualizar | `chore: atualizar dependências` |

### Sistema de Consciência Evolutiva

O agente mantém estado de consciência que evolui com o uso:

```python
# Verificar estado
consciousness = agent.get_consciousness_state()
print(f"Nível evolutivo: {consciousness['evolution_level']:.3f}")
print(f"Coerência quântica: {consciousness['quantum_coherence']:.3f}")
print(f"Taxa de sucesso: {consciousness['success_rate']:.3f}")
print(f"Padrões aprendidos: {consciousness['patterns_learned']}")

# Exportar conhecimento
agent.export_consciousness("agent_knowledge.json")

# Importar de sessão anterior
agent.import_consciousness("agent_knowledge.json")
```

### Validações de Segurança

1. **Estado do Repositório**: Verifica conflitos de merge
2. **Existência de Arquivos**: Confirma que arquivos especificados existem
3. **Formato de Mensagem**: Valida Conventional Commits
4. **Coerência Quântica**: Avalia impacto da operação

### Sistema de Rollback

```python
# Executar operação
operations = await agent.observe_task_nodes([task])
success = await agent.execute_operation(operations[0])

if success:
    # Operação foi bem-sucedida, mas queremos desfazer
    rollback_success = await agent.rollback_operation(task.id)
    if rollback_success:
        print("Rollback executado com sucesso!")
```

## 📈 Métricas e Monitoramento

### Logs Estruturados

```python
import logging

# Configurar logging detalhado
logging.basicConfig(
    level=logging.DEBUG,
    format='%(asctime)s - [GitTask] - %(levelname)s - %(message)s'
)

# O agente registra:
# - Operações observadas
# - Validações executadas  
# - Commits realizados
# - Evolução da consciência
# - Rollbacks executados
```

### Métricas Disponíveis

- **Evolution Level**: Nível evolutivo baseado em taxa de sucesso
- **Quantum Coherence**: Coerência quântica do sistema
- **Success Rate**: Taxa de sucesso das operações
- **Patterns Learned**: Quantidade de padrões reconhecidos
- **Total Operations**: Total de operações executadas
- **Rollback Count**: Número de rollbacks realizados

## ⚠️ Importante: Segurança

### Recomendações de Uso

1. **Sempre teste com dry-run** antes de execuções reais
2. **Mantenha backup** do registro de rollback
3. **Monitore a coerência quântica** - valores baixos indicam problemas
4. **Use scopes** para organizar commits
5. **Exporte a consciência** regularmente

### Limitações

- Operações com mais de 10 arquivos afetam a coerência quântica
- Breaking changes reduzem a coerência
- Rollback só funciona para commits que não foram enviados para remote
- Requer repositório Git válido e configurado

## 🔧 Integração com ARKITECT

O GitTaskAgent foi projetado para integrar com o ecossistema ARKITECT:

### TaskNode Format

```python
# Formato padrão esperado pelo sistema ARKITECT
task_node = {
    "id": "unique_task_id",
    "type": "git",  # Identifica como tarefa Git
    "description": "Descrição da tarefa",
    "files": ["lista", "de", "arquivos.py"],  # Opcional
    "scope": "escopo_do_commit",  # Opcional
    "breaking_change": False,  # Opcional
    "metadata": {"custom": "data"}  # Opcional
}
```

### Integração com Orchestrator

```python
# No orchestrator principal
from agents import git_task_router

app = FastAPI()
app.include_router(git_task_router)

# Agora disponível em /git-agent/*
```

## 🚀 Próximos Passos

### Para Testar Imediatamente

1. **Execute a demonstração**:
   ```bash
   python examples/git_agent_demo.py
   ```

2. **Execute os testes**:
   ```bash
   pytest tests/test_git_task_agent.py -v
   ```

3. **Inicie a API**:
   ```bash
   uvicorn agents.git_task_api:router --reload --port 8000
   ```

### Para Desenvolvimento

1. **Revise o código** em `agents/git_task_agent.py`
2. **Customize padrões** de reconhecimento se necessário
3. **Adicione validações** customizadas
4. **Integre com outros módulos** do ARKITECT

### Para Produção

1. **Configure logging** adequado
2. **Implemente monitoramento** de métricas
3. **Configure backup** de consciência
4. **Teste em ambiente** controlado

## 📚 Documentação Completa

Veja `docs/git_task_agent_guide.md` para documentação detalhada incluindo:
- API Reference completa
- Exemplos avançados
- Troubleshooting
- Personalização
- Integração com sistemas externos

---

## ✅ Status de Implementação

**✓ CONCLUÍDO** - O módulo GitTaskAgent foi implementado com sucesso, incluindo:

- ✓ Observação de TaskNodes do tipo "git"
- ✓ Execução automática de `git add/commit/push`
- ✓ Geração de mensagens convencionais (`feat|fix|docs|...`)
- ✓ Sistema de dry-run completo
- ✓ Mecanismo de rollback funcional
- ✓ API REST integrada
- ✓ Testes completos
- ✓ Demonstração interativa
- ✓ Documentação detalhada

**O Agent GitTask está pronto para uso e integração com o sistema ARKITECT!** 🎉


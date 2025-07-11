# ARKITECT Makefile
# Automação de tarefas para desenvolvimento e deploy

.PHONY: help install dev build test lint format clean docker-build docker-up docker-down

# Configurações
PYTHON ?= python3
PIP ?= pip
CARGO ?= cargo
DOCKER ?= docker
DOCKER_COMPOSE ?= docker-compose

# Variáveis de ambiente
ARKITECT_ENV ?= development
DOCKER_REGISTRY ?= eonframework
DOCKER_TAG ?= latest

# Cores para output
RED = \033[0;31m
GREEN = \033[0;32m
YELLOW = \033[1;33m
BLUE = \033[0;34m
NC = \033[0m # No Color

help: ## Mostrar ajuda
	@echo "$(BLUE)ARKITECT - Plataforma Simbiótica de Meta-Governança$(NC)"
	@echo ""
	@echo "$(YELLOW)Comandos disponíveis:$(NC)"
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  $(GREEN)%-20s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)Variáveis de ambiente:$(NC)"
	@echo "  ARKITECT_ENV    = $(ARKITECT_ENV)"
	@echo "  DOCKER_REGISTRY = $(DOCKER_REGISTRY)"
	@echo "  DOCKER_TAG      = $(DOCKER_TAG)"

# ========================================
# SETUP E INSTALAÇÃO
# ========================================

install: ## Instalar todas as dependências
	@echo "$(BLUE)Instalando dependências do ARKITECT...$(NC)"
	$(PIP) install --upgrade pip
	$(PIP) install maturin
	$(PIP) install -e ".[dev,testing,docs]"
	maturin develop
	@echo "$(GREEN)Dependências instaladas com sucesso!$(NC)"

install-dev: ## Instalar dependências de desenvolvimento
	@echo "$(BLUE)Instalando dependências de desenvolvimento...$(NC)"
	$(PIP) install -e ".[dev]"
	pre-commit install
	@echo "$(GREEN)Dependências de desenvolvimento instaladas!$(NC)"

setup-env: ## Configurar ambiente de desenvolvimento
	@echo "$(BLUE)Configurando ambiente...$(NC)"
	@if [ ! -f .env ]; then cp .env.example .env; echo "$(YELLOW)Arquivo .env criado. Configure conforme necessário.$(NC)"; fi
	@echo "$(GREEN)Ambiente configurado!$(NC)"

# ========================================
# DESENVOLVIMENTO
# ========================================

dev: ## Iniciar ambiente de desenvolvimento
	@echo "$(BLUE)Iniciando ambiente de desenvolvimento...$(NC)"
	$(DOCKER_COMPOSE) up -d postgres redis minio prometheus grafana
	@echo "$(GREEN)Serviços iniciados. Execute 'make run' para iniciar a API.$(NC)"

run: ## Executar a aplicação localmente
	@echo "$(BLUE)Iniciando ARKITECT API...$(NC)"
	uvicorn arkitect.api.main:app --reload --host 0.0.0.0 --port 8000

run-celery: ## Executar Celery worker
	@echo "$(BLUE)Iniciando Celery worker...$(NC)"
	celery -A arkitect.core.tasks worker --loglevel=info

run-celery-beat: ## Executar Celery beat
	@echo "$(BLUE)Iniciando Celery beat...$(NC)"
	celery -A arkitect.core.tasks beat --loglevel=info

shell: ## Iniciar shell Python com contexto da aplicação
	@echo "$(BLUE)Iniciando shell...$(NC)"
	$(PYTHON) -c "from arkitect.core.config import settings; import arkitect; print('ARKITECT shell ready!')"; $(PYTHON)

# ========================================
# BUILD E COMPILAÇÃO
# ========================================

build: ## Build da aplicação (Rust + Python)
	@echo "$(BLUE)Building ARKITECT...$(NC)"
	$(CARGO) build --release
	maturin build --release
	@echo "$(GREEN)Build concluído!$(NC)"

build-rust: ## Build apenas do componente Rust
	@echo "$(BLUE)Building componente Rust...$(NC)"
	$(CARGO) build --release
	@echo "$(GREEN)Build Rust concluído!$(NC)"

build-python: ## Build apenas do componente Python
	@echo "$(BLUE)Building componente Python...$(NC)"
	maturin build --release
	@echo "$(GREEN)Build Python concluído!$(NC)"

wheel: ## Gerar wheel para distribuição
	@echo "$(BLUE)Gerando wheel...$(NC)"
	maturin build --release --out dist/
	@echo "$(GREEN)Wheel gerado em dist/$(NC)"

# ========================================
# TESTES
# ========================================

test: ## Executar todos os testes
	@echo "$(BLUE)Executando testes...$(NC)"
	$(CARGO) test
	pytest arkitect/tests/ -v --cov=arkitect --cov-report=term-missing
	@echo "$(GREEN)Testes concluídos!$(NC)"

test-rust: ## Executar apenas testes Rust
	@echo "$(BLUE)Executando testes Rust...$(NC)"
	$(CARGO) test
	@echo "$(GREEN)Testes Rust concluídos!$(NC)"

test-python: ## Executar apenas testes Python
	@echo "$(BLUE)Executando testes Python...$(NC)"
	pytest arkitect/tests/ -v --cov=arkitect --cov-report=term-missing
	@echo "$(GREEN)Testes Python concluídos!$(NC)"

test-integration: ## Executar testes de integração
	@echo "$(BLUE)Executando testes de integração...$(NC)"
	$(DOCKER_COMPOSE) up -d
	sleep 10
	pytest arkitect/tests/integration/ -v
	$(DOCKER_COMPOSE) down
	@echo "$(GREEN)Testes de integração concluídos!$(NC)"

test-coverage: ## Gerar relatório de cobertura HTML
	@echo "$(BLUE)Gerando relatório de cobertura...$(NC)"
	pytest arkitect/tests/ --cov=arkitect --cov-report=html
	@echo "$(GREEN)Relatório gerado em htmlcov/index.html$(NC)"

# ========================================
# LINT E FORMATAÇÃO
# ========================================

lint: ## Executar linting em todo o código
	@echo "$(BLUE)Executando linting...$(NC)"
	$(CARGO) clippy -- -D warnings
	flake8 arkitect/
	mypy arkitect/
	@echo "$(GREEN)Linting concluído!$(NC)"

format: ## Formatar código
	@echo "$(BLUE)Formatando código...$(NC)"
	$(CARGO) fmt
	black arkitect/
	isort arkitect/
	@echo "$(GREEN)Código formatado!$(NC)"

format-check: ## Verificar formatação sem modificar
	@echo "$(BLUE)Verificando formatação...$(NC)"
	$(CARGO) fmt -- --check
	black --check arkitect/
	isort --check-only arkitect/
	@echo "$(GREEN)Verificação de formatação concluída!$(NC)"

pre-commit: ## Executar pre-commit hooks
	@echo "$(BLUE)Executando pre-commit...$(NC)"
	pre-commit run --all-files
	@echo "$(GREEN)Pre-commit concluído!$(NC)"

# ========================================
# BANCO DE DADOS
# ========================================

migration: ## Criar nova migração
	@echo "$(BLUE)Criando migração...$(NC)"
	@read -p "Nome da migração: " name; \
	alembic revision --autogenerate -m "$$name"
	@echo "$(GREEN)Migração criada!$(NC)"

migrate: ## Aplicar migrações
	@echo "$(BLUE)Aplicando migrações...$(NC)"
	alembic upgrade head
	@echo "$(GREEN)Migrações aplicadas!$(NC)"

migrate-down: ## Reverter última migração
	@echo "$(BLUE)Revertendo migração...$(NC)"
	alembic downgrade -1
	@echo "$(GREEN)Migração revertida!$(NC)"

db-reset: ## Resetar banco de dados
	@echo "$(YELLOW)ATENÇÃO: Isso vai apagar todos os dados!$(NC)"
	@read -p "Tem certeza? (y/N) " confirm; \
	if [ "$$confirm" = "y" ] || [ "$$confirm" = "Y" ]; then \
		$(DOCKER_COMPOSE) exec postgres psql -U arkitect -d arkitect -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;"; \
		alembic upgrade head; \
		echo "$(GREEN)Banco resetado!$(NC)"; \
	else \
		echo "$(YELLOW)Operação cancelada.$(NC)"; \
	fi

# ========================================
# DOCKER
# ========================================

docker-build: ## Build da imagem Docker
	@echo "$(BLUE)Building imagem Docker...$(NC)"
	$(DOCKER) build -t $(DOCKER_REGISTRY)/arkitect:$(DOCKER_TAG) .
	@echo "$(GREEN)Imagem Docker criada!$(NC)"

docker-build-dev: ## Build da imagem Docker para desenvolvimento
	@echo "$(BLUE)Building imagem Docker de desenvolvimento...$(NC)"
	$(DOCKER) build -f Dockerfile.dev -t $(DOCKER_REGISTRY)/arkitect:dev .
	@echo "$(GREEN)Imagem Docker de desenvolvimento criada!$(NC)"

docker-up: ## Iniciar todos os serviços com Docker Compose
	@echo "$(BLUE)Iniciando serviços...$(NC)"
	$(DOCKER_COMPOSE) up -d
	@echo "$(GREEN)Serviços iniciados!$(NC)"
	@echo "$(YELLOW)Aguarde alguns segundos para os serviços ficarem prontos...$(NC)"
	@echo "$(BLUE)ARKITECT API: http://localhost:8000$(NC)"
	@echo "$(BLUE)Grafana: http://localhost:3000 (admin/arkitect_admin)$(NC)"
	@echo "$(BLUE)Prometheus: http://localhost:9090$(NC)"
	@echo "$(BLUE)MinIO: http://localhost:9001 (arkitect/arkitect_secret)$(NC)"

docker-down: ## Parar todos os serviços
	@echo "$(BLUE)Parando serviços...$(NC)"
	$(DOCKER_COMPOSE) down
	@echo "$(GREEN)Serviços parados!$(NC)"

docker-logs: ## Ver logs dos serviços
	@echo "$(BLUE)Logs dos serviços:$(NC)"
	$(DOCKER_COMPOSE) logs -f

docker-clean: ## Limpar imagens e volumes Docker
	@echo "$(YELLOW)Limpando recursos Docker...$(NC)"
	$(DOCKER_COMPOSE) down -v --rmi all
	$(DOCKER) system prune -f
	@echo "$(GREEN)Limpeza concluída!$(NC)"

# ========================================
# DOCUMENTAÇÃO
# ========================================

docs: ## Gerar documentação
	@echo "$(BLUE)Gerando documentação...$(NC)"
	mkdocs build
	@echo "$(GREEN)Documentação gerada em site/$(NC)"

docs-serve: ## Servir documentação localmente
	@echo "$(BLUE)Servindo documentação...$(NC)"
	mkdocs serve

docs-deploy: ## Deploy da documentação para GitHub Pages
	@echo "$(BLUE)Fazendo deploy da documentação...$(NC)"
	mkdocs gh-deploy
	@echo "$(GREEN)Documentação publicada!$(NC)"

# ========================================
# MONITORAMENTO
# ========================================

metrics: ## Ver métricas da aplicação
	@echo "$(BLUE)Métricas da aplicação:$(NC)"
	curl -s http://localhost:8000/metrics | head -20

health: ## Verificar saúde da aplicação
	@echo "$(BLUE)Verificando saúde da aplicação...$(NC)"
	curl -s http://localhost:8000/health | jq .

status: ## Status dos serviços
	@echo "$(BLUE)Status dos serviços:$(NC)"
	$(DOCKER_COMPOSE) ps

# ========================================
# RELEASE E DEPLOY
# ========================================

version: ## Mostrar versão atual
	@echo "$(BLUE)Versão atual:$(NC)"
	@grep version pyproject.toml | head -1 | cut -d'"' -f2

release: ## Criar release (tag + build + push)
	@echo "$(BLUE)Criando release...$(NC)"
	@echo "$(YELLOW)Implementar script de release$(NC)"
	# TODO: Implementar script de release automatizado

deploy-staging: ## Deploy para staging
	@echo "$(BLUE)Deploy para staging...$(NC)"
	@echo "$(YELLOW)Implementar deploy para staging$(NC)"
	# TODO: Implementar script de deploy

deploy-production: ## Deploy para produção
	@echo "$(RED)Deploy para produção!$(NC)"
	@echo "$(YELLOW)Implementar deploy para produção$(NC)"
	# TODO: Implementar script de deploy de produção

# ========================================
# LIMPEZA E MANUTENÇÃO
# ========================================

clean: ## Limpar arquivos temporários
	@echo "$(BLUE)Limpando arquivos temporários...$(NC)"
	$(CARGO) clean
	rm -rf target/
	rm -rf build/
	rm -rf dist/
	rm -rf *.egg-info/
	rm -rf .pytest_cache/
	rm -rf htmlcov/
	find . -type d -name __pycache__ -exec rm -rf {} +
	find . -type f -name "*.pyc" -delete
	@echo "$(GREEN)Limpeza concluída!$(NC)"

clean-all: clean docker-clean ## Limpeza completa (incluindo Docker)
	@echo "$(GREEN)Limpeza completa concluída!$(NC)"

reset: clean-all install ## Reset completo do ambiente
	@echo "$(GREEN)Ambiente resetado!$(NC)"

# ========================================
# UTILITÁRIOS
# ========================================

check: format-check lint test ## Verificação completa (CI)
	@echo "$(GREEN)Verificação completa concluída!$(NC)"

ci: clean install check ## Simular CI localmente
	@echo "$(GREEN)CI local concluído!$(NC)"

info: ## Informações do sistema
	@echo "$(BLUE)Informações do sistema:$(NC)"
	@echo "Python: $$($(PYTHON) --version)"
	@echo "Pip: $$($(PIP) --version)"
	@echo "Cargo: $$($(CARGO) --version)"
	@echo "Docker: $$($(DOCKER) --version)"
	@echo "Docker Compose: $$($(DOCKER_COMPOSE) --version)"
	@echo "OS: $$(uname -s)"
	@echo "Arch: $$(uname -m)"

benchmark: ## Executar benchmarks
	@echo "$(BLUE)Executando benchmarks...$(NC)"
	$(CARGO) bench
	@echo "$(GREEN)Benchmarks concluídos!$(NC)"

# ========================================
# DESENVOLVIMENTO DE AGENTES
# ========================================

agent-test: ## Testar agentes ARKITECT
	@echo "$(BLUE)Testando agentes...$(NC)"
	python -m arkitect.agents.test_runner
	@echo "$(GREEN)Teste de agentes concluído!$(NC)"

agent-deploy: ## Deploy de agente específico
	@echo "$(BLUE)Deploy de agente...$(NC)"
	@read -p "Nome do agente: " agent; \
	python -m arkitect.agents.deploy "$$agent"
	@echo "$(GREEN)Agente deployado!$(NC)"

# Default target
.DEFAULT_GOAL := help


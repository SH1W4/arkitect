# Multi-stage build for ARKITECT - Plataforma Simbiótica
FROM rust:1.75-slim as rust-builder

# Instalar dependências do sistema para Rust
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Configurar diretório de trabalho
WORKDIR /app

# Copiar arquivos de configuração do Rust
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build do componente Rust
RUN cargo build --release

# Estágio Python
FROM python:3.11-slim as python-builder

# Instalar dependências do sistema
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libssl-dev \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Instalar Rust para maturin
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Instalar maturin
RUN pip install maturin

# Configurar diretório de trabalho
WORKDIR /app

# Copiar arquivos de configuração
COPY pyproject.toml ./
COPY arkitect/ ./arkitect/
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build com maturin
RUN maturin build --release
RUN pip install target/wheels/*.whl

# Estágio final
FROM python:3.11-slim

# Instalar dependências do sistema em runtime
RUN apt-get update && apt-get install -y \
    libssl3 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Criar usuário não-root
RUN groupadd -r arkitect && useradd -r -g arkitect arkitect

# Configurar diretório de trabalho
WORKDIR /app

# Copiar dependências do Python builder
COPY --from=python-builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=python-builder /usr/local/bin /usr/local/bin

# Copiar aplicação
COPY arkitect/ ./arkitect/
COPY pyproject.toml ./

# Instalar dependências Python adicionais
RUN pip install --no-cache-dir fastapi uvicorn[standard] redis asyncpg sqlalchemy alembic

# Configurar permissões
RUN chown -R arkitect:arkitect /app
USER arkitect

# Variáveis de ambiente
ENV PYTHONPATH=/app
ENV PYTHONUNBUFFERED=1
ENV ARKITECT_ENV=production

# Healthcheck
HEALTHCHECK --interval=30s --timeout=30s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8000/health || exit 1

# Expor porta
EXPOSE 8000

# Comando padrão
CMD ["uvicorn", "arkitect.api.main:app", "--host", "0.0.0.0", "--port", "8000"]


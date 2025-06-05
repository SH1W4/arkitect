# Guia de Desenvolvimento

## Ambiente de Desenvolvimento

### Configuração Inicial

1. Configure o ambiente virtual:
   ```powershell
   python -m venv venv
   .\venv\Scripts\Activate
   pip install -r requirements.txt
   ```

2. Configure as variáveis de ambiente:
   - Copie `.env.example` para `.env`
   - Ajuste as variáveis conforme necessário

### Docker

Para desenvolvimento com Docker:

```bash
docker-compose up --build
```

### Redis

O Redis é usado para cache e filas de tarefas. Certifique-se de que está rodando:

- Via Docker: incluído no docker-compose
- Local: necessário instalar separadamente

## Padrões de Código

- Use Black para formatação
- Siga as diretrizes do Flake8
- Mantenha 100% de cobertura de testes
- Documente novas funcionalidades

## Testes

```bash
pytest  # executa todos os testes
pytest --cov=src  # com cobertura
```

## Logs

Os logs são gerenciados pela biblioteca Loguru:
- Console: nível definido em LOG_LEVEL
- Arquivo: rotacionado a cada 10MB

## CI/CD

- PRs devem passar em todos os testes
- Scanning de segurança é obrigatório
- Mantenha as dependências atualizadas


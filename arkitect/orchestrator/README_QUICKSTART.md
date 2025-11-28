# 🚀 ARKITECT OrchestratorAPI - Quick Start

## Início Rápido

### 1. Executar Demo Simplificada
```bash
python quickstart.py
```

Este script:
- ✅ Verifica dependências automaticamente
- 📦 Instala pacotes necessários se ausentes
- 🚀 Inicia uma versão simplificada da API
- 🌐 Disponibiliza endpoints básicos

### 2. Testar API
Após iniciar, acesse:
- **API Root**: http://localhost:8000
- **Documentação**: http://localhost:8000/docs
- **Health Check**: http://localhost:8000/health
- **Métricas Simples**: http://localhost:8000/metrics/simple

### 3. Versão Completa
Para a versão completa com todas as funcionalidades:

```bash
# Instalar todas as dependências
pip install -r requirements.txt

# Executar versão completa
python main.py
```

## 🎯 Funcionalidades da Demo

### Endpoints Disponíveis
- `GET /` - Informações básicas da API
- `GET /health` - Status de saúde
- `GET /metrics/simple` - Métricas básicas do sistema
- `GET /docs` - Documentação Swagger

### Validação Rápida
```bash
# Teste de health
curl http://localhost:8000/health

# Teste de métricas
curl http://localhost:8000/metrics/simple
```

## 🔧 Troubleshooting

### Problemas Comuns
1. **Dependências não instaladas**
   - O script instala automaticamente
   - Se falhar: `pip install fastapi uvicorn pydantic pydantic-settings psutil`

2. **Porta 8000 em uso**
   - Edite `quickstart.py` e mude a porta
   - Ou finalize processo usando a porta: `netstat -ano | findstr :8000`

3. **Python < 3.8**
   - Instale Python 3.8+ 
   - Verifique com: `python --version`

### Próximos Passos

1. **Para desenvolvimento completo**:
   ```bash
   python start.py --reload
   ```

2. **Para produção com Docker**:
   ```bash
   docker-compose up -d
   ```

3. **Para testes automatizados**:
   ```bash
   python test_api.py
   ```

---

**🎉 Parabéns! Sua API está funcionando!**

Para funcionalidades avançadas como WebSocket, gRPC, alertas e administração, consulte o [README.md](README.md) completo.


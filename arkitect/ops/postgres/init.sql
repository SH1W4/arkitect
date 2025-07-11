-- ARKITECT PostgreSQL Initialization Script
-- Configuração inicial do banco de dados

-- Criar extensões necessárias
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
CREATE EXTENSION IF NOT EXISTS "btree_gin";

-- Criar schemas
CREATE SCHEMA IF NOT EXISTS arkitect_core;
CREATE SCHEMA IF NOT EXISTS arkitect_agents;
CREATE SCHEMA IF NOT EXISTS arkitect_monitoring;

-- Configurar permissões
GRANT ALL PRIVILEGES ON SCHEMA arkitect_core TO arkitect;
GRANT ALL PRIVILEGES ON SCHEMA arkitect_agents TO arkitect;
GRANT ALL PRIVILEGES ON SCHEMA arkitect_monitoring TO arkitect;

-- Funções auxiliares
CREATE OR REPLACE FUNCTION arkitect_core.update_modified_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Tabela de configurações do sistema
CREATE TABLE IF NOT EXISTS arkitect_core.system_config (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    key VARCHAR(255) UNIQUE NOT NULL,
    value JSONB NOT NULL,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Trigger para update_modified_column
CREATE TRIGGER update_system_config_modtime
    BEFORE UPDATE ON arkitect_core.system_config
    FOR EACH ROW
    EXECUTE FUNCTION arkitect_core.update_modified_column();

-- Inserir configurações iniciais
INSERT INTO arkitect_core.system_config (key, value, description) VALUES
('system.version', '"0.1.0"', 'Versão atual do sistema ARKITECT'),
('system.environment', '"development"', 'Ambiente de execução'),
('agents.max_concurrent', '10', 'Número máximo de agentes concorrentes'),
('monitoring.enabled', 'true', 'Flag para habilitar monitoramento')
ON CONFLICT (key) DO NOTHING;

-- Índices para performance
CREATE INDEX IF NOT EXISTS idx_system_config_key ON arkitect_core.system_config(key);
CREATE INDEX IF NOT EXISTS idx_system_config_created_at ON arkitect_core.system_config(created_at);

-- Comentários
COMMENT ON SCHEMA arkitect_core IS 'Schema principal do ARKITECT';
COMMENT ON SCHEMA arkitect_agents IS 'Schema para gestão de agentes';
COMMENT ON SCHEMA arkitect_monitoring IS 'Schema para dados de monitoramento';
COMMENT ON TABLE arkitect_core.system_config IS 'Configurações globais do sistema';

-- Mensagem de sucesso
DO $$
BEGIN
    RAISE NOTICE 'ARKITECT database initialized successfully!';
END
$$;


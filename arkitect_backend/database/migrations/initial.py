from sqlalchemy import create_engine, MetaData, Table, Column, Integer, String, Float, DateTime, ForeignKey, JSON
from datetime import datetime

metadata = MetaData()

# Quantum State Table
quantum_states = Table(
    'quantum_states',
    metadata,
    Column('id', Integer, primary_key=True),
    Column('state_id', String, unique=True, nullable=False),
    Column('quantum_state', JSON, nullable=False),
    Column('coherence_level', Float, nullable=False),
    Column('timestamp', DateTime, default=datetime.utcnow),
    Column('metadata', JSON)
)

# Consciousness States Table
consciousness_states = Table(
    'consciousness_states',
    metadata,
    Column('id', Integer, primary_key=True),
    Column('level', String, nullable=False),
    Column('coherence', Float, nullable=False),
    Column('evolution_progress', Integer, nullable=False),
    Column('last_evolution', DateTime, default=datetime.utcnow),
    Column('attributes', JSON)
)

# Tasks Table
tasks = Table(
    'tasks',
    metadata,
    Column('id', Integer, primary_key=True),
    Column('task_id', String, unique=True, nullable=False),
    Column('name', String, nullable=False),
    Column('status', String, nullable=False),
    Column('created_at', DateTime, default=datetime.utcnow),
    Column('updated_at', DateTime, default=datetime.utcnow, onupdate=datetime.utcnow),
    Column('configuration', JSON),
    Column('metrics', JSON)
)

# EON Framework Connections Table
eon_connections = Table(
    'eon_connections',
    metadata,
    Column('id', Integer, primary_key=True),
    Column('framework_id', String, unique=True, nullable=False),
    Column('status', String, nullable=False),
    Column('framework_url', String, nullable=False),
    Column('connected_at', DateTime, default=datetime.utcnow),
    Column('last_sync', DateTime),
    Column('config', JSON)
)

# System Metrics Table
system_metrics = Table(
    'system_metrics',
    metadata,
    Column('id', Integer, primary_key=True),
    Column('timestamp', DateTime, default=datetime.utcnow),
    Column('metric_type', String, nullable=False),
    Column('value', Float, nullable=False),
    Column('context', JSON)
)

# Analytics Summary Table
analytics_summaries = Table(
    'analytics_summaries',
    metadata,
    Column('id', Integer, primary_key=True),
    Column('timestamp', DateTime, default=datetime.utcnow),
    Column('total_tasks', Integer, nullable=False),
    Column('average_execution_time', String, nullable=False),
    Column('system_efficiency', Float, nullable=False),
    Column('quantum_stability', String, nullable=False),
    Column('details', JSON)
)

def create_tables(engine):
    metadata.create_all(engine)

def drop_tables(engine):
    metadata.drop_all(engine)


import pytest
from fastapi.testclient import TestClient
from arkitect_backend.main import app
from arkitect_backend.config import settings
from arkitect_backend.middleware.auth import ARKITECTAuth
from datetime import timedelta

@pytest.fixture
def client():
    return TestClient(app)

@pytest.fixture
def auth_headers():
    auth = ARKITECTAuth()
    token_data = {"sub": "test@example.com", "role": "admin"}
    token = auth.create_token(token_data, expires_delta=timedelta(minutes=30))
    return {"Authorization": f"Bearer {token}"}

def test_health_check(client):
    response = client.get("/health")
    assert response.status_code == 200
    assert response.json()["status"] == "operational"

def test_symbiotic_core(client, auth_headers):
    # Test core initialization
    init_payload = {
        "config": {
            "consciousness_level": "quantum",
            "memory_capacity": "infinite",
            "evolution_rate": 0.95
        }
    }
    response = client.post("/api/v1/core/initialize", json=init_payload, headers=auth_headers)
    assert response.status_code == 200
    assert response.json()["status"] == "success"

    # Test core status
    response = client.get("/api/v1/core/status", headers=auth_headers)
    assert response.status_code == 200
    assert "consciousness_level" in response.json()

def test_quantum_memory(client, auth_headers):
    # Test quantum state storage
    state_payload = {
        "quantum_state": {
            "superposition": [1, 0],
            "entanglement": 0.95
        },
        "coherence_level": 0.99
    }
    response = client.post("/api/v1/memory/store", json=state_payload, headers=auth_headers)
    assert response.status_code == 200
    state_id = response.json()["state_id"]

    # Test quantum state retrieval
    response = client.get(f"/api/v1/memory/retrieve/{state_id}", headers=auth_headers)
    assert response.status_code == 200
    assert response.json()["state_id"] == state_id

def test_consciousness_evolution(client, auth_headers):
    # Test consciousness evolution
    evolution_params = {
        "target_level": "transcendent",
        "evolution_speed": 0.8,
        "coherence_threshold": 0.95
    }
    response = client.post("/api/v1/consciousness/evolve", json=evolution_params, headers=auth_headers)
    assert response.status_code == 200
    assert response.json()["status"] == "success"

    # Test consciousness state
    response = client.get("/api/v1/consciousness/state", headers=auth_headers)
    assert response.status_code == 200
    assert "level" in response.json()
    assert "coherence" in response.json()

def test_eon_framework_integration(client, auth_headers):
    # Test EON Framework connection
    config = {
        "framework_url": "http://localhost:8001",
        "api_key": "test_key",
        "sync_interval": 60
    }
    response = client.post("/api/v1/eon/connect", json=config, headers=auth_headers)
    assert response.status_code == 200
    assert response.json()["status"] == "connected"

    # Test quantum bridge synchronization
    sync_data = {
        "bridge_id": "test_bridge",
        "sync_type": "full",
        "quantum_state": {"coherence": 0.99}
    }
    response = client.post("/api/v1/eon/sync", json=sync_data, headers=auth_headers)
    assert response.status_code == 200
    assert response.json()["status"] == "synchronized"

def test_task_management(client, auth_headers):
    # Test task creation
    task = {
        "name": "quantum_processing",
        "configuration": {
            "priority": "high",
            "quantum_resources": ["q1", "q2"]
        }
    }
    response = client.post("/api/v1/tasks", json=task, headers=auth_headers)
    assert response.status_code == 200
    task_id = response.json()["task_id"]

    # Test task metrics
    response = client.get(f"/api/v1/tasks/{task_id}/metrics", headers=auth_headers)
    assert response.status_code == 200
    assert "execution_time" in response.json()
    assert "quantum_coherence" in response.json()

    # Test analytics summary
    response = client.get("/api/v1/analytics/summary", headers=auth_headers)
    assert response.status_code == 200
    assert "total_tasks" in response.json()
    assert "system_efficiency" in response.json()


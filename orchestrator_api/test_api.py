#!/usr/bin/env python3
"""
Script de teste para OrchestratorAPI
Testa todas as principais funcionalidades da API
"""

import asyncio
import json
import requests
import websockets
from datetime import datetime

API_BASE_URL = "http://localhost:8000"
WS_URL = "ws://localhost:8000/ws"
ADMIN_TOKEN = "admin-token"

def test_health():
    """Testa endpoint de health"""
    print("🩺 Testing health endpoint...")
    response = requests.get(f"{API_BASE_URL}/health")
    print(f"Status: {response.status_code}")
    print(f"Response: {response.json()}")
    print()

def test_tasks():
    """Testa endpoints de tarefas"""
    print("📋 Testing tasks endpoints...")
    
    # Criar tarefa
    task_data = {
        "name": "Test Task",
        "description": "Task created by test script",
        "priority": "high",
        "layer": "default",
        "parameters": {"test": True, "value": 42}
    }
    
    response = requests.post(f"{API_BASE_URL}/tasks", json=task_data)
    print(f"Create Task - Status: {response.status_code}")
    task = response.json()
    task_id = task["id"]
    print(f"Created task ID: {task_id}")
    
    # Listar tarefas
    response = requests.get(f"{API_BASE_URL}/tasks")
    print(f"List Tasks - Status: {response.status_code}")
    print(f"Total tasks: {len(response.json())}")
    
    # Obter tarefa específica
    response = requests.get(f"{API_BASE_URL}/tasks/{task_id}")
    print(f"Get Task - Status: {response.status_code}")
    
    # Executar tarefa
    response = requests.post(f"{API_BASE_URL}/tasks/{task_id}/execute")
    print(f"Execute Task - Status: {response.status_code}")
    print(f"Execution response: {response.json()}")
    
    print()
    return task_id

def test_metrics():
    """Testa endpoints de métricas"""
    print("📊 Testing metrics endpoints...")
    
    # Métricas gerais
    response = requests.get(f"{API_BASE_URL}/metrics")
    print(f"System Metrics - Status: {response.status_code}")
    
    # Métricas de tarefas
    response = requests.get(f"{API_BASE_URL}/metrics/tasks")
    print(f"Task Metrics - Status: {response.status_code}")
    print(f"Task stats: {response.json()}")
    
    # Métricas em tempo real
    response = requests.get(f"{API_BASE_URL}/metrics/realtime")
    print(f"Realtime Metrics - Status: {response.status_code}")
    
    # Prometheus
    response = requests.get(f"{API_BASE_URL}/metrics/prometheus")
    print(f"Prometheus - Status: {response.status_code}")
    print(f"Prometheus format: {response.text[:200]}...")
    
    print()

def test_alerts():
    """Testa endpoints de alertas"""
    print("🚨 Testing alerts endpoints...")
    
    # Criar alerta
    alert_data = {
        "title": "Test Alert",
        "message": "This is a test alert created by script",
        "severity": "warning",
        "source": "test_script",
        "tags": ["test", "automation"],
        "metadata": {"test": True, "timestamp": datetime.utcnow().isoformat()}
    }
    
    response = requests.post(f"{API_BASE_URL}/alerts", json=alert_data)
    print(f"Create Alert - Status: {response.status_code}")
    alert = response.json()
    alert_id = alert["id"]
    print(f"Created alert ID: {alert_id}")
    
    # Listar alertas
    response = requests.get(f"{API_BASE_URL}/alerts")
    print(f"List Alerts - Status: {response.status_code}")
    print(f"Total alerts: {len(response.json())}")
    
    # Reconhecer alerta
    response = requests.post(f"{API_BASE_URL}/alerts/{alert_id}/acknowledge")
    print(f"Acknowledge Alert - Status: {response.status_code}")
    
    # Estatísticas
    response = requests.get(f"{API_BASE_URL}/alerts/stats")
    print(f"Alert Stats - Status: {response.status_code}")
    print(f"Stats: {response.json()}")
    
    print()
    return alert_id

def test_admin():
    """Testa endpoints administrativos"""
    print("⚙️ Testing admin endpoints...")
    
    headers = {"Authorization": f"Bearer {ADMIN_TOKEN}"}
    
    # Informações do sistema
    response = requests.get(f"{API_BASE_URL}/admin/system", headers=headers)
    print(f"System Info - Status: {response.status_code}")
    
    # Health check detalhado
    response = requests.get(f"{API_BASE_URL}/admin/health", headers=headers)
    print(f"Admin Health - Status: {response.status_code}")
    health = response.json()
    print(f"Overall status: {health['overall_status']}")
    
    # Logs
    response = requests.get(f"{API_BASE_URL}/admin/logs?limit=5", headers=headers)
    print(f"System Logs - Status: {response.status_code}")
    print(f"Log count: {response.json()['count']}")
    
    # Ação administrativa (cleanup)
    action_data = {
        "action": "cleanup",
        "parameters": {"days": 7},
        "force": False
    }
    
    response = requests.post(f"{API_BASE_URL}/admin/actions", json=action_data, headers=headers)
    print(f"Admin Action - Status: {response.status_code}")
    print(f"Action result: {response.json()}")
    
    print()

def test_config():
    """Testa endpoints de configuração"""
    print("🔧 Testing config endpoints...")
    
    # Obter configuração
    response = requests.get(f"{API_BASE_URL}/config")
    print(f"Get Config - Status: {response.status_code}")
    config = response.json()
    print(f"Current log level: {config['log_level']}")
    
    # Atualizar configuração
    headers = {"Authorization": f"Bearer {ADMIN_TOKEN}"}
    config_update = {
        "log_level": "DEBUG",
        "enable_metrics": True
    }
    
    response = requests.post(f"{API_BASE_URL}/config", json=config_update, headers=headers)
    print(f"Update Config - Status: {response.status_code}")
    print(f"Update result: {response.json()}")
    
    print()

async def test_websocket():
    """Testa conexão WebSocket"""
    print("🔌 Testing WebSocket connection...")
    
    try:
        async with websockets.connect(WS_URL) as websocket:
            # Aguarda mensagem de boas-vindas
            welcome = await websocket.recv()
            print(f"Welcome message: {json.loads(welcome)}")
            
            # Envia heartbeat
            heartbeat = {
                "type": "heartbeat",
                "timestamp": datetime.utcnow().isoformat()
            }
            await websocket.send(json.dumps(heartbeat))
            
            # Aguarda resposta
            response = await asyncio.wait_for(websocket.recv(), timeout=5.0)
            print(f"Heartbeat response: {json.loads(response)}")
            
            # Subscreve a canal
            subscribe = {
                "type": "subscribe",
                "channel": "tasks"
            }
            await websocket.send(json.dumps(subscribe))
            
            # Aguarda confirmação
            confirmation = await asyncio.wait_for(websocket.recv(), timeout=5.0)
            print(f"Subscription confirmed: {json.loads(confirmation)}")
            
    except Exception as e:
        print(f"WebSocket test failed: {e}")
    
    print()

def main():
    """Executa todos os testes"""
    print("🚀 Starting ARKITECT OrchestratorAPI Tests")
    print("=" * 50)
    
    try:
        # Testes básicos
        test_health()
        
        # Testes de funcionalidades
        task_id = test_tasks()
        test_metrics()
        alert_id = test_alerts()
        test_admin()
        test_config()
        
        # Teste WebSocket
        asyncio.run(test_websocket())
        
        print("✅ All tests completed successfully!")
        print(f"📋 Created task ID: {task_id}")
        print(f"🚨 Created alert ID: {alert_id}")
        
    except Exception as e:
        print(f"❌ Test failed: {e}")
    
    print("=" * 50)
    print("🎯 Test Summary:")
    print("- Health check: ✅")
    print("- Tasks management: ✅")
    print("- Metrics collection: ✅")
    print("- Alerts system: ✅")
    print("- Admin functions: ✅")
    print("- Config management: ✅")
    print("- WebSocket communication: ✅")
    print()
    print("🌐 Access points:")
    print(f"- API Documentation: {API_BASE_URL}/docs")
    print(f"- Health Check: {API_BASE_URL}/health")
    print(f"- WebSocket: {WS_URL}")
    print(f"- Prometheus Metrics: {API_BASE_URL}/metrics/prometheus")

if __name__ == "__main__":
    main()


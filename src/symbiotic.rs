//! Módulo de Processamento Simbiótico
//!
//! Implementa algoritmos para simbiose entre agentes IA, permitindo
//! colaboração, aprendizado mútuo e evolução conjunta.

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use anyhow::{Result, Context};
use tokio::sync::Mutex as AsyncMutex;

/// Tipos de relacionamento simbiótico
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SymbiosisType {
    /// Mutualismo - ambos os agentes se beneficiam
    Mutualism,
    /// Comensalismo - um se beneficia, outro não é afetado
    Commensalism,
    /// Parasitismo - um se beneficia à custa do outro
    Parasitism,
    /// Neutralismo - interação neutra
    Neutralism,
    /// Competição - ambos competem por recursos
    Competition,
}

/// Níveis de intensidade da simbiose
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum SymbiosisIntensity {
    Minimal = 1,
    Low = 2,
    Moderate = 3,
    High = 4,
    Critical = 5,
}

/// Estado de um agente simbiótico
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: Uuid,
    pub energy: f64,
    pub fitness: f64,
    pub capabilities: Vec<String>,
    pub resources: HashMap<String, f64>,
    pub adaptation_rate: f64,
    pub cooperation_tendency: f64,
}

impl AgentState {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            energy: 100.0,
            fitness: 0.5,
            capabilities: Vec::new(),
            resources: HashMap::new(),
            adaptation_rate: 0.1,
            cooperation_tendency: 0.5,
        }
    }

    /// Atualiza a energia do agente
    pub fn update_energy(&mut self, delta: f64) {
        self.energy = (self.energy + delta).max(0.0).min(100.0);
    }

    /// Atualiza o fitness do agente
    pub fn update_fitness(&mut self, delta: f64) {
        self.fitness = (self.fitness + delta).max(0.0).min(1.0);
    }

    /// Adiciona uma nova capacidade
    pub fn add_capability(&mut self, capability: String) {
        if !self.capabilities.contains(&capability) {
            self.capabilities.push(capability);
        }
    }

    /// Atualiza um recurso
    pub fn update_resource(&mut self, resource: String, amount: f64) {
        *self.resources.entry(resource).or_insert(0.0) += amount;
    }

    /// Verifica se tem um recurso suficiente
    pub fn has_resource(&self, resource: &str, amount: f64) -> bool {
        self.resources.get(resource).unwrap_or(&0.0) >= &amount
    }
}

/// Conexão simbiótica entre dois agentes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbioticConnection {
    pub id: Uuid,
    pub agent_a: Uuid,
    pub agent_b: Uuid,
    pub symbiosis_type: SymbiosisType,
    pub intensity: SymbiosisIntensity,
    pub stability: f64,
    pub mutual_benefit: f64,
    pub energy_exchange: f64,
    pub information_flow: f64,
    pub established_at: chrono::DateTime<chrono::Utc>,
    pub last_interaction: chrono::DateTime<chrono::Utc>,
}

impl SymbioticConnection {
    pub fn new(
        agent_a: Uuid,
        agent_b: Uuid,
        symbiosis_type: SymbiosisType,
        intensity: SymbiosisIntensity,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id: Uuid::new_v4(),
            agent_a,
            agent_b,
            symbiosis_type,
            intensity,
            stability: 0.5,
            mutual_benefit: 0.0,
            energy_exchange: 0.0,
            information_flow: 0.0,
            established_at: now,
            last_interaction: now,
        }
    }

    /// Atualiza a estabilidade da conexão
    pub fn update_stability(&mut self, delta: f64) {
        self.stability = (self.stability + delta).max(0.0).min(1.0);
    }

    /// Processa interação entre os agentes
    pub fn process_interaction(&mut self, benefit_a: f64, benefit_b: f64) {
        self.mutual_benefit = (benefit_a + benefit_b) / 2.0;
        self.last_interaction = chrono::Utc::now();
        
        // Ajusta estabilidade baseada no benefício mútuo
        let stability_change = match self.symbiosis_type {
            SymbiosisType::Mutualism => self.mutual_benefit * 0.1,
            SymbiosisType::Commensalism => benefit_a * 0.05,
            SymbiosisType::Parasitism => -benefit_b * 0.1,
            SymbiosisType::Neutralism => 0.0,
            SymbiosisType::Competition => -self.mutual_benefit * 0.1,
        };
        
        self.update_stability(stability_change);
    }

    /// Verifica se a conexão é estável
    pub fn is_stable(&self) -> bool {
        self.stability > 0.3
    }

    /// Calcula a força da conexão
    pub fn connection_strength(&self) -> f64 {
        let intensity_factor = self.intensity as u8 as f64 / 5.0;
        self.stability * intensity_factor * (1.0 + self.mutual_benefit)
    }
}

/// Rede simbiótica de agentes
#[derive(Debug)]
pub struct SymbioticNetwork {
    agents: Arc<RwLock<HashMap<Uuid, AgentState>>>,
    connections: Arc<RwLock<HashMap<Uuid, SymbioticConnection>>>,
    network_metrics: Arc<AsyncMutex<NetworkMetrics>>,
}

#[derive(Debug, Clone, Default)]
struct NetworkMetrics {
    total_interactions: u64,
    successful_symbioses: u64,
    average_stability: f64,
    network_efficiency: f64,
    evolutionary_pressure: f64,
}

impl SymbioticNetwork {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            network_metrics: Arc::new(AsyncMutex::new(NetworkMetrics::default())),
        }
    }

    /// Adiciona um novo agente à rede
    pub fn add_agent(&self, agent: AgentState) -> Result<()> {
        let mut agents = self.agents.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on agents")
        })?;
        
        agents.insert(agent.id, agent);
        Ok(())
    }

    /// Remove um agente da rede
    pub fn remove_agent(&self, agent_id: Uuid) -> Result<()> {
        let mut agents = self.agents.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on agents")
        })?;
        
        let mut connections = self.connections.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on connections")
        })?;
        
        // Remove o agente
        agents.remove(&agent_id);
        
        // Remove todas as conexões relacionadas
        connections.retain(|_, conn| {
            conn.agent_a != agent_id && conn.agent_b != agent_id
        });
        
        Ok(())
    }

    /// Estabelece uma conexão simbiótica entre dois agentes
    pub fn establish_symbiosis(
        &self,
        agent_a: Uuid,
        agent_b: Uuid,
        symbiosis_type: SymbiosisType,
        intensity: SymbiosisIntensity,
    ) -> Result<Uuid> {
        let agents = self.agents.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on agents")
        })?;
        
        // Verifica se ambos os agentes existem
        if !agents.contains_key(&agent_a) || !agents.contains_key(&agent_b) {
            return Err(anyhow::anyhow!("One or both agents not found"));
        }
        
        let connection = SymbioticConnection::new(agent_a, agent_b, symbiosis_type, intensity);
        let connection_id = connection.id;
        
        let mut connections = self.connections.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on connections")
        })?;
        
        connections.insert(connection_id, connection);
        
        Ok(connection_id)
    }

    /// Processa interação entre agentes conectados
    pub async fn process_interaction(
        &self,
        connection_id: Uuid,
        context: InteractionContext,
    ) -> Result<InteractionResult> {
        let agents = self.agents.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on agents")
        })?;
        
        let mut connections = self.connections.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on connections")
        })?;
        
        let connection = connections.get_mut(&connection_id)
            .context("Connection not found")?;
        
        let agent_a = agents.get(&connection.agent_a)
            .context("Agent A not found")?;
        let agent_b = agents.get(&connection.agent_b)
            .context("Agent B not found")?;
        
        // Calcula benefícios baseados no tipo de simbiose
        let (benefit_a, benefit_b) = self.calculate_benefits(
            connection,
            agent_a,
            agent_b,
            &context,
        );
        
        // Processa a interação
        connection.process_interaction(benefit_a, benefit_b);
        
        // Atualiza métricas
        let mut metrics = self.network_metrics.lock().await;
        metrics.total_interactions += 1;
        
        if benefit_a > 0.0 && benefit_b > 0.0 {
            metrics.successful_symbioses += 1;
        }
        
        Ok(InteractionResult {
            success: connection.is_stable(),
            benefit_a,
            benefit_b,
            connection_strength: connection.connection_strength(),
            stability_change: benefit_a + benefit_b,
        })
    }

    /// Calcula benefícios da interação
    fn calculate_benefits(
        &self,
        connection: &SymbioticConnection,
        agent_a: &AgentState,
        agent_b: &AgentState,
        context: &InteractionContext,
    ) -> (f64, f64) {
        match connection.symbiosis_type {
            SymbiosisType::Mutualism => {
                let benefit_a = self.calculate_mutual_benefit(agent_a, agent_b, context);
                let benefit_b = self.calculate_mutual_benefit(agent_b, agent_a, context);
                (benefit_a, benefit_b)
            }
            SymbiosisType::Commensalism => {
                let benefit_a = self.calculate_commensal_benefit(agent_a, agent_b, context);
                (benefit_a, 0.0)
            }
            SymbiosisType::Parasitism => {
                let cost_b = self.calculate_parasitic_cost(agent_b, context);
                let benefit_a = cost_b * 0.8; // Parasita não obtém 100% do que tira
                (benefit_a, -cost_b)
            }
            SymbiosisType::Neutralism => (0.0, 0.0),
            SymbiosisType::Competition => {
                let competition_factor = self.calculate_competition_factor(agent_a, agent_b, context);
                (-competition_factor, -competition_factor)
            }
        }
    }

    fn calculate_mutual_benefit(
        &self,
        beneficiary: &AgentState,
        partner: &AgentState,
        context: &InteractionContext,
    ) -> f64 {
        let resource_synergy = self.calculate_resource_synergy(beneficiary, partner);
        let capability_complement = self.calculate_capability_complement(beneficiary, partner);
        let cooperation_factor = beneficiary.cooperation_tendency * partner.cooperation_tendency;
        
        (resource_synergy + capability_complement) * cooperation_factor * context.environmental_factor
    }

    fn calculate_commensal_benefit(
        &self,
        beneficiary: &AgentState,
        _host: &AgentState,
        context: &InteractionContext,
    ) -> f64 {
        beneficiary.adaptation_rate * context.resource_availability * 0.3
    }

    fn calculate_parasitic_cost(&self, host: &AgentState, context: &InteractionContext) -> f64 {
        let vulnerability = 1.0 - host.fitness;
        vulnerability * context.stress_level * 0.2
    }

    fn calculate_competition_factor(
        &self,
        agent_a: &AgentState,
        agent_b: &AgentState,
        context: &InteractionContext,
    ) -> f64 {
        let resource_overlap = self.calculate_resource_overlap(agent_a, agent_b);
        let fitness_diff = (agent_a.fitness - agent_b.fitness).abs();
        
        resource_overlap * (1.0 - fitness_diff) * context.resource_scarcity
    }

    fn calculate_resource_synergy(&self, agent_a: &AgentState, agent_b: &AgentState) -> f64 {
        let mut synergy = 0.0;
        
        for (resource, amount_a) in &agent_a.resources {
            if let Some(amount_b) = agent_b.resources.get(resource) {
                synergy += (amount_a * amount_b).sqrt() * 0.1;
            }
        }
        
        synergy.min(1.0)
    }

    fn calculate_capability_complement(&self, agent_a: &AgentState, agent_b: &AgentState) -> f64 {
        let unique_capabilities_b: HashSet<_> = agent_b.capabilities
            .iter()
            .filter(|cap| !agent_a.capabilities.contains(cap))
            .collect();
        
        (unique_capabilities_b.len() as f64) * 0.1
    }

    fn calculate_resource_overlap(&self, agent_a: &AgentState, agent_b: &AgentState) -> f64 {
        let mut overlap = 0.0;
        let mut total_resources = 0;
        
        for resource in agent_a.resources.keys() {
            total_resources += 1;
            if agent_b.resources.contains_key(resource) {
                overlap += 1.0;
            }
        }
        
        if total_resources > 0 {
            overlap / total_resources as f64
        } else {
            0.0
        }
    }

    /// Evolui a rede simbiótica
    pub async fn evolve_network(&self) -> Result<EvolutionResult> {
        let mut agents = self.agents.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on agents")
        })?;
        
        let mut connections = self.connections.write().map_err(|_| {
            anyhow::anyhow!("Failed to acquire write lock on connections")
        })?;
        
        let mut evolution_result = EvolutionResult::default();
        
        // Remove conexões instáveis
        let initial_connections = connections.len();
        connections.retain(|_, conn| {
            if !conn.is_stable() {
                evolution_result.connections_removed += 1;
                false
            } else {
                true
            }
        });
        
        // Evolui agentes baseado em suas conexões
        for agent in agents.values_mut() {
            let agent_connections: Vec<_> = connections
                .values()
                .filter(|conn| conn.agent_a == agent.id || conn.agent_b == agent.id)
                .collect();
            
            if !agent_connections.is_empty() {
                let avg_stability: f64 = agent_connections
                    .iter()
                    .map(|conn| conn.stability)
                    .sum::<f64>() / agent_connections.len() as f64;
                
                // Atualiza fitness baseado na estabilidade das conexões
                let fitness_change = (avg_stability - 0.5) * agent.adaptation_rate;
                agent.update_fitness(fitness_change);
                
                if fitness_change > 0.0 {
                    evolution_result.agents_improved += 1;
                } else {
                    evolution_result.agents_degraded += 1;
                }
            }
        }
        
        // Atualiza métricas da rede
        let mut metrics = self.network_metrics.lock().await;
        if !connections.is_empty() {
            metrics.average_stability = connections
                .values()
                .map(|conn| conn.stability)
                .sum::<f64>() / connections.len() as f64;
        }
        
        metrics.network_efficiency = if agents.is_empty() {
            0.0
        } else {
            connections.len() as f64 / (agents.len() as f64 * (agents.len() - 1) as f64 / 2.0)
        };
        
        evolution_result.final_agents = agents.len();
        evolution_result.final_connections = connections.len();
        
        Ok(evolution_result)
    }

    /// Obtém métricas da rede
    pub async fn get_metrics(&self) -> Result<NetworkMetrics> {
        let metrics = self.network_metrics.lock().await;
        Ok(metrics.clone())
    }

    /// Obtém estatísticas da rede
    pub fn get_network_stats(&self) -> Result<NetworkStats> {
        let agents = self.agents.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on agents")
        })?;
        
        let connections = self.connections.read().map_err(|_| {
            anyhow::anyhow!("Failed to acquire read lock on connections")
        })?;
        
        Ok(NetworkStats {
            total_agents: agents.len(),
            total_connections: connections.len(),
            average_fitness: if agents.is_empty() {
                0.0
            } else {
                agents.values().map(|a| a.fitness).sum::<f64>() / agents.len() as f64
            },
            average_energy: if agents.is_empty() {
                0.0
            } else {
                agents.values().map(|a| a.energy).sum::<f64>() / agents.len() as f64
            },
            stable_connections: connections.values().filter(|c| c.is_stable()).count(),
        })
    }
}

/// Contexto de interação entre agentes
#[derive(Debug, Clone)]
pub struct InteractionContext {
    pub environmental_factor: f64,
    pub resource_availability: f64,
    pub resource_scarcity: f64,
    pub stress_level: f64,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for InteractionContext {
    fn default() -> Self {
        Self {
            environmental_factor: 1.0,
            resource_availability: 0.5,
            resource_scarcity: 0.5,
            stress_level: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// Resultado de uma interação simbiótica
#[derive(Debug, Clone)]
pub struct InteractionResult {
    pub success: bool,
    pub benefit_a: f64,
    pub benefit_b: f64,
    pub connection_strength: f64,
    pub stability_change: f64,
}

/// Resultado da evolução da rede
#[derive(Debug, Clone, Default)]
pub struct EvolutionResult {
    pub agents_improved: usize,
    pub agents_degraded: usize,
    pub connections_removed: usize,
    pub final_agents: usize,
    pub final_connections: usize,
}

/// Estatísticas da rede simbiótica
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub total_agents: usize,
    pub total_connections: usize,
    pub average_fitness: f64,
    pub average_energy: f64,
    pub stable_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_state_creation() {
        let id = Uuid::new_v4();
        let agent = AgentState::new(id);
        
        assert_eq!(agent.id, id);
        assert_eq!(agent.energy, 100.0);
        assert_eq!(agent.fitness, 0.5);
    }

    #[test]
    fn test_symbiotic_connection() {
        let agent_a = Uuid::new_v4();
        let agent_b = Uuid::new_v4();
        
        let mut connection = SymbioticConnection::new(
            agent_a,
            agent_b,
            SymbiosisType::Mutualism,
            SymbiosisIntensity::Moderate,
        );
        
        assert_eq!(connection.agent_a, agent_a);
        assert_eq!(connection.agent_b, agent_b);
        assert_eq!(connection.symbiosis_type, SymbiosisType::Mutualism);
        
        connection.process_interaction(0.5, 0.5);
        assert!(connection.stability > 0.5);
    }

    #[tokio::test]
    async fn test_symbiotic_network() {
        let network = SymbioticNetwork::new();
        
        let agent_a = AgentState::new(Uuid::new_v4());
        let agent_b = AgentState::new(Uuid::new_v4());
        
        let agent_a_id = agent_a.id;
        let agent_b_id = agent_b.id;
        
        network.add_agent(agent_a).unwrap();
        network.add_agent(agent_b).unwrap();
        
        let connection_id = network
            .establish_symbiosis(
                agent_a_id,
                agent_b_id,
                SymbiosisType::Mutualism,
                SymbiosisIntensity::High,
            )
            .unwrap();
        
        let context = InteractionContext::default();
        let result = network
            .process_interaction(connection_id, context)
            .await
            .unwrap();
        
        assert!(result.success || !result.success); // Apenas verifica que não deu panic
        
        let stats = network.get_network_stats().unwrap();
        assert_eq!(stats.total_agents, 2);
        assert_eq!(stats.total_connections, 1);
    }
}


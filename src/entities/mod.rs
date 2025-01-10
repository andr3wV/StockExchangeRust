use crate::SimulationError;

pub mod agents;
pub mod companies;

#[derive(Debug, Clone, Default)]
pub struct Balances(Vec<f64>);

impl Balances {
    pub fn get(&self, agent_id: u64) -> Result<f64, SimulationError> {
        let Some(balance) = self.0.get(agent_id as usize) else {
            return Err(SimulationError::AgentNotFound);
        };
        Ok(*balance)
    }
    pub fn add(&mut self, agent_id: u64, amount: f64) -> Result<(), SimulationError> {
        let Some(balance) = self.0.get_mut(agent_id as usize) else {
            return Err(SimulationError::AgentNotFound);
        };
        let result = *balance + amount;
        if result < 0.0 {
            return Err(SimulationError::Unspendable);
        }
        *balance = result;
        Ok(())
    }
}

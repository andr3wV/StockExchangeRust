use stocks::{
    entities::agents::{Agents, Agent, AgentPreferences, AgentHoldings},
    transaction::{Transaction, TodoTransactions},
    trade_house::{TradeAction, Trade},
    SimulationError,
};

#[test]
fn asset_exchange_success() -> Result<(), SimulationError> {
    let agent1 = Agent {
        id: 0,
        balance: 100.0,
        holding: AgentHoldings::default(),
        preferences: AgentPreferences::default(),
    };
    let mut agent2 = Agent {
        id: 1,
        balance: 0.0,
        holding: AgentHoldings::default(),
        preferences: AgentPreferences::default(),
    };
    agent2.holding.0.insert(0, 100);
    let mut agents = Agents::load(&[agent1, agent2]);

    let agent1_buys = TodoTransactions {
        agent_id: 0,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Buy,
        trade: Trade::new(100),
    };
    agents.deduct_assets_from_todotransaction(&agent1_buys)?;
    let agent2_sells = TodoTransactions {
        agent_id: 1,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Sell,
        trade: Trade::new(100),
    };
    agents.deduct_assets_from_todotransaction(&agent2_sells)?;
    let transaction = Transaction::new(0, 1, 0, 100, 1.0);
    agents.exchange_assets_from_transaction(&transaction)?;

    let agents = agents.save()?;
    assert_eq!(agents[0].balance, 0.0);
    assert_eq!(agents[0].holding.0.get(&0), Some(100).as_ref());
    assert_eq!(agents[1].balance, 100.0);
    assert_eq!(agents[1].holding.0.get(&0), Some(0).as_ref());
    Ok(())
}

#[test]
#[should_panic]
fn asset_exchange_failure_1() {
    // Agent 1 has no balance to buy
    let agent1 = Agent {
        id: 0,
        balance: 0.0,
        holding: AgentHoldings::default(),
        preferences: AgentPreferences::default(),
    };
    let mut agent2 = Agent {
        id: 1,
        balance: 0.0,
        holding: AgentHoldings::default(),
        preferences: AgentPreferences::default(),
    };
    agent2.holding.0.insert(0, 100);
    let mut agents = Agents::load(&[agent1, agent2]);

    let agent1_buys = TodoTransactions {
        agent_id: 0,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Buy,
        trade: Trade::new(100),
    };
    agents.deduct_assets_from_todotransaction(&agent1_buys).unwrap();
    let agent2_sells = TodoTransactions {
        agent_id: 1,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Sell,
        trade: Trade::new(100),
    };
    agents.deduct_assets_from_todotransaction(&agent2_sells).unwrap();
    let transaction = Transaction::new(0, 1, 0, 100, 1.0);
    agents.exchange_assets_from_transaction(&transaction).unwrap();

    let agents = agents.save().unwrap();
    assert_eq!(agents[0].balance, 0.0);
    assert_eq!(agents[0].holding.0.get(&0), Some(100).as_ref());
    assert_eq!(agents[1].balance, 100.0);
    assert_eq!(agents[1].holding.0.get(&0), Some(0).as_ref());
}

#[test]
#[should_panic]
fn asset_exchange_failure_2() {
    // Agent 2 has no shares to sell
    let agent1 = Agent {
        id: 0,
        balance: 100.0,
        holding: AgentHoldings::default(),
        preferences: AgentPreferences::default(),
    };
    let agent2 = Agent {
        id: 1,
        balance: 0.0,
        holding: AgentHoldings::default(),
        preferences: AgentPreferences::default(),
    };
    // agent2.holding.0.insert(0, 100);
    let mut agents = Agents::load(&[agent1, agent2]);

    let agent1_buys = TodoTransactions {
        agent_id: 0,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Buy,
        trade: Trade::new(100),
    };
    agents.deduct_assets_from_todotransaction(&agent1_buys).unwrap();
    let agent2_sells = TodoTransactions {
        agent_id: 1,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Sell,
        trade: Trade::new(100),
    };
    agents.deduct_assets_from_todotransaction(&agent2_sells).unwrap();
    let transaction = Transaction::new(0, 1, 0, 100, 1.0);
    agents.exchange_assets_from_transaction(&transaction).unwrap();

    let agents = agents.save().unwrap();
    assert_eq!(agents[0].balance, 0.0);
    assert_eq!(agents[0].holding.0.get(&0), Some(100).as_ref());
    assert_eq!(agents[1].balance, 100.0);
    assert_eq!(agents[1].holding.0.get(&0), Some(0).as_ref());
}

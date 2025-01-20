use stocks::{
    entities::agents::{Agent, Agents},
    trade_house::{Trade, TradeAction},
    transaction::{TodoTransaction, Transaction},
};

#[test]
fn asset_exchange_success() {
    let mut agents = Agents::load(&[
        Agent::new(0, 100.0, &[], &[]),
        Agent::new(1, 0.0, &[(0, 100)], &[]),
    ]);

    let agent1_buys = TodoTransaction {
        agent_id: 0,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Buy,
        trade: Trade::new(100),
    };
    agents
        .deduct_assets_from_todotransaction(&agent1_buys)
        .unwrap();
    let agent2_sells = TodoTransaction {
        agent_id: 1,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Sell,
        trade: Trade::new(100),
    };
    agents
        .deduct_assets_from_todotransaction(&agent2_sells)
        .unwrap();
    let transaction = Transaction::new(0, 1, 0, 100, 1.0);
    agents
        .exchange_assets_from_transaction(&transaction)
        .unwrap();

    let agents = agents.save().unwrap();
    assert_eq!(agents[0].balance, 0.0);
    assert_eq!(agents[0].holding.0.get(&0), Some(100).as_ref());
    assert_eq!(agents[1].balance, 100.0);
    assert_eq!(agents[1].holding.0.get(&0), Some(0).as_ref());
}

#[test]
#[should_panic]
fn asset_exchange_failure_1() {
    // Agent 1 has no balance to buy
    let mut agents = Agents::load(&[
        Agent::new(0, 0.0, &[], &[]),
        Agent::new(0, 0.0, &[(0, 100)], &[]),
    ]);

    let agent1_buys = TodoTransaction {
        agent_id: 0,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Buy,
        trade: Trade::new(100),
    };
    agents
        .deduct_assets_from_todotransaction(&agent1_buys)
        .unwrap();
    let agent2_sells = TodoTransaction {
        agent_id: 1,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Sell,
        trade: Trade::new(100),
    };
    agents
        .deduct_assets_from_todotransaction(&agent2_sells)
        .unwrap();
    let transaction = Transaction::new(0, 1, 0, 100, 1.0);
    agents
        .exchange_assets_from_transaction(&transaction)
        .unwrap();

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
    let mut agents = Agents::load(&[Agent::new(0, 100.0, &[], &[]), Agent::new(0, 0.0, &[], &[])]);

    let agent1_buys = TodoTransaction {
        agent_id: 0,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Buy,
        trade: Trade::new(100),
    };
    agents
        .deduct_assets_from_todotransaction(&agent1_buys)
        .unwrap();
    let agent2_sells = TodoTransaction {
        agent_id: 1,
        company_id: 0,
        strike_price: 1.0,
        action: TradeAction::Sell,
        trade: Trade::new(100),
    };
    agents
        .deduct_assets_from_todotransaction(&agent2_sells)
        .unwrap();
    let transaction = Transaction::new(0, 1, 0, 100, 1.0);
    agents
        .exchange_assets_from_transaction(&transaction)
        .unwrap();

    let agents = agents.save().unwrap();
    assert_eq!(agents[0].balance, 0.0);
    assert_eq!(agents[0].holding.0.get(&0), Some(100).as_ref());
    assert_eq!(agents[1].balance, 100.0);
    assert_eq!(agents[1].holding.0.get(&0), Some(0).as_ref());
}

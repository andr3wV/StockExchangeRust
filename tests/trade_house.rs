use stocks::{
    entities::{
        agents::{Agent, Agents},
        companies::{Companies, Company},
    },
    market::Market,
    trade_house::{Trade, TradeAction},
    transaction::TodoTransaction,
    OFFER_LIFETIME,
};

#[test]
fn getting_added_to_offers_list() {
    let mut agents = Agents::load(&[Agent::new(0, 100.0, &[], &[]), Agent::new(1, 0.0, &[], &[])]);
    let mut companies = Companies::load(&[Company::new(0, 100.0, 0.0, 0.0, (0.0, 0, 0))]);
    let mut market = Market::new();
    market
        .trade(
            false,
            &TodoTransaction {
                agent_id: 0,
                company_id: 0,
                strike_price: 1.0,
                action: TradeAction::Buy,
                trade: Trade::new(10),
            },
            &mut agents,
            &mut companies,
            0.0,
        )
        .unwrap();
    assert_eq!(market.house.get_mut_trade_offers(0).buyer_offers.len(), 1);
}

#[test]
fn offer_resolving() {
    let mut agents = Agents::load(&[Agent::new(0, 100.0, &[], &[]), Agent::new(1, 0.0, &[(0, 100)], &[])]);
    let mut companies = Companies::load(&[Company::new(0, 100.0, 0.0, 0.0, (0.0, 0, 0))]);
    let mut market = Market::new();
    market
        .trade(
            false,
            &TodoTransaction {
                agent_id: 0,
                company_id: 0,
                strike_price: 1.0,
                action: TradeAction::Buy,
                trade: Trade::new(100),
            },
            &mut agents,
            &mut companies,
            0.0,
        )
        .unwrap();
    market
        .trade(
            false,
            &TodoTransaction {
                agent_id: 1,
                company_id: 0,
                strike_price: 1.0,
                action: TradeAction::Sell,
                trade: Trade::new(50),
            },
            &mut agents,
            &mut companies,
            0.0,
        )
        .unwrap();

    // 50 remain
    assert_eq!(market.house.get_mut_trade_offers(0).buyer_offers.len(), 1);
    assert_eq!(market.house.get_mut_trade_offers(0).seller_offers.len(), 0);

    // to put the trade up, the agent needs to give that money
    assert_eq!(agents.balances.get(0).unwrap(), 0.0);
    assert_eq!(agents.balances.get(1).unwrap(), 50.0); 

    assert_eq!(agents.holdings.get(0, 0), 50);
    assert_eq!(agents.holdings.get(1, 0), 50);
}

#[test]
fn offer_refund() {
    let mut agents = Agents::load(&[Agent::new(0, 100.0, &[], &[])]);
    let mut companies = Companies::load(&[Company::new(0, 100.0, 0.0, 0.0, (0.0, 0, 0))]);
    let mut market = Market::new();
    market
        .trade(
            false,
            &TodoTransaction {
                agent_id: 0,
                company_id: 0,
                strike_price: 1.0,
                action: TradeAction::Buy,
                trade: Trade::new(100),
            },
            &mut agents,
            &mut companies,
            0.0,
        )
        .unwrap();

    for _ in 0..(OFFER_LIFETIME - 1) {
        market.house.tick();
    }
    let mut tick_data = market.house.tick();
    let failed_offers = tick_data.0.get_mut(&0).unwrap();
    let failed_offer_data = failed_offers.pop();
    let failed_offer = failed_offer_data.unwrap();

    assert_eq!(failed_offer.0.data.number_of_shares, 100);
    assert_eq!(failed_offer.0.offerer_id, 0);
    assert_eq!(failed_offer.0.strike_price, 1.0);
    assert_eq!(failed_offer.1, TradeAction::Buy);
}

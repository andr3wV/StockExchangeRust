use stocks::{
    entities::{
        agents::{Agent, Agents},
        companies::{Companies, Company},
    },
    market::Market,
    trade_house::{Trade, TradeAction},
    transaction::TodoTransaction,
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

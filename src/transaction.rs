use crate::{
    entities::agents::Agents,
    log,
    logger::Log,
    market::Market,
    trade_house::{Trade, TradeAction},
    SimulationError,
};
use rand::Rng;
use serde::{Deserialize, Serialize};

// Todo Decide if you want to store the Transaction
/// Represents an exchange of captial between 2 agents
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    /// The agent which gave away his shares
    pub buyer_id: u64,
    /// The agent which bought the shares
    pub seller_id: u64,

    pub company_id: u64,
    pub number_of_shares: u64,
    /// The price per share at which the exchange was done
    pub strike_price: f64,
}

/// Represents an exchange of captial between agent & company
#[derive(Serialize, Deserialize, Debug)]
pub struct CompanyTransaction {
    /// The agent which gave away his shares
    pub buyer_agent_id: u64,
    /// The agent which bought the shares
    pub seller_company_id: u64,

    pub number_of_shares: u64,
    /// The price per share at which the exchange was done
    pub strike_price: f64,
}

pub struct TodoTransactions {
    pub agent_id: u64,
    pub company_id: u64,
    pub strike_price: f64,
    pub action: TradeAction,
    pub trade: Trade,
}

impl Transaction {
    pub fn new(
        buyer_id: u64,
        seller_id: u64,
        company_id: u64,
        number_of_shares: u64,
        strike_price: f64,
    ) -> Self {
        log!(info "Transaction: buyer_id: {}, seller_id: {}, company_id: {}, number_of_shares: {}, strike_price: {}", buyer_id, seller_id, company_id, number_of_shares, strike_price);
        Self {
            buyer_id,
            seller_id,
            company_id,
            number_of_shares,
            strike_price,
        }
    }
}

impl TodoTransactions {
    pub fn trade(
        &self,
        market: &mut Market,
        agents: &mut Agents,
        rng: &mut impl Rng,
    ) -> Result<(), SimulationError> {
        let result = market.trade(
            self.agent_id,
            self.company_id,
            self.strike_price,
            5.0,
            &self.trade,
            self.action,
        );
        if self.action == TradeAction::Sell {
            agents
                .holdings
                .pop(self.agent_id, self.company_id, self.trade.number_of_shares)?;
        } else {
            if agents.balances.get(self.agent_id)?
                < self.strike_price * (self.trade.number_of_shares as f64)
            {
                return Err(SimulationError::Unspendable);
            }
            agents.balances.add(
                self.agent_id,
                -(self.strike_price * (self.trade.number_of_shares as f64)),
            )?;
        }

        match result {
            Ok(action_state) => {
                agents.handle_action_state(action_state, market, self.company_id)?
            }
            Err(offer_idxs) => handle_offer_idxs(offer_idxs, market, rng, self),
        }
        Ok(())
    }
}
fn handle_offer_idxs(
    offer_idxs: Vec<usize>,
    market: &mut Market,
    rng: &mut impl Rng,
    transaction: &TodoTransactions,
) {
    // 30% chance of accept this offer
    if rng.gen_ratio(7, 10) {
        return;
    }

    let target_offers = match transaction.action {
        TradeAction::Buy => {
            &market
                .house
                .get_mut_trade_offers(transaction.company_id)
                .seller_offers
        }
        TradeAction::Sell => {
            &market
                .house
                .get_mut_trade_offers(transaction.company_id)
                .buyer_offers
        }
    };

    // choose a random offer
    let offer_idx = offer_idxs[rng.gen_range(0..offer_idxs.len())];
    let offer = target_offers[offer_idx].clone();

    let (new_transaction, extra_shares_left) = market.trade_offer(
        transaction.company_id,
        &offer,
        transaction.agent_id,
        &transaction.trade,
        transaction.action,
    );
    if extra_shares_left > 0 {
        market.house.add_trade_offer(
            transaction.agent_id,
            transaction.company_id,
            transaction.strike_price,
            Trade::new(extra_shares_left),
            transaction.action,
        );
    }
    market.add_transaction(transaction.company_id, new_transaction.strike_price);
}

impl CompanyTransaction {
    pub fn new(
        buyer_agent_id: u64,
        seller_company_id: u64,
        number_of_shares: u64,
        strike_price: f64,
    ) -> Self {
        log!(info "CompanyTransaction: buyer_agent_id: {}, seller_company_id: {}, number_of_shares: {}, strike_price: {}", buyer_agent_id, seller_company_id, number_of_shares, strike_price);
        Self {
            buyer_agent_id,
            seller_company_id,
            number_of_shares,
            strike_price,
        }
    }
    pub fn trade(&self, market: &mut Market, agents: &mut Agents, rng: &mut impl Rng) {}
}

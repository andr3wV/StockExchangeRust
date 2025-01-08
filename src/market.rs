use std::collections::HashMap;

use crate::{
    entities::MarketValue,
    max, min,
    trade_house::{FailedOffer, Offer, StockOption, Trade, TradeAction, TradeHouse},
    transaction::Transaction,
};
use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Market {
    /// If the interval is set for 5 seconds
    /// Have a function, called something like tick().
    /// When it is called, the maximum and minimum values in the vec are stored in
    /// `highest_price` and `lowest_price`. And the average is set at `current_price`
    /// Also calculate the standard deviation and store it in `standard_deviation`
    recent_transactions: HashMap<u64, Vec<f64>>,
    pub house: TradeHouse,
}

#[derive(Debug)]
pub enum ActionState {
    AddedToOffers,
    InstantlyResolved(Transaction),
    PartiallyResolved(Transaction),
}

impl Market {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn trade(
        &mut self,
        agent_id: u64,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        trade: &Trade,
        action: TradeAction,
    ) -> Result<ActionState, Vec<usize>> {
        let appropriate_trade_offer = self.house.get_appropriate_trade_offer(
            company_id,
            strike_price,
            acceptable_strike_price_deviation,
            action.complement(),
        );

        let all_offers = self.house.get_mut_trade_offers(company_id);
        let Some(offer_idxs) = appropriate_trade_offer else {
            self.house
                .add_trade_offer(agent_id, company_id, strike_price, trade.clone(), action);
            return Ok(ActionState::AddedToOffers);
        };
        if offer_idxs.is_empty() {
            self.house
                .add_trade_offer(agent_id, company_id, strike_price, trade.clone(), action);
            return Ok(ActionState::AddedToOffers);
        }
        let target_offer = match action {
            TradeAction::Buy => &mut all_offers.seller_offers,
            TradeAction::Sell => &mut all_offers.buyer_offers,
        };

        for offer_idx in offer_idxs.iter() {
            let offer = target_offer[*offer_idx].clone();
            // Don't autoresolve if it can be slightly worse for us
            if offer.strike_price > strike_price {
                continue;
            }

            target_offer.remove(*offer_idx);
            let (transaction, extra_shares_left) =
                self.trade_offer(company_id, &offer, agent_id, trade, action.clone());

            if extra_shares_left == 0 {
                return Ok(ActionState::InstantlyResolved(transaction));
            }

            self.house.add_trade_offer(
                agent_id,
                company_id,
                strike_price,
                Trade::new(extra_shares_left),
                action,
            );

            return Ok(ActionState::PartiallyResolved(transaction));
        }
        Err(offer_idxs)
    }

    /// ! Does not remove the offer from the trade house
    pub fn trade_offer(
        &mut self,
        company_id: u64,
        offer: &Offer<Trade>,
        accepter_id: u64,
        trade: &Trade,
        action: TradeAction,
    ) -> (Transaction, u64) {
        let (buyer_id, seller_id) = match action {
            TradeAction::Buy => (accepter_id, offer.offerer_id),
            TradeAction::Sell => (offer.offerer_id, accepter_id),
        };

        if offer.data.number_of_shares == trade.number_of_shares {
            return (
                Transaction::new(
                    buyer_id,
                    seller_id,
                    company_id,
                    trade.number_of_shares,
                    offer.strike_price,
                ),
                0,
            );
        }

        let transaction = if offer.data.number_of_shares > trade.number_of_shares {
            self.house.add_trade_offer(
                offer.offerer_id,
                company_id,
                offer.strike_price,
                Trade::new(offer.data.number_of_shares - trade.number_of_shares),
                TradeAction::Sell,
            );
            Transaction::new(
                buyer_id,
                seller_id,
                company_id,
                trade.number_of_shares,
                offer.strike_price,
            )
        } else {
            Transaction::new(
                buyer_id,
                seller_id,
                company_id,
                offer.data.number_of_shares,
                offer.strike_price,
            )
        };
        (
            transaction,
            max(0, trade.number_of_shares - offer.data.number_of_shares),
        )
    }

    pub fn add_transaction(&mut self, company_id: u64, price: f64) {
        let tracker = self.recent_transactions.entry(company_id).or_default();
        tracker.push(price);
    }
    pub fn tick_individual_company(&mut self, company_id: u64, market_value: &mut MarketValue) {
        let recent_transactions = self.recent_transactions.entry(company_id).or_default();
        if recent_transactions.is_empty() {
            market_value.highest_price = market_value.current_price;
            market_value.lowest_price = market_value.current_price;
            return;
        }
        let max: f64 = recent_transactions
            .iter()
            .fold(recent_transactions[0], |a, &b| max(a, b));
        let min: f64 = recent_transactions
            .iter()
            .fold(recent_transactions[0], |a, &b| min(a, b));
        let sum: f64 = recent_transactions.iter().sum();
        let avg = sum / (recent_transactions.len() as f64);

        market_value.highest_price = max;
        market_value.lowest_price = min;
        market_value.overall_movement_start = market_value.overall_movement_end;
        market_value.current_price = avg;
        market_value.overall_movement_end = recent_transactions.last().unwrap().clone();

        self.recent_transactions.clear();
    }

    pub fn tick_failures(
        &mut self,
        expired_trades: &mut HashMap<u64, Vec<FailedOffer<Trade>>>,
        expired_options: &mut HashMap<u64, Vec<FailedOffer<StockOption>>>,
    ) {
        let house_tick_data = self.house.tick();
        expired_trades.extend(house_tick_data.0);
        expired_options.extend(house_tick_data.1);
    }
}

impl MarketValue {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn rand() -> Self {
        Self {
            current_price: random::<f64>() * 100.0,
            highest_price: random::<f64>() * 100.0,
            lowest_price: random::<f64>() * 100.0,
            overall_movement_start: random::<f64>() * 100.0,
            overall_movement_end: random::<f64>() * 100.0,
        }
    }
}

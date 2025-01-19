use crate::{
    entities::{agents::Agents, companies::Companies, companies::MarketValue},
    max, min,
    trade_house::{FailedOffer, Offer, StockOption, Trade, TradeAction, TradeHouse},
    transaction::{TodoTransactions, Transaction},
    SimulationError,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

    pub fn rand_do_trade(
        &mut self,
        rng: &mut impl Rng,
        agents: &mut Agents,
        companies: &mut Companies,
        transactions: &mut [TodoTransactions],
    ) -> Result<(), SimulationError> {
        for todo_transaction in transactions.iter() {
            let Ok(Some(possible_offers)) = self.trade(rng.gen_ratio(6, 10), todo_transaction, agents, companies, 5.0)
            else {
                continue;
            };

            if rng.gen_ratio(7, 10) {
                continue;
            }

            let offer_idx = rng.gen_range(0..possible_offers.len());
            let all_offers = self.house.get_mut_trade_offers(todo_transaction.company_id);
            let target_offers = match todo_transaction.action {
                TradeAction::Buy => &mut all_offers.seller_offers,
                TradeAction::Sell => &mut all_offers.buyer_offers,
            };
            let offer = target_offers.remove(offer_idx);

            let transaction = self
                .convert_trade_offer_and_todo_transaction_to_transaction(&offer, todo_transaction);
            self.add_transaction(todo_transaction.company_id, transaction.strike_price);
            agents.exchange_assets_from_transaction(&transaction)?;
        }
        Ok(())
    }
    pub fn trade(
        &mut self,
        willing_to_accept_company_shares_if_they_are_present: bool,
        todo_transaction: &TodoTransactions,
        agents: &mut Agents,
        companies: &mut Companies,
        acceptable_strike_price_deviation: f64,
    ) -> Result<Option<Vec<Offer<Trade>>>, SimulationError> {
        agents.deduct_assets_from_todotransaction(todo_transaction)?;

        //
        if
            companies.check_lots_from_todotransaction(todo_transaction) &&
            willing_to_accept_company_shares_if_they_are_present
        {
            companies.add_bet_from_todotransaction(todo_transaction);
            return Ok(None);
        }

        // Check if there is an appropriate trade offer
        let appropriate_trade_offer = self.house.get_appropriate_trade_offer(
            todo_transaction.company_id,
            todo_transaction.strike_price,
            acceptable_strike_price_deviation,
            todo_transaction.action.complement(),
        );

        let Some(offer_idxs) = appropriate_trade_offer else {
            // If none, then add trade to the house
            self.house
                .add_trade_offer_from_todo_transaction(todo_transaction);
            return Ok(None);
        };
        if offer_idxs.is_empty() {
            // If none, then add trade to the house
            self.house
                .add_trade_offer_from_todo_transaction(todo_transaction);
            return Ok(None);
        }
        let all_offers = self.house.get_mut_trade_offers(todo_transaction.company_id);
        let target_offers = match todo_transaction.action {
            TradeAction::Buy => &mut all_offers.seller_offers,
            TradeAction::Sell => &mut all_offers.buyer_offers,
        };

        for offer_idx in offer_idxs.iter() {
            let offer = target_offers[*offer_idx].clone();
            // Don't autoresolve if it can be slightly worse for us
            if offer.strike_price > todo_transaction.strike_price {
                continue;
            }

            // Resolve the offer
            target_offers.remove(*offer_idx);

            let transaction = self
                .convert_trade_offer_and_todo_transaction_to_transaction(&offer, todo_transaction);
            self.add_transaction(todo_transaction.company_id, transaction.strike_price);
            agents.exchange_assets_from_transaction(&transaction)?;
            return Ok(None);
        }

        return Ok(Some(
            offer_idxs
                .iter()
                .map(|idx| target_offers[*idx].clone())
                .collect(),
        ));
    }

    pub fn convert_trade_offer_and_todo_transaction_to_transaction(
        &mut self,
        offer: &Offer<Trade>,
        todo_transaction: &TodoTransactions,
    ) -> Transaction {
        let (buyer_id, seller_id) = match todo_transaction.action {
            TradeAction::Buy => (todo_transaction.agent_id, offer.offerer_id),
            TradeAction::Sell => (offer.offerer_id, todo_transaction.agent_id),
        };

        if offer.data.number_of_shares == todo_transaction.trade.number_of_shares {
            self.house
                .remove_trade_offer(todo_transaction.company_id, offer.clone());
            return Transaction::new(
                buyer_id,
                seller_id,
                todo_transaction.company_id,
                todo_transaction.trade.number_of_shares,
                offer.strike_price,
            );
        }

        if offer.data.number_of_shares > todo_transaction.trade.number_of_shares {
            self.house.add_trade_offer(
                offer.offerer_id,
                todo_transaction.company_id,
                offer.strike_price,
                Trade::new(offer.data.number_of_shares - todo_transaction.trade.number_of_shares),
                TradeAction::Sell,
            );
            return Transaction::new(
                buyer_id,
                seller_id,
                todo_transaction.company_id,
                todo_transaction.trade.number_of_shares,
                offer.strike_price,
            );
        }
        self.house.add_trade_offer(
            todo_transaction.agent_id,
            todo_transaction.company_id,
            todo_transaction.strike_price,
            Trade::new(todo_transaction.trade.number_of_shares - offer.data.number_of_shares),
            todo_transaction.action,
        );
        Transaction::new(
            buyer_id,
            seller_id,
            todo_transaction.company_id,
            offer.data.number_of_shares,
            offer.strike_price,
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
        market_value.overall_movement_end = *recent_transactions.last().unwrap();

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
    pub fn rand(rng: &mut impl Rng) -> Self {
        Self {
            current_price: rng.gen_range(0.0..100.0),
            highest_price: rng.gen_range(0.0..100.0),
            lowest_price: rng.gen_range(0.0..100.0),
            overall_movement_start: rng.gen_range(0.0..100.0),
            overall_movement_end: rng.gen_range(0.0..100.0),
        }
    }
}

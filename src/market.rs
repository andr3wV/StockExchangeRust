use std::collections::HashMap;

use crate::{max, min, trade_house::{Offer, OfferAsk, Trade, TradeHouse}, transaction::Transaction};
use rand::random;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Market {
    market_values: HashMap<u64, MarketValueTracker>,
    pub house: TradeHouse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketValue {
/// Current price of a stock as shown for display purposes
    pub current_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub overall_movement_start: f64,
    pub overall_movement_end: f64,
}

#[derive(Serialize, Deserialize, Debug)]
struct MarketValueTracker {
    market_value: MarketValue,
    /// If the interval is set for 5 seconds
    /// Have a function, called something like tick().
    /// When it is called, the maximum and minimum values in the vec are stored in
    /// `highest_price` and `lowest_price`. And the average is set at `current_price`
    /// Also calculate the standard deviation and store it in `standard_deviation`
    recent_transactions_strike_prices: Vec<f64>,
}

#[derive(Debug)]
pub enum ActionState {
    AddedToOffers,
    InstantlyResolved(Transaction),
    PartiallyResolved(Transaction),
}

impl Market {
    pub fn new() -> Self {
        Self {
            house: TradeHouse::new(),
            market_values: HashMap::new(),
        }
    }

    pub fn load_market_values(&mut self, market_values: &HashMap<u64, MarketValue>) {
        for (company_id, market_value) in market_values.iter() {
            let mut tracker = MarketValueTracker::new();
            tracker.market_value = market_value.clone();
            self.market_values.insert(company_id.clone(), tracker);
        }
    }

    pub fn buy_trade(
        &mut self,
        agent_id: u64,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        trade: &Trade,
      ) -> Result<ActionState, Vec<usize>>  {
        let appropriate_trade_offer = self.house.get_appropriate_trade_offer(
            company_id,
            strike_price,
            acceptable_strike_price_deviation,
            OfferAsk::Sell,
        );

        let all_offers = self.house.get_mut_trade_offers(company_id);
        let Some(offer_idxs) = appropriate_trade_offer else {
            self.house.add_trade_offer(agent_id, company_id, strike_price, trade.clone(), OfferAsk::Buy);
            return Ok(ActionState::AddedToOffers);
        };
        if offer_idxs.len() == 0 {
            self.house.add_trade_offer(agent_id, company_id, strike_price, trade.clone(), OfferAsk::Buy);
            return Ok(ActionState::AddedToOffers);
        }
        for offer_idx in offer_idxs.iter() {
            let offer = all_offers.seller_offers[*offer_idx].clone();
            // Don't autoresolve if it can be slightly worse for us
            if offer.strike_price > strike_price {
                continue;
            }

            all_offers.seller_offers.remove(*offer_idx);
            let (transaction, extra_shares_left) = self.buy_trade_offer(company_id, &offer, agent_id, trade);
            if extra_shares_left == 0 {
                return Ok(ActionState::InstantlyResolved(transaction));
            }

            self.house.add_trade_offer(
                agent_id,
                company_id,
                strike_price,
                Trade::new(extra_shares_left),
                OfferAsk::Buy,
            );

            return Ok(ActionState::PartiallyResolved(transaction));
        }
        return Err(offer_idxs);
    }

    pub fn sell_trade(
        &mut self,
        agent_id: u64,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        trade: &Trade,
    ) -> Result<ActionState, Vec<usize>> {
        let appropriate_trade_offer = self.house.get_appropriate_trade_offer(
            company_id,
            strike_price,
            acceptable_strike_price_deviation,
            OfferAsk::Buy,
        );
        let all_offers = self.house.get_mut_trade_offers(company_id);
        let Some(offer_idxs) = appropriate_trade_offer else {
            self.house.add_trade_offer(agent_id, company_id, strike_price, trade.clone(), OfferAsk::Sell);
            return Ok(ActionState::AddedToOffers);
        };
        if offer_idxs.len() == 0 {
            self.house.add_trade_offer(agent_id, company_id, strike_price, trade.clone(), OfferAsk::Sell);
            return Ok(ActionState::AddedToOffers);
        }
        for offer_idx in offer_idxs.iter() {
            let offer = all_offers.buyer_offers[*offer_idx].clone();
            // Don't autoresolve if it can be slightly worse for us
            if offer.strike_price < strike_price {
                continue;
            }
            all_offers.buyer_offers.remove(*offer_idx);
            let (transaction, extra_shares_left) = self.sell_trade_offer(company_id, &offer, agent_id, trade);
            if extra_shares_left == 0 {
                return Ok(ActionState::InstantlyResolved(transaction));
            }
            self.house.add_trade_offer(
                agent_id,
                company_id,
                strike_price,
                Trade::new(extra_shares_left),
                OfferAsk::Sell,
            );
            return Ok(ActionState::PartiallyResolved(transaction));
        }
        return Err(offer_idxs);
    }

    /// ! Does not remove the offer from the trade house
    pub fn buy_trade_offer(&mut self, company_id: u64, offer: &Offer<Trade>, accepter_id: u64, trade: &Trade) -> (Transaction, u64) {
        if offer.data.number_of_shares == trade.number_of_shares {
            return (Transaction::new(
                accepter_id,
                offer.offerer_id,
                company_id,
                trade.number_of_shares,
                offer.strike_price,
            ), 0);
        }

        let transaction;
        if offer.data.number_of_shares > trade.number_of_shares {
            self.house.add_trade_offer(
                offer.offerer_id,
                company_id,
                offer.strike_price,
                Trade::new(offer.data.number_of_shares - trade.number_of_shares),
                OfferAsk::Sell,
            );
            transaction = Transaction::new(
                accepter_id,
                offer.offerer_id,
                company_id,
                trade.number_of_shares,
                offer.strike_price,
            );
        } else {
            transaction = Transaction::new(
                accepter_id,
                offer.offerer_id,
                company_id,
                offer.data.number_of_shares,
                offer.strike_price,
            );
        }
        return (transaction, max(0, trade.number_of_shares - offer.data.number_of_shares));
    }

    pub fn sell_trade_offer(&mut self, company_id: u64, offer: &Offer<Trade>, accepter_id: u64, trade: &Trade) -> (Transaction, u64) {
        if offer.data.number_of_shares == trade.number_of_shares {
            return (Transaction::new(
                offer.offerer_id,
                accepter_id,
                company_id,
                trade.number_of_shares,
                offer.strike_price,
            ), 0);
        }

        let transaction;
        if offer.data.number_of_shares > trade.number_of_shares {
            self.house.add_trade_offer(
                offer.offerer_id,
                company_id,
                offer.strike_price,
                Trade::new(offer.data.number_of_shares - trade.number_of_shares),
                OfferAsk::Buy,
            );
            transaction = Transaction::new(
                offer.offerer_id,
                accepter_id,
                company_id,
                trade.number_of_shares,
                offer.strike_price,
            );
        } else {
            transaction = Transaction::new(
                offer.offerer_id,
                accepter_id,
                company_id,
                offer.data.number_of_shares,
                offer.strike_price,
            );
        }
        return (transaction, max(0, trade.number_of_shares - offer.data.number_of_shares));
    }

    pub fn add_transaction(&mut self, company_id: u64, price: f64) {
        if !self.market_values.contains_key(&company_id) {
            self.market_values.insert(company_id, MarketValueTracker::new());
        }
        self.market_values.get_mut(&company_id).unwrap().add_transaction(price);
    }

    pub fn tick(&mut self) -> HashMap<u64, MarketValue> {
        let mut market_values: HashMap<u64, MarketValue> = HashMap::new();
        for (id, market_value_tracker) in self.market_values.iter_mut() {
            market_values.insert(*id, market_value_tracker.tick());
        }
        return market_values;
    }
}

impl MarketValue {
    pub fn new() -> Self {
        Self {
            current_price: 0.0,
            highest_price: 0.0,
            lowest_price: 0.0,
            overall_movement_start: 0.0,
            overall_movement_end: 0.0,
        }
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

impl MarketValueTracker {
    pub fn new() -> Self {
        Self {
            market_value: MarketValue::new(),
            recent_transactions_strike_prices: Vec::new(),
        }
    }

    pub fn add_transaction(&mut self, price: f64) {
        self.recent_transactions_strike_prices.push(price);
    }

    pub fn tick(&mut self) -> MarketValue {
        if self.recent_transactions_strike_prices.is_empty() {
            self.market_value.highest_price = self.market_value.current_price;
            self.market_value.lowest_price = self.market_value.current_price;
            return self.market_value.clone();
        }
        let max = self.recent_transactions_strike_prices.iter().fold(self.recent_transactions_strike_prices[0], |a, &b| max(a, b));
        let min = self.recent_transactions_strike_prices.iter().fold(self.recent_transactions_strike_prices[0], |a, &b| min(a, b));
        let sum: f64 = self.recent_transactions_strike_prices.iter().sum();
        let avg = sum / (self.recent_transactions_strike_prices.len() as f64);

        self.market_value.highest_price = max;
        self.market_value.lowest_price = min;
        self.market_value.overall_movement_start = self.market_value.overall_movement_end;
        self.market_value.current_price = avg;
        self.market_value.overall_movement_end = self.recent_transactions_strike_prices.last().unwrap().clone();

        self.recent_transactions_strike_prices.clear();        
        return self.market_value.clone();
    }
}

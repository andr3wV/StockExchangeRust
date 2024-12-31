use std::collections::HashMap;

use crate::trade_house::{OfferAsk, Trade, TradeHouse};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Market {
    market_values: HashMap<u64, MarketValueTracker>,
    house: TradeHouse,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MarketValue {
/// Current price of a stock as shown for display purposes
    current_price: f64,
    highest_price: f64,
    lowest_price: f64,
    overall_movement_start: f64,
    overall_movement_end: f64,
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

pub enum ActionState {
    AddedToOffers,
    InstantlyResolved,
}

impl Market {
    pub fn new() -> Self {
        Self {
            house: TradeHouse::new(),
            market_values: HashMap::new(),
        }
    }

    pub fn buy_trade(
        &mut self,
        agent_id: u64,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        trade: Trade,
      ) -> Result<ActionState, Vec<usize>>  {
        let appropriate_trade_offer = self.house.get_appropriate_trade_offer(
            company_id,
            strike_price,
            acceptable_strike_price_deviation,
            OfferAsk::Sell,
        );
        let all_offers = self.house.get_mut_trade_offers(company_id);
        let Some(offer_ids) = appropriate_trade_offer else {
            self.house.add_trade_offer(agent_id, company_id, strike_price, trade, OfferAsk::Buy);
            return Ok(ActionState::AddedToOffers);
        };
        for offer_id in offer_ids.iter() {
            let offer = all_offers.seller_offers[*offer_id].clone();
            if offer.strike_price > strike_price {
                continue;
            }
            if offer.data.number_of_shares == trade.number_of_shares {
                self.house.get_mut_trade_offers(company_id).remove_offer(*offer_id);
                return Ok(ActionState::InstantlyResolved);
            }

            if offer.data.number_of_shares > trade.number_of_shares {
                self.house.get_mut_trade_offers(company_id).remove_offer(*offer_id);
                self.house.add_trade_offer(
                    offer.offerer_id,
                    company_id,
                    offer.strike_price,
                    Trade::new(offer.data.number_of_shares - trade.number_of_shares),
                    OfferAsk::Sell,
                );
            } else {
                self.house.get_mut_trade_offers(company_id).remove_offer(*offer_id);
                self.house.add_trade_offer(
                    agent_id,
                    company_id,
                    strike_price,
                    Trade::new(trade.number_of_shares - offer.data.number_of_shares),
                    OfferAsk::Buy,
                );
            }
            return Ok(ActionState::AddedToOffers);
        }
        return Err(offer_ids);
    }

    pub fn sell_trade(
        &mut self,
        agent_id: u64,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        trade: Trade,
    ) -> Result<ActionState, Vec<usize>> {
        let appropriate_trade_offer = self.house.get_appropriate_trade_offer(
            company_id,
            strike_price,
            acceptable_strike_price_deviation,
            OfferAsk::Buy,
        );
        let all_offers = self.house.get_mut_trade_offers(company_id);
        let Some(offer_ids) = appropriate_trade_offer else {
            self.house.add_trade_offer(agent_id, company_id, strike_price, trade, OfferAsk::Sell);
            return Ok(ActionState::AddedToOffers);
        };
        for offer_id in offer_ids.iter() {
            let offer = all_offers.buyer_offers[*offer_id].clone();
            if offer.strike_price < strike_price {
                continue;
            }
            if offer.data.number_of_shares == trade.number_of_shares {
                self.house.get_mut_trade_offers(company_id).remove_offer(*offer_id);
                return Ok(ActionState::InstantlyResolved);
            }

            if offer.data.number_of_shares > trade.number_of_shares {
                self.house.get_mut_trade_offers(company_id).remove_offer(*offer_id);
                self.house.add_trade_offer(
                    offer.offerer_id,
                    company_id,
                    offer.strike_price,
                    Trade::new(offer.data.number_of_shares - trade.number_of_shares),
                    OfferAsk::Buy,
                );
            } else {
                self.house.get_mut_trade_offers(company_id).remove_offer(*offer_id);
                self.house.add_trade_offer(
                    agent_id,
                    company_id,
                    strike_price,
                    Trade::new(trade.number_of_shares - offer.data.number_of_shares),
                    OfferAsk::Sell,
                );
            }
            return Ok(ActionState::AddedToOffers);
        }
        return Err(offer_ids);
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

fn max(a: f64, b: f64) -> f64 {
    if a > b {
        a
    } else {
        b
    }
}
fn min(a: f64, b: f64) -> f64 {
    if a < b {
        a
    } else {
        b
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
        let max = self.recent_transactions_strike_prices.iter().fold(0.0, |a, &b| max(a, b));
        let min = self.recent_transactions_strike_prices.iter().fold(0.0, |a, &b| min(a, b));
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

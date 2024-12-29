use std::collections::HashMap;

use rand::random;
use serde::{Deserialize, Serialize};

/// Basically stores all the requested trades that weren't immediately resolved
#[derive(Serialize, Deserialize, Debug)]
pub struct TradeHouse {
    stock_trade: HashMap<u64, Offers<Trade>>,
    option_trade: HashMap<u64, Offers<StockOption>>,
}

/// All the trade offers of the certain company
#[derive(Serialize, Deserialize, Debug)]
struct Offers<T> {
    seller_offers: Vec<Offer<T>>,
    buyer_offers: Vec<Offer<T>>,
    lowest_strike_price: f64,
    highest_strike_price: f64,
}

/// A specific trade offer
#[derive(Serialize, Deserialize, Debug)]
struct Offer<T> {
    id: u64,
    offerer_id: u64,
    strike_price: f64,
    data: T,
}

#[derive(Serialize, Deserialize, Debug)]
struct Trade {
    number_of_stocks: u64,
}

/// A specific option offer
/// MAYBE RECONSIDER THE ATTRIBUTES
#[derive(Serialize, Deserialize, Debug)]
struct StockOption {
    number_of_shares: u64,
    time_to_expiry: u64,
}

impl TradeHouse {
    pub fn new() -> Self {
        Self {
            stock_trade: HashMap::new(),
            option_trade: HashMap::new(),
        }
    }
}

impl<T> Offers<T> {
    pub fn new() -> Self {
        Self {
            seller_offers: Vec::new(),
            buyer_offers: Vec::new(),
            highest_strike_price: 0.0,
            lowest_strike_price: 0.0,
        }
    }
    pub fn add_seller_offer(&mut self, trade: Offer<T>) {
        if trade.strike_price > self.highest_strike_price {
            self.highest_strike_price = trade.strike_price;
        }
        if trade.strike_price < self.lowest_strike_price {
            self.lowest_strike_price = trade.strike_price;
        }
        self.seller_offers.push(trade);
    }
    pub fn add_buyer_offer(&mut self, trade: Offer<T>) {
        if trade.strike_price > self.highest_strike_price {
            self.highest_strike_price = trade.strike_price;
        }
        if trade.strike_price < self.lowest_strike_price {
            self.lowest_strike_price = trade.strike_price;
        }
        self.buyer_offers.push(trade);
    }
}

impl<T> Offer<T> {
    pub fn new(offerer_id: u64, strike_price: f64, data: T) -> Self {
        Self {
            id: random(),
            offerer_id,
            strike_price,
            data,
        }
    }
}
use std::collections::HashMap;

use rand::random;
use serde::{Deserialize, Serialize};

/// Basically stores all the requested trades that weren't immediately resolved
#[derive(Serialize, Deserialize, Debug)]
pub struct TradeHouse {
    trade_offers: HashMap<u64, Offers<Trade>>,
    option_offers: HashMap<u64, Offers<StockOption>>,
}

/// All the offers of the certain company
#[derive(Serialize, Deserialize, Debug)]
pub struct Offers<T> {
    seller_offers: Vec<Offer<T>>,
    buyer_offers: Vec<Offer<T>>,
    lowest_strike_price: f64,
    highest_strike_price: f64,
}

/// A specific offer
#[derive(Serialize, Deserialize, Debug)]
pub struct Offer<T> {
    id: u64,
    offerer_id: u64,
    strike_price: f64,
    data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Trade {
    number_of_shares: u64,
}

/// A specific option offer
/// MAYBE RECONSIDER THE ATTRIBUTES
#[derive(Serialize, Deserialize, Debug)]
pub struct StockOption {
    number_of_shares: u64,
    time_to_expiry: u64,
}

impl TradeHouse {
    pub fn new() -> Self {
        Self {
            trade_offers: HashMap::new(),
            option_offers: HashMap::new(),
        }
    }
    pub fn get_mut_trade_offers(&mut self, company_id: u64) -> &mut Offers<Trade> {
        if !self.trade_offers.contains_key(&company_id) {
            self.trade_offers.insert(company_id, Offers::new());
        }
        self.trade_offers.get_mut(&company_id).unwrap()
    }
    pub fn get_mut_option_offers(&mut self, company_id: u64) -> &mut Offers<StockOption> {
        if !self.option_offers.contains_key(&company_id) {
            self.option_offers.insert(company_id, Offers::new());
        }
        self.option_offers.get_mut(&company_id).unwrap()
    }

    pub fn add_buyer_trade_offer(&mut self, offerer_id: u64, company_id: u64, strike_price: f64, trade: Trade) {
        let company_stock_trade = self.get_mut_trade_offers(company_id);
        company_stock_trade.buyer_offers.push(Offer::new(offerer_id, strike_price, trade));
    }
    pub fn add_seller_trade_offer(&mut self, offerer_id: u64, company_id: u64, strike_price: f64, trade: Trade) {
        let company_stock_trade = self.get_mut_trade_offers(company_id);
        company_stock_trade.seller_offers.push(Offer::new(offerer_id, strike_price, trade));
    }
    pub fn add_buyer_option_offer(&mut self, offerer_id: u64, company_id: u64, strike_price: f64, option: StockOption) {
        let company_stock_trade = self.get_mut_option_offers(company_id);
        company_stock_trade.buyer_offers.push(Offer::new(offerer_id, strike_price, option));
    }
    pub fn add_seller_option_offer(&mut self, offerer_id: u64, company_id: u64, strike_price: f64, option: StockOption) {
        let company_stock_trade = self.get_mut_option_offers(company_id);
        company_stock_trade.seller_offers.push(Offer::new(offerer_id, strike_price, option));
    }
    /// Returns the indices of trade_house.trade_offers which matches the strike_price
    pub fn check_for_similar_buyer_trade_offer(&mut self, company_id: u64, strike_price: f64) -> Option<Vec<usize>> {
        if !self.trade_offers.contains_key(&company_id) {
            return None;
        }
        Some(self.trade_offers.get(&company_id).unwrap().buyer_offers.iter().enumerate().filter_map(|(i, item)| {
            if item.strike_price == strike_price {
                return Some(i);
            }
            None
        }).collect())
    }

    /// Returns the indices of trade_house.trade_offers which matches the strike_price
    pub fn check_for_similar_seller_trade_offer(&mut self, company_id: u64, strike_price: f64) -> Option<Vec<usize>> {
        if !self.trade_offers.contains_key(&company_id) {
            return None;
        }
        Some(self.trade_offers.get(&company_id).unwrap().seller_offers.iter().enumerate().filter_map(|(i, item)| {
            if item.strike_price == strike_price {
                return Some(i);
            }
            None
        }).collect())
    }

    /// Returns the indices of trade_house.option_offers which matches the strike_price
    pub fn check_for_similar_buyer_option_offer(&mut self, company_id: u64, strike_price: f64) -> Option<Vec<usize>> {
        if !self.option_offers.contains_key(&company_id) {
            return None;
        }
        Some(self.option_offers.get(&company_id).unwrap().buyer_offers.iter().enumerate().filter_map(|(i, item)| {
            if item.strike_price == strike_price {
                return Some(i);
            }
            None
        }).collect())
    }

    /// Returns the indices of trade_house.option_offers which matches the strike_price
    pub fn check_for_similar_seller_option_offer(&mut self, company_id: u64, strike_price: f64) -> Option<Vec<usize>> {
        if !self.option_offers.contains_key(&company_id) {
            return None;
        }
        Some(self.option_offers.get(&company_id).unwrap().seller_offers.iter().enumerate().filter_map(|(i, item)| {
            if item.strike_price == strike_price {
                return Some(i);
            }
            None
        }).collect())
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
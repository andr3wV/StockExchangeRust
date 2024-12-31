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
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Offers<T> {
    pub seller_offers: Vec<Offer<T>>,
    pub buyer_offers: Vec<Offer<T>>,
    pub lowest_strike_price: f64,
    pub highest_strike_price: f64,
}

/// A specific offer
/// 
///todo Allow users to cancel their offers (to get their holdings back)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Offer<T> {
    pub id: u64,
    pub offerer_id: u64,
    pub strike_price: f64,
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Trade {
    pub number_of_shares: u64,
}
impl Trade {
    pub fn new(number_of_shares: u64) -> Self {
        Self {
            number_of_shares,
        }
    }
}

/// A specific option offer
/// MAYBE RECONSIDER THE ATTRIBUTES
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StockOption {
    pub number_of_shares: u64,
    pub time_to_expiry: u64,
}
impl StockOption {
    pub fn new(number_of_shares: u64, time_to_expiry: u64) -> Self {
        Self {
            number_of_shares,
            time_to_expiry,
        }
    }
}

#[derive(PartialEq)]
pub enum OfferAsk {
    Buy,
    Sell,
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

    pub fn add_trade_offer(&mut self, offerer_id: u64, company_id: u64, strike_price: f64, trade: Trade, offer_ask: OfferAsk) {
        match offer_ask {
            OfferAsk::Buy => self.add_buyer_trade_offer(offerer_id, company_id, strike_price, trade),
            OfferAsk::Sell => self.add_seller_trade_offer(offerer_id, company_id, strike_price, trade),
        }
    }

    pub fn add_option_offer(&mut self, offerer_id: u64, company_id: u64, strike_price: f64, option: StockOption, offer_ask: OfferAsk) {
        match offer_ask {
            OfferAsk::Buy => self.add_buyer_option_offer(offerer_id, company_id, strike_price, option),
            OfferAsk::Sell => self.add_seller_option_offer(offerer_id, company_id, strike_price, option),
        }
    }

    pub fn get_appropriate_trade_offer(&mut self, company_id: u64, strike_price: f64, acceptable_strike_price_deviation: f64, offer_ask: OfferAsk) -> Option<Vec<usize>> {
        match offer_ask {
            OfferAsk::Buy => self.get_appropriate_buyer_trade_offer(company_id, strike_price, acceptable_strike_price_deviation),
            OfferAsk::Sell => self.get_appropriate_seller_trade_offer(company_id, strike_price, acceptable_strike_price_deviation),
        }
    }

    pub fn get_appropriate_option_offer(&mut self, company_id: u64, strike_price: f64, acceptable_strike_price_deviation: f64, offer_ask: OfferAsk) -> Option<Vec<usize>> {
        match offer_ask {
            OfferAsk::Buy => self.get_appropriate_buyer_option_offer(company_id, strike_price, acceptable_strike_price_deviation),
            OfferAsk::Sell => self.get_appropriate_seller_option_offer(company_id, strike_price, acceptable_strike_price_deviation),
        }
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
    pub fn get_appropriate_buyer_trade_offer(&mut self, company_id: u64, strike_price: f64, acceptable_strike_price_deviation: f64,) -> Option<Vec<usize>> {
        Some(self.trade_offers
            .get(&company_id)?
            .buyer_offers
            .iter()
            .enumerate()
            .filter_map(|(i, offer)| {
                if offer.strike_price >= strike_price - acceptable_strike_price_deviation {
                    return Some(i);
                }
                None
            }).collect::<Vec<usize>>())
    }

    /// Returns the indices of trade_house.trade_offers which matches the strike_price
    pub fn get_appropriate_seller_trade_offer(&mut self, company_id: u64, strike_price: f64, acceptable_strike_price_deviation: f64,) -> Option<Vec<usize>> {
        Some(self.trade_offers
            .get(&company_id)?
            .seller_offers
            .iter()
            .enumerate()
            .filter_map(|(i, offer)| {
                if offer.strike_price <= strike_price + acceptable_strike_price_deviation {
                    return Some(i);
                }
                None
            }).collect::<Vec<usize>>())
    }

    /// Returns the indices of trade_house.option_offers which matches the strike_price
    pub fn get_appropriate_buyer_option_offer(&mut self, company_id: u64, strike_price: f64, acceptable_strike_price_deviation: f64,) -> Option<Vec<usize>> {
        Some(self.option_offers
            .get(&company_id)?
            .buyer_offers
            .iter()
            .enumerate()
            .filter_map(|(i, offer)| {
                if offer.strike_price >= strike_price - acceptable_strike_price_deviation {
                    return Some(i);
                }
                None
            }).collect::<Vec<usize>>())
    }

    /// Returns the indices of trade_house.option_offers which matches the strike_price
    pub fn get_appropriate_seller_option_offer(&mut self, company_id: u64, strike_price: f64, acceptable_strike_price_deviation: f64,) -> Option<Vec<usize>> {
        Some(self.option_offers
            .get(&company_id)?
            .seller_offers
            .iter()
            .enumerate()
            .filter_map(|(i, offer)| {
                if offer.strike_price <= strike_price + acceptable_strike_price_deviation {
                    return Some(i);
                }
                None
            }).collect::<Vec<usize>>())
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

    pub fn remove_offer(&mut self, offer_id: usize) {
        self.seller_offers.retain(|offer| offer.id != offer_id as u64);
        self.buyer_offers.retain(|offer| offer.id != offer_id as u64);
    }

    pub fn remove_offer_by_idx(&mut self, index: usize) {
        self.seller_offers.remove(index);
        self.buyer_offers.remove(index);
    }

    pub fn add_offer(&mut self, trade: Offer<T>, offer_ask: OfferAsk) {
        match offer_ask {
            OfferAsk::Buy => self.add_buyer_offer(trade),
            OfferAsk::Sell => self.add_seller_offer(trade),
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
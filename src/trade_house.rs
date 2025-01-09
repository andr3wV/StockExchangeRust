use std::collections::HashMap;

use crate::OFFER_LIFETIME;
use rand::random;
use serde::{Deserialize, Serialize};

/// Basically stores all the requested trades that weren't immediately resolved
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TradeHouse {
    trade_offers: HashMap<u64, Offers<Trade>>,
    option_offers: HashMap<u64, Offers<StockOption>>,
}

/// All the offers of the certain company
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Offers<T>
where
    T: Clone + Default,
{
    pub seller_offers: Vec<Offer<T>>,
    pub buyer_offers: Vec<Offer<T>>,
    pub lowest_strike_price: f64,
    pub highest_strike_price: f64,
}

/// A specific offer
///
///todo Allow users to cancel their offers (to get their holdings back)
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Offer<T>
where
    T: Clone + Default,
{
    pub id: u64,
    pub offerer_id: u64,
    pub strike_price: f64,
    pub data: T,
    pub lifetime: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Trade {
    pub number_of_shares: u64,
}
impl Trade {
    pub fn new(number_of_shares: u64) -> Self {
        Self { number_of_shares }
    }
}

pub struct FailedOffer<T: Clone + Default>(pub Offer<T>, pub TradeAction);

/// A specific option offer
/// MAYBE RECONSIDER THE ATTRIBUTES
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
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

#[derive(PartialEq, Clone, Debug, Serialize, Deserialize, Copy)]
pub enum TradeAction {
    Buy,
    Sell,
}

impl TradeAction {
    pub fn complement(&self) -> Self {
        match self {
            TradeAction::Buy => TradeAction::Sell,
            TradeAction::Sell => TradeAction::Buy,
        }
    }
}

impl TradeHouse {
    pub fn new() -> Self {
        Self {
            trade_offers: HashMap::new(),
            option_offers: HashMap::new(),
        }
    }

    pub fn get_mut_trade_offers(&mut self, company_id: u64) -> &mut Offers<Trade> {
        self.trade_offers.entry(company_id).or_default()
    }

    pub fn get_mut_option_offers(&mut self, company_id: u64) -> &mut Offers<StockOption> {
        self.option_offers.entry(company_id).or_default()
    }

    pub fn add_trade_offer(
        &mut self,
        offerer_id: u64,
        company_id: u64,
        strike_price: f64,
        trade: Trade,
        offer_ask: TradeAction,
    ) {
        let company_stock_trade = self.get_mut_trade_offers(company_id);
        (match offer_ask {
            TradeAction::Buy => &mut company_stock_trade.buyer_offers,
            TradeAction::Sell => &mut company_stock_trade.seller_offers,
        })
        .push(Offer::new(offerer_id, strike_price, trade));
    }

    pub fn add_option_offer(
        &mut self,
        offerer_id: u64,
        company_id: u64,
        strike_price: f64,
        option: StockOption,
        offer_ask: TradeAction,
    ) {
        let company_stock_trade = self.get_mut_option_offers(company_id);
        (match offer_ask {
            TradeAction::Buy => &mut company_stock_trade.buyer_offers,
            TradeAction::Sell => &mut company_stock_trade.seller_offers,
        })
        .push(Offer::new(offerer_id, strike_price, option));
    }

    pub fn get_appropriate_trade_offer(
        &mut self,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        offer_ask: TradeAction,
    ) -> Option<Vec<usize>> {
        match offer_ask {
            TradeAction::Buy => self.get_appropriate_buyer_trade_offer(
                company_id,
                strike_price,
                acceptable_strike_price_deviation,
            ),
            TradeAction::Sell => self.get_appropriate_seller_trade_offer(
                company_id,
                strike_price,
                acceptable_strike_price_deviation,
            ),
        }
    }

    /// Returns the indices of trade_house.trade_offers which matches the strike_price
    pub fn get_appropriate_buyer_trade_offer(
        &mut self,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
    ) -> Option<Vec<usize>> {
        Some(
            self.trade_offers
                .get(&company_id)?
                .buyer_offers
                .iter()
                .enumerate()
                .filter_map(|(i, offer)| {
                    if offer.strike_price >= strike_price - acceptable_strike_price_deviation {
                        return Some(i);
                    }
                    None
                })
                .collect::<Vec<usize>>(),
        )
    }

    /// Returns the indices of trade_house.trade_offers which matches the strike_price
    pub fn get_appropriate_seller_trade_offer(
        &mut self,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
    ) -> Option<Vec<usize>> {
        Some(
            self.trade_offers
                .get(&company_id)?
                .seller_offers
                .iter()
                .enumerate()
                .filter_map(|(i, offer)| {
                    if offer.strike_price <= strike_price + acceptable_strike_price_deviation {
                        return Some(i);
                    }
                    None
                })
                .collect::<Vec<usize>>(),
        )
    }

    pub fn get_appropriate_option_offer(
        &mut self,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        offer_ask: TradeAction,
    ) -> Option<Vec<usize>> {
        match offer_ask {
            TradeAction::Buy => self.get_appropriate_buyer_option_offer(
                company_id,
                strike_price,
                acceptable_strike_price_deviation,
            ),
            TradeAction::Sell => self.get_appropriate_seller_option_offer(
                company_id,
                strike_price,
                acceptable_strike_price_deviation,
            ),
        }
    }

    /// Returns the indices of trade_house.option_offers which matches the strike_price
    pub fn get_appropriate_buyer_option_offer(
        &mut self,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
    ) -> Option<Vec<usize>> {
        Some(
            self.option_offers
                .get(&company_id)?
                .buyer_offers
                .iter()
                .enumerate()
                .filter_map(|(i, offer)| {
                    if offer.strike_price >= strike_price - acceptable_strike_price_deviation {
                        return Some(i);
                    }
                    None
                })
                .collect::<Vec<usize>>(),
        )
    }

    /// Returns the indices of trade_house.option_offers which matches the strike_price
    pub fn get_appropriate_seller_option_offer(
        &mut self,
        company_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
    ) -> Option<Vec<usize>> {
        Some(
            self.option_offers
                .get(&company_id)?
                .seller_offers
                .iter()
                .enumerate()
                .filter_map(|(i, offer)| {
                    if offer.strike_price <= strike_price + acceptable_strike_price_deviation {
                        return Some(i);
                    }
                    None
                })
                .collect::<Vec<usize>>(),
        )
    }

    pub fn tick(
        &mut self,
    ) -> (
        HashMap<u64, Vec<FailedOffer<Trade>>>,
        HashMap<u64, Vec<FailedOffer<StockOption>>>,
    ) {
        let mut trade_offers = HashMap::new();
        let mut option_offers = HashMap::new();
        for (company_id, offers) in self.trade_offers.iter_mut() {
            let expired_trades = offers.tick();
            if !expired_trades.is_empty() {
                trade_offers.insert(*company_id, expired_trades);
            }
        }
        for (company_id, offers) in self.option_offers.iter_mut() {
            let expired_options = offers.tick();
            if !expired_options.is_empty() {
                option_offers.insert(*company_id, expired_options);
            }
        }
        (trade_offers, option_offers)
    }
}

impl<T: Clone + Default> Offers<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn remove_offer(&mut self, offer_id: usize) {
        self.seller_offers
            .retain(|offer| offer.id != offer_id as u64);
        self.buyer_offers
            .retain(|offer| offer.id != offer_id as u64);
    }

    pub fn remove_offer_by_idx(&mut self, index: usize) {
        self.seller_offers.remove(index);
        self.buyer_offers.remove(index);
    }

    pub fn add_offer(&mut self, trade: Offer<T>, offer_ask: TradeAction) {
        match offer_ask {
            TradeAction::Buy => self.add_buyer_offer(trade),
            TradeAction::Sell => self.add_seller_offer(trade),
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

    pub fn tick(&mut self) -> Vec<FailedOffer<T>> {
        let mut expired_offers = Vec::new();
        for i in (0..self.seller_offers.len()).rev() {
            let Some(offer) = self.seller_offers[i].tick() else {
                continue;
            };
            expired_offers.push(FailedOffer(offer, TradeAction::Sell));
            self.seller_offers.remove(i);
        }
        for i in (0..self.buyer_offers.len()).rev() {
            let Some(offer) = self.buyer_offers[i].tick() else {
                continue;
            };
            expired_offers.push(FailedOffer(offer, TradeAction::Buy));
            self.buyer_offers.remove(i);
        }
        expired_offers
    }
}

impl<T: Clone + Default> Offer<T> {
    pub fn new(offerer_id: u64, strike_price: f64, data: T) -> Self {
        Self {
            id: random(),
            offerer_id,
            strike_price,
            data,
            lifetime: OFFER_LIFETIME,
        }
    }
    pub fn tick(&mut self) -> Option<Offer<T>> {
        self.lifetime -= 1;
        if self.lifetime == 0 {
            return Some(self.clone());
        }
        None
    }
}

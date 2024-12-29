use std::collections::HashMap;

use crate::trade_house::TradeHouse;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Market {
    market_values: HashMap<u64, MarketValue>,
    house: TradeHouse,
}

#[derive(Serialize, Deserialize, Debug)]
struct MarketValue {
    /// Current price of a stock as shown for display purposes
    current_price: f64,
    highest_price: f64,
    lowest_price: f64,
    standard_deviation: f64,

    /// If the interval is set for 5 seconds
    /// Have a function, called something like tick().
    /// When it is called, the maximum and minimum values in the vec are stored in
    /// `highest_price` and `lowest_price`. And the average is set at `current_price`
    /// Also calculate the standard deviation and store it in `standard_deviation`
    recent_transactions_strike_prices: Vec<f64>,
}

impl Market {
    pub fn new() -> Self {
        Self {
            house: TradeHouse::new(),
            market_values: HashMap::new(),
        }
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
            standard_deviation: 0.0,
            recent_transactions_strike_prices: Vec::new(),
        }
    }
    pub fn add_transaction(&mut self, price: f64) {
        self.recent_transactions_strike_prices.push(price);
    }
    pub fn tick(&mut self) {
        if self.recent_transactions_strike_prices.is_empty() {
            return;
        }
        let max = self.recent_transactions_strike_prices.iter().fold(0.0, |a, &b| max(a, b));
        let min = self.recent_transactions_strike_prices.iter().fold(0.0, |a, &b| min(a, b));
        let sum: f64 = self.recent_transactions_strike_prices.iter().sum();
        let avg = sum / self.recent_transactions_strike_prices.len() as f64;
        let variance = self.recent_transactions_strike_prices.iter().fold(0.0, |a, &b| a + (b - avg).powi(2));
        let standard_deviation = (variance / self.recent_transactions_strike_prices.len() as f64).sqrt();

        self.highest_price = max;
        self.lowest_price = min;
        self.current_price = avg;
        self.standard_deviation = standard_deviation;

        self.recent_transactions_strike_prices.clear();        
    }
}

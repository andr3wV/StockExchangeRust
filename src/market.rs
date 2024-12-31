use std::collections::HashMap;

use crate::trade_house::TradeHouse;
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

impl Market {
    pub fn new() -> Self {
        Self {
            house: TradeHouse::new(),
            market_values: HashMap::new(),
        }
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
        let avg = sum / self.recent_transactions_strike_prices.len() as f64;

        self.market_value.highest_price = max;
        self.market_value.lowest_price = min;
        self.market_value.overall_movement_start = self.market_value.overall_movement_end;
        self.market_value.current_price = avg;
        self.market_value.overall_movement_end = self.recent_transactions_strike_prices.last().unwrap().clone();

        self.recent_transactions_strike_prices.clear();        
        return self.market_value.clone();
    }
}

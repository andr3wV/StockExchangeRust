use std::collections::HashMap;

use crate::trade_house::House;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Market {
    market_values: HashMap<u64, MarketValue>,
    house: House,
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
            house: House::new(),
            market_values: HashMap::new(),
        }
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
}

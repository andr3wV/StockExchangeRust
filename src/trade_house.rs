use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Basically stores all the requested trades that weren't immediately resolved
#[derive(Serialize, Deserialize, Debug)]
pub struct House {
    stock_trade: HashMap<u64, OpenTrades>,
    option_trade: HashMap<u64, OpenOptions>,
}

/// All the trades of the certain company
#[derive(Serialize, Deserialize, Debug)]
struct OpenTrades {
    trades: Vec<Trade>,
    lowest_strike_price: f64,
    highest_strike_price: f64,
}

/// A specific trade offer
#[derive(Serialize, Deserialize, Debug)]
struct Trade {
    id: u64,
    transactor_id: u64,
    strike_price: f64,
    number_of_shares: u64,
}

/// All the options of the certain company
#[derive(Serialize, Deserialize, Debug)]
struct OpenOptions {
    options: Vec<Option>,
    // todo: figure shit out here
}

/// A specific option offer
#[derive(Serialize, Deserialize, Debug)]
struct Option {
    // todo: figure shit out here
}

impl House {
    pub fn new() -> Self {
        Self {
            stock_trade: HashMap::new(),
            option_trade: HashMap::new(),
        }
    }
}

impl OpenTrades {
    pub fn new() -> Self {
        Self {
            trades: Vec::new(),
            highest_strike_price: 0.0,
            lowest_strike_price: 0.0,
        }
    }
}

impl Trade {}

impl OpenOptions {
    pub fn new() -> Self {
        Self {
            options: Vec::new(),
        }
    }
}

impl Option {}

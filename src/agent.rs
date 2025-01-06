use std::collections::HashMap;

use crate::{market::MarketValue, trade_house::TradeAction, transaction::Holdings};

use rand::{random, Rng};
use serde::{Deserialize, Serialize};

static MAX_INITIAL_BALANCE: f64 = 100000.0;

#[derive(Serialize, Deserialize, Debug)]
pub struct Agent {
    pub id: u64,
    /// How much money does the agent have
    pub balance: f64,
    /// How many shares does an agent hold in a company
    pub holdings: Holdings,
    pub try_transactions: HashMap<u64, f64>,
}

pub static SYMBOL_LENGTH: usize = 4;
pub static MAX_RANDOM_TOTAL_SHARES: u64 = 16000;

#[derive(Serialize, Deserialize, Debug)]
pub struct Company {
    pub id: u64,
    pub name: String,
    pub code: [char; SYMBOL_LENGTH],
    pub market_value: MarketValue,
}

fn random_string() -> String {
    (0..10).map(|_| rand_char()).collect()
}

fn rand_char() -> char {
    let mut rng = rand::thread_rng();
    let mut i: u8 = rng.gen_range(0..52);
    if i < 26 {
        return ('a' as u8 + i) as char;
    }
    i -= 26;
    return ('A' as u8 + i) as char;
}

impl Agent {
    pub fn new(id: u64, balance: f64, holdings: Holdings) -> Self {
        Self {
            id,
            balance,
            holdings,
            try_transactions: HashMap::new(),
        }
    }
    pub fn rand() -> Self {
        Self::new(random(), random::<f64>() * MAX_INITIAL_BALANCE, Holdings::new())
    }
    pub fn can_buy(&self, price: f64, quantity: u64) -> bool {
        self.balance >= price * quantity as f64
    }
    pub fn can_sell(&self, company_id: u64, quantity: u64) -> bool {
        self.holdings.get(&company_id) >= quantity
    }
    pub fn add_failed_transaction(&mut self, company_id: u64, failed_price: f64, offer_type: &TradeAction) {
        let price;
        match offer_type {
            TradeAction::Buy => {
                price = failed_price;
            }
            TradeAction::Sell => {
                price = -failed_price;
            }
        }
        self.try_transactions.insert(company_id, price + failed_price * 0.25);
    }
}

impl Company {
    pub fn new(id: u64, name: String, code: [char; SYMBOL_LENGTH], market_value: MarketValue) -> Self {
        Self {
            id,
            name,
            code,
            market_value,
        }
    }
    pub fn rand() -> Self {
        Self::new(
            random(),
            random_string(),
            (0..SYMBOL_LENGTH).map(|_| rand_char()).collect::<Vec<char>>().try_into().unwrap(),
            MarketValue::rand()
        )
    }
}

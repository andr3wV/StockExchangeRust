use crate::NUM_OF_COMPANIES;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MarketValue {
    /// Current price of a stock as shown for display purposes
    pub current_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub overall_movement_start: f64,
    pub overall_movement_end: f64,
}

pub const SYMBOL_LENGTH: usize = 4;
pub const MAX_RANDOM_TOTAL_SHARES: u64 = 16000;

#[derive(Default)]
pub struct Companies {
    pub num_of_companies: u64,
    pub market_values: Vec<MarketValue>,
}

#[derive(Serialize, Deserialize)]
pub struct Company {
    pub id: u64,
    pub market_value: MarketValue,
}

impl Companies {
    pub fn new() -> Self {
        Self {
            num_of_companies: 0,
            market_values: Vec::new(),
        }
    }
    pub fn rand() -> Self {
        let mut market_values = Vec::with_capacity(NUM_OF_COMPANIES as usize);
        market_values
            .iter_mut()
            .for_each(|x| *x = MarketValue::rand());
        Self {
            num_of_companies: NUM_OF_COMPANIES,
            market_values,
        }
    }
    pub fn load(companies: &[Company]) -> Self {
        let num_of_companies = companies.len() as u64;
        let mut market_values = Vec::with_capacity(num_of_companies as usize);
        for company in companies.iter() {
            market_values.push(company.market_value.clone());
        }
        Self {
            num_of_companies,
            market_values,
        }
    }
    pub fn get_current_price(&self, company_id: u64) -> f64 {
        match self.market_values.get(company_id as usize) {
            Some(market_value) => market_value.current_price,
            None => 0.0,
        }
    }
    pub fn iter(&self) -> std::ops::Range<u64> {
        0..self.num_of_companies
    }
    pub fn rand_company_id(&self) -> u64 {
        rand::random::<u64>() % self.num_of_companies
    }
    pub fn save(&self) -> Vec<Company> {
        let mut companies = Vec::with_capacity(self.num_of_companies as usize);
        for (id, market_value) in self.market_values.iter().enumerate() {
            companies.push(Company {
                id: id as u64,
                market_value: market_value.clone(),
            });
        }
        companies
    }
}

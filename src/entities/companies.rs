use rand::Rng;
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
pub const MAX_NUM_OF_HYPE_COMPANIES: usize = 2;

#[derive(Default)]
pub struct Companies {
    pub num_of_companies: u64,
    pub market_values: Vec<MarketValue>,
    pub balances: Vec<f64>,
    pub hype: [Option<u64>; MAX_NUM_OF_HYPE_COMPANIES],
}

#[derive(Serialize, Deserialize)]
pub struct Company {
    pub id: u64,
    pub market_value: MarketValue,
    pub balance: f64,
}

fn rand_hype(
    rng: &mut impl Rng,
    number_of_companies: usize,
) -> [Option<u64>; MAX_NUM_OF_HYPE_COMPANIES] {
    let mut hype = [None; MAX_NUM_OF_HYPE_COMPANIES];
    for hype_item in hype
        .iter_mut()
        .take(rng.gen_range(0..MAX_NUM_OF_HYPE_COMPANIES))
    {
        *hype_item = Some(rng.gen_range(0..number_of_companies as u64));
    }
    hype
}

impl Companies {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn rand(number_of_companies: usize, rng: &mut impl Rng) -> Self {
        let mut market_values = Vec::with_capacity(number_of_companies);
        let mut balances = Vec::with_capacity(number_of_companies);
        for _ in 0..number_of_companies {
            balances.push(rng.gen_range(10_000.0..1_000_000.0));
            market_values.push(MarketValue::rand(rng));
        }
        Self {
            num_of_companies: number_of_companies as u64,
            market_values,
            balances,
            hype: rand_hype(rng, number_of_companies),
        }
    }
    pub fn load(companies: &[Company]) -> Self {
        let num_of_companies = companies.len() as u64;
        let mut market_values = Vec::with_capacity(num_of_companies as usize);
        let mut balances = Vec::with_capacity(num_of_companies as usize);
        for company in companies.iter() {
            market_values.push(company.market_value.clone());
            balances.push(company.balance);
        }
        Self {
            num_of_companies,
            market_values,
            balances,
            hype: vec![None; MAX_NUM_OF_HYPE_COMPANIES].try_into().unwrap(),
        }
    }
    pub fn load_mut(&mut self, companies: &[Company]) {
        self.num_of_companies += companies.len() as u64;
        for company in companies.iter() {
            self.market_values.push(company.market_value.clone());
            self.balances.push(company.balance);
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
    pub fn rand_company_id(&self, rng: &mut impl Rng) -> u64 {
        rng.gen_range(0..self.num_of_companies)
    }
    pub fn save(&self) -> Vec<Company> {
        let mut companies = Vec::with_capacity(self.num_of_companies as usize);
        for id in 0..(self.num_of_companies as usize) {
            companies.push(Company {
                id: id as u64,
                market_value: self.market_values[id].clone(),
                balance: self.balances[id],
            });
        }
        companies
    }
}

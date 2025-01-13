use rand::Rng;
use rand_distr::{Distribution, Normal};
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
pub const MIN_PROFIT_PERCENT_FOR_POSITIVE_HYPE_CONSIDERATION: f64 = 70.0;
pub const MAX_PROFIT_PERCENT_FOR_NEGATIVE_HYPE_CONSIDERATION: f64 = -30.0;

#[derive(Default)]
pub struct Companies {
    pub num_of_companies: u64,
    pub market_values: Vec<MarketValue>,
    pub balances: Vec<f64>,
    pub selling_shares_prices: Vec<f64>,
    pub selling_shares_counts: Vec<u64>,
    pub expected_profits: Vec<f64>,
    pub news: Vec<f64>,
    pub hype: [Option<(u64, f64)>; MAX_NUM_OF_HYPE_COMPANIES],
}

#[derive(Serialize, Deserialize)]
pub struct Company {
    pub id: u64,
    pub market_value: MarketValue,
    pub balance: f64,
    pub selling_shares_count: u64,
    pub selling_shares_price: f64,
    pub expected_profit: f64,
    pub news: f64,
}

fn rand_hype(
    rng: &mut impl Rng,
    number_of_companies: usize,
) -> [Option<(u64, f64)>; MAX_NUM_OF_HYPE_COMPANIES] {
    let mut hype = [None; MAX_NUM_OF_HYPE_COMPANIES];
    for hype_item in hype
        .iter_mut()
        .take(rng.gen_range(0..MAX_NUM_OF_HYPE_COMPANIES))
    {
        *hype_item = Some((
            rng.gen_range(0..number_of_companies as u64),
            rng.gen_range(-100.0..100.0),
        ));
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
        let mut selling_shares_prices = Vec::with_capacity(number_of_companies);
        let mut selling_shares_counts = Vec::with_capacity(number_of_companies);
        let mut expected_profits = Vec::with_capacity(number_of_companies);
        let mut news = Vec::with_capacity(number_of_companies);
        for _ in 0..number_of_companies {
            balances.push(rng.gen_range(10_000.0..1_000_000.0));
            market_values.push(MarketValue::rand(rng));
            selling_shares_prices.push(0.0);
            selling_shares_counts.push(0);

            let expected_profit = rng.gen_range(100.0..10_000.0);
            expected_profits.push(expected_profit);
            let Ok(normal) = Normal::new(0.0, 100.0 / expected_profit) else {
                // If the normal distribution fails, fuck it then
                news.push(0.0);
                continue;
            };
            news.push(normal.sample(rng) * 100.0);
        }
        Self {
            num_of_companies: number_of_companies as u64,
            market_values,
            balances,
            hype: rand_hype(rng, number_of_companies),
            selling_shares_prices,
            selling_shares_counts,
            expected_profits,
            news,
        }
    }
    pub fn load(companies: &[Company]) -> Self {
        let num_of_companies = companies.len();
        let mut market_values = Vec::with_capacity(num_of_companies);
        let mut balances = Vec::with_capacity(num_of_companies);
        let mut selling_shares_prices = Vec::with_capacity(num_of_companies);
        let mut selling_shares_counts = Vec::with_capacity(num_of_companies);
        let mut expected_profits = Vec::with_capacity(num_of_companies);
        let mut news = Vec::with_capacity(num_of_companies);
        for company in companies.iter() {
            market_values.push(company.market_value.clone());
            balances.push(company.balance);
            selling_shares_prices.push(company.selling_shares_price);
            selling_shares_counts.push(company.selling_shares_count);
            expected_profits.push(company.expected_profit);
            news.push(company.news);
        }
        Self {
            num_of_companies: num_of_companies as u64,
            market_values,
            balances,
            hype: vec![None; MAX_NUM_OF_HYPE_COMPANIES].try_into().unwrap(),
            selling_shares_prices,
            selling_shares_counts,
            expected_profits,
            news,
        }
    }
    pub fn load_mut(&mut self, companies: &[Company]) {
        self.num_of_companies += companies.len() as u64;
        for company in companies.iter() {
            self.market_values.push(company.market_value.clone());
            self.balances.push(company.balance);
            self.selling_shares_prices
                .push(company.selling_shares_price);
            self.selling_shares_counts
                .push(company.selling_shares_count);
            self.expected_profits.push(company.expected_profit);
            self.news.push(company.news);
        }
    }
    pub fn save(&self) -> Vec<Company> {
        let mut companies = Vec::with_capacity(self.num_of_companies as usize);
        for id in 0..(self.num_of_companies as usize) {
            companies.push(Company {
                id: id as u64,
                market_value: self.market_values[id].clone(),
                balance: self.balances[id],
                selling_shares_price: self.selling_shares_prices[id],
                selling_shares_count: self.selling_shares_counts[id],
                expected_profit: self.expected_profits[id],
                news: self.news[id],
            });
        }
        companies
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
    pub fn rand_release_news(&mut self, rng: &mut impl Rng) {
        let mut hypeable_companies = Vec::new();
        for id in 0..self.num_of_companies {
            let expected_profit = self.expected_profits[id as usize];
            let Ok(normal) = Normal::new(0.0, 100.0 / expected_profit) else {
                // If the normal distribution fails, we just add the expected profit
                self.balances[id as usize] += expected_profit;
                continue;
            };
            let deviation: f64 = normal.sample(rng);
            let Some(hypeable_news) = self.release_news(id, deviation) else {
                continue;
            };
            hypeable_companies.push((id, hypeable_news));
        }
        self.send_hype(&mut hypeable_companies);
    }
    pub fn release_news(&mut self, company_id: u64, deviation: f64) -> Option<f64> {
        let id = company_id as usize;
        let balance = &mut self.balances[id];
        let news = deviation * 100.0;
        *balance += self.expected_profits[id] * deviation;
        self.news[id] = news;
        if (MAX_PROFIT_PERCENT_FOR_NEGATIVE_HYPE_CONSIDERATION
            ..=MIN_PROFIT_PERCENT_FOR_POSITIVE_HYPE_CONSIDERATION)
            .contains(&news)
        {
            return None;
        }
        Some(news)
    }
    pub fn send_hype(&mut self, hypeable_companies: &mut Vec<(u64, f64)>) {
        let mut item = hypeable_companies.pop();
        for hype in self.hype.iter_mut() {
            let Some(clean_item) = item else {
                return;
            };
            let Some(hype) = hype else {
                *hype = Some(clean_item);
                item = hypeable_companies.pop();
                continue;
            };
            if clean_item.1.abs() < hype.1.abs() {
                continue;
            }
            *hype = clean_item;
            item = hypeable_companies.pop();
        }
    }
}

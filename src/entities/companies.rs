use super::agents::Agents;
use crate::SimulationError;
use rand::Rng;
use rand_distr::{Distribution, Normal};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MarketValue {
    /// Current price of a stock as shown for display purposes
    pub current_price: f64,
    pub highest_price: f64,
    pub lowest_price: f64,
    pub overall_movement_start: f64,
    pub overall_movement_end: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Lots {
    pub strike_price: f64,
    pub number_of_shares: u64,
    pub bets: HashMap<u64, u64>,
}

pub const SYMBOL_LENGTH: usize = 4;
pub const MAX_NUM_OF_HYPE_COMPANIES: usize = 2;
pub const MIN_PROFIT_PERCENT_FOR_POSITIVE_HYPE_CONSIDERATION: f64 = 70.0;
pub const MAX_PROFIT_PERCENT_FOR_NEGATIVE_HYPE_CONSIDERATION: f64 = -30.0;

#[derive(Default)]
pub struct Companies {
    pub num_of_companies: u64,
    pub market_values: Vec<MarketValue>,
    pub balances: Vec<f64>,
    pub expected_profits: Vec<f64>,
    pub news: Vec<f64>,
    pub hype: [Option<(u64, f64)>; MAX_NUM_OF_HYPE_COMPANIES],
    pub lots: Vec<Lots>,
}

#[derive(Serialize, Deserialize)]
pub struct Company {
    pub id: u64,
    pub market_value: MarketValue,
    pub balance: f64,
    pub expected_profit: f64,
    pub news: f64,
    pub lots: Lots,
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

impl Lots {
    pub fn new(strike_price: f64, number_of_shares: u64) -> Self {
        Self {
            strike_price,
            number_of_shares,
            bets: HashMap::new(),
        }
    }
    pub fn rand(rng: &mut impl Rng) -> Self {
        Self {
            strike_price: rng.gen_range(10.0..1_000.0),
            number_of_shares: rng.gen_range(1..1_000_000),
            bets: HashMap::new(), // no random bets because agents might not have the money for the bet
                                  // or be uninterested
        }
    }
    pub fn add_bet(
        &mut self,
        agents: &mut Agents,
        agent_id: u64,
        number_of_shares: u64,
    ) -> Result<(), SimulationError> {
        agents
            .balances
            .add(agent_id, self.strike_price * number_of_shares as f64)?;
        self.bets
            .entry(agent_id)
            .and_modify(|bet| *bet += number_of_shares)
            .or_insert(number_of_shares);
        Ok(())
    }
    pub fn remove_bet(
        &mut self,
        agent_id: u64,
        number_of_shares: u64,
    ) -> Result<(), SimulationError> {
        let Some(bet) = self.bets.get_mut(&agent_id) else {
            return Err(SimulationError::AgentNotFound(agent_id));
        };
        if *bet < number_of_shares {
            return Err(SimulationError::Unspendable);
        }
        *bet -= number_of_shares;
        if *bet == 0 {
            self.bets.remove(&agent_id);
        }
        Ok(())
    }
    pub fn distribute_shares(&mut self, company_id: u64, agents: &mut Agents) {
        let mut bets = self.bets.iter().collect::<Vec<_>>();
        bets.sort_by(|a, b| b.1.cmp(a.1));
        for (&agent_id, &number_of_shares) in bets.iter() {
            if self.number_of_shares < number_of_shares {
                continue;
            }
            agents
                .holdings
                .insert(agent_id, company_id, number_of_shares);
            self.number_of_shares -= number_of_shares;
        }
        self.bets.clear();
    }
}

impl Companies {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn rand(number_of_companies: usize, rng: &mut impl Rng) -> Self {
        let mut market_values = Vec::with_capacity(number_of_companies);
        let mut balances = Vec::with_capacity(number_of_companies);
        let mut expected_profits = Vec::with_capacity(number_of_companies);
        let mut news = Vec::with_capacity(number_of_companies);
        let mut lots = Vec::with_capacity(number_of_companies);
        for _ in 0..number_of_companies {
            balances.push(rng.gen_range(10_000.0..1_000_000.0));
            market_values.push(MarketValue::rand(rng));
            lots.push(Lots::rand(rng));

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
            expected_profits,
            news,
            lots,
        }
    }
    pub fn load(companies: &[Company]) -> Self {
        let num_of_companies = companies.len();
        let mut market_values = Vec::with_capacity(num_of_companies);
        let mut balances = Vec::with_capacity(num_of_companies);
        let mut expected_profits = Vec::with_capacity(num_of_companies);
        let mut news = Vec::with_capacity(num_of_companies);
        let mut lots = Vec::with_capacity(num_of_companies);
        for company in companies.iter() {
            market_values.push(company.market_value.clone());
            balances.push(company.balance);
            expected_profits.push(company.expected_profit);
            news.push(company.news);
            lots.push(company.lots.clone());
        }
        Self {
            num_of_companies: num_of_companies as u64,
            market_values,
            balances,
            hype: vec![None; MAX_NUM_OF_HYPE_COMPANIES].try_into().unwrap(),
            expected_profits,
            news,
            lots,
        }
    }
    pub fn load_mut(&mut self, companies: &[Company]) {
        self.num_of_companies += companies.len() as u64;
        for company in companies.iter() {
            self.market_values.push(company.market_value.clone());
            self.balances.push(company.balance);
            self.expected_profits.push(company.expected_profit);
            self.news.push(company.news);
            self.lots.push(company.lots.clone());
        }
    }
    pub fn save(&self) -> Vec<Company> {
        let mut companies = Vec::with_capacity(self.num_of_companies as usize);
        for id in 0..(self.num_of_companies as usize) {
            companies.push(Company {
                id: id as u64,
                market_value: self.market_values[id].clone(),
                balance: self.balances[id],
                expected_profit: self.expected_profits[id],
                news: self.news[id],
                lots: self.lots[id].clone(),
            });
        }
        companies
    }
    pub fn get_current_price(&self, company_id: u64) -> f64 {
        self.market_values
            .get(company_id as usize)
            .map(|market_value| market_value.current_price)
            .unwrap_or(0.0)
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

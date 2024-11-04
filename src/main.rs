// # Industry
// An industry is a current status of success OR how important it is at a given time
//
// # Company
// A company is within a particular industry.
// A company might be within multiple industries but then they will need a weighted distribution on
// how much of them is in a particular one.
// Companies have quaterly earnings report.
//
// # Confidence
// There is this term "Confidence" thrown around when talking about a particular company's state
// The current stock price is nothing more than the price at which the last transaction took place.
// For many stocks, transactions are occurring every second the stock market is open.
//
// This seem pretty compilcated, I think I should build a simple copy first and then iterate

// # Simple Copy
// A company has a stock price, it goes up when someone buys a stock, it goes down when someone sells
// a stock.
// An agent can buy and sell a particular stock

use rand::{prelude::*, thread_rng, rngs::ThreadRng};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use stocks::stats;

struct Company {
    name: String,
    code: [char; 3],
    id: usize,
    stock_price: f32,
    number_of_stocks: u32,
}

impl Company {
    pub fn new(name: String, code: [char; 3], stock_price: f32, id: usize) -> Self {
        Self {
            name,
            code,
            id,
            stock_price,
            number_of_stocks: 0,
        }
    }
}

impl Debug for Company {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!(
            "{}({})[${}|{}]",
            self.name,
            code_str(self.code),
            self.stock_price,
            self.number_of_stocks
        ))
    }
}

#[derive(Clone)]
struct Agent {
    id: usize,
    money: f32,
}

impl Agent {
    pub fn new(id: usize, money: f32) -> Self {
        Self { id, money }
    }
}

impl Debug for Agent {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("Agent({})[${}]", self.id, self.money))
    }
}

fn get_id(company_id: usize, agent_id: usize) -> u64 {
    (((company_id as u64) << 32) | (agent_id as u64)).into()
}

fn update_stock(
    stocks: &mut HashMap<u64, u32>,
    company_id: usize,
    agent_id: usize,
    new_value: u32,
) {
    stocks.insert(get_id(company_id, agent_id), new_value);
}
fn deviate_stock(
    stocks: &mut HashMap<u64, u32>,
    company_id: usize,
    agent_id: usize,
    deviation: i64,
) {
    if deviation == 0 {
        return;
    }
    if deviation < 0 {
        stocks.insert(
            get_id(company_id, agent_id),
            get_stock(stocks, company_id, agent_id) - (-deviation) as u32,
        );
        return;
    }
    stocks.insert(
        get_id(company_id, agent_id),
        get_stock(stocks, company_id, agent_id) + deviation as u32,
    );
}
fn get_stock(stocks: &HashMap<u64, u32>, company_id: usize, agent_id: usize) -> u32 {
    stocks
        .get(&get_id(company_id, agent_id))
        .unwrap_or(&0)
        .clone()
}

fn get_code(s: &str) -> [char; 3] {
    let mut chars = s.chars();
    [
        chars.next().unwrap_or('\0'),
        chars.next().unwrap_or('\0'),
        chars.next().unwrap_or('\0'),
    ]
}
fn code_str(code: [char; 3]) -> String {
    let mut s = String::with_capacity(3);
    s += &code[0].to_string();
    s += &code[1].to_string();
    s += &code[2].to_string();
    s
}

#[derive(Debug)]
enum StockBuyingError {
    NotEnoughMoneyToBuy,
}

#[derive(Debug)]
enum StockSellingError {
    TryingToSellMoreStocksThanOwned,
}

struct StockMarket {
    holdings: HashMap<u64, u32>,
    companies: Vec<Company>,
    agents: Vec<Agent>,
}

impl StockMarket {
    pub fn new() -> Self {
        Self {
            holdings: HashMap::new(),
            companies: Vec::new(),
            agents: Vec::new(),
        }
    }

    pub fn add_company(
        &mut self,
        name: String,
        code: [char; 3],
        initial_public_offering: f32,
    ) -> usize {
        let id = self.companies.len();
        self.companies
            .push(Company::new(name, code, initial_public_offering, id));
        id
    }

    pub fn add_agent(&mut self) -> usize {
        let id = self.agents.len();
        self.agents.push(Agent::new(id, 10000.0));
        id
    }

    pub fn stock_price(&self, company_id: usize) -> f32 {
        self.companies[company_id].stock_price
    }
    pub fn deviate_stock_price(&mut self, company_id: usize, deviation: f32) {
        if deviation == 0.0 {
            return;
        }
        self.companies[company_id].stock_price += deviation;
    }
    pub fn deviate_number_of_stocks(&mut self, company_id: usize, deviation: i64) {
        if deviation == 0 {
            return;
        }
        if deviation < 0 {
            self.companies[company_id].number_of_stocks -= (-deviation) as u32;
            return;
        }
        self.companies[company_id].number_of_stocks += deviation as u32;
    }

    pub fn buy_stock(
        &mut self,
        company_id: usize,
        agent_id: usize,
        number_of_stocks: u32,
    ) -> Result<(), StockBuyingError> {
        // transacting the agent
        {
            let remaining_money = self.agents[agent_id].money
                - number_of_stocks as f32 * self.companies[company_id].stock_price;
            if remaining_money < 0.0 {
                return Err(StockBuyingError::NotEnoughMoneyToBuy);
            }
            self.agents[agent_id].money = remaining_money;
            deviate_stock(
                &mut self.holdings,
                company_id,
                agent_id,
                number_of_stocks as i64,
            );
        }
        // updating the company stock details
        {
            self.deviate_stock_price(company_id, 0.1 * (number_of_stocks as f32));
            self.deviate_number_of_stocks(company_id, number_of_stocks as i64);
        }
        Ok(())
    }
    pub fn sell_stock(
        &mut self,
        company_id: usize,
        agent_id: usize,
        number_of_stocks: u32,
    ) -> Result<(), StockSellingError> {
        // transacting the agent
        {
            let currently_owned_stocks = get_stock(&self.holdings, company_id, agent_id);
            if currently_owned_stocks < number_of_stocks {
                return Err(StockSellingError::TryingToSellMoreStocksThanOwned);
            }
            self.agents[agent_id].money +=
                number_of_stocks as f32 * self.stock_price(company_id);
            deviate_stock(
                &mut self.holdings,
                company_id,
                agent_id,
                -(number_of_stocks as i64),
            );
        }
        // updating the company stock details
        {
            self.deviate_stock_price(company_id, -0.1 * (number_of_stocks as f32));
            self.deviate_number_of_stocks(company_id, -(number_of_stocks as i64));
        }
        Ok(())
    }
}

struct PreferenceDistribution {
    buying_preference: Vec<f32>,
    selling_preference: Vec<f32>,
}
impl PreferenceDistribution {
    pub fn new() -> Self {
        Self {
            buying_preference: Vec::new(),
            selling_preference: Vec::new(),
        }
    }
    pub fn add_company(&mut self) {
        self.buying_preference.push(1.0);
        self.selling_preference.push(1.0);
    }
    pub fn buying_mod_sum_distribution(&self) -> f32 {
        self.buying_preference
            .iter()
            .map(|&x| if x > 0.0 { x } else { -x })
            .sum()
    }
    pub fn selling_mod_sum_distribution(&self) -> f32 {
        self.selling_preference
            .iter()
            .map(|&x| if x > 0.0 { x } else { -x })
            .sum()
    }
    pub fn buying_probability(&self, company_id: usize) -> f32 {
        self.buying_preference[company_id] / self.buying_mod_sum_distribution()
    }
    pub fn selling_probability(&self, company_id: usize) -> f32 {
        self.selling_preference[company_id] / self.selling_mod_sum_distribution()
    }
}

struct Simulation {
    market: StockMarket,
    preference_distribution: PreferenceDistribution,
}

impl Simulation {
    pub fn new() -> Self {
        Self {
            market: StockMarket::new(),
            preference_distribution: PreferenceDistribution::new(),
        }
    }

    pub fn add_companies(&mut self, companies_data: Vec<(String, String, f32)>) {
        for company_data in companies_data.iter() {
            _ = self.market.add_company(
                company_data.0.clone(),
                get_code(company_data.1.as_str()),
                company_data.2,
            );
            self.preference_distribution.add_company();
        }
    }

    pub fn spawn_agents(&mut self, number_of_agents: u32) {
        for _ in 0..number_of_agents {
            _ = self.market.add_agent();
        }
    }

    /*
     * All agents buy a random stock from the market
     */
    pub fn buy_random(&mut self, rng: &mut ThreadRng) {
        for agent in self.market.agents.clone().iter() {
            let action = rng.gen_range(0..=self.market.companies.len());
            if action == self.market.companies.len() {
                // don't buy
                continue;
            }
            let company_id = self.market.companies[action].id.clone();
            _ = self
                .market
                .buy_stock(company_id, agent.id, rng.gen_range(0..=5));
        }
    }

    /*
     * Force a probabilistic amount of agents to buy a specific stock from the market
     */
    pub fn buy_preferred(&mut self, rng: &mut ThreadRng, preferred_company_id: usize, preference_probability: f32,) {
        let preferred_company_id = self.market.companies[preferred_company_id].id.clone();
        for agent in self.market.agents.clone().iter() {
            let action = rng.gen_range(0.0..1.0);
            if action >= preference_probability {
                // don't buy
                continue;
            }
            _ = self
                .market
                .buy_stock(preferred_company_id, agent.id, rng.gen_range(0..=5));
        }
    }

    /*
     * All agents sell a random stock from the market
     */
    pub fn sell_random(&mut self, rng: &mut ThreadRng) {
        for agent in self.market.agents.clone().iter() {
            let action = rng.gen_range(0..=self.market.companies.len());
            if action == self.market.companies.len() {
                // don't sell
                continue;
            }
            let company_id = self.market.companies[action].id.clone();
            _ = self
                .market
                .sell_stock(company_id, agent.id, rng.gen_range(0..=5));
        }
    }

    /*
     * Force a probabilistic amount of agents to sell a specific stock from the market
     */
    pub fn sell_preferred(&mut self, rng: &mut ThreadRng, preferred_company_id: usize, preference_probability: f32,) {
        let preferred_company_id = self.market.companies[preferred_company_id].id.clone();
        for agent in self.market.agents.clone().iter() {
            let action = rng.gen_range(0.0..1.0);
            if action >= preference_probability {
                // don't sell
                continue;
            }
            _ = self
                .market
                .sell_stock(preferred_company_id, agent.id, rng.gen_range(0..=5));
        }
    }

    /*
     * All agents either buy or sell a random stock from the market
     */
    pub fn trade_random(&mut self) {
        let mut rng = thread_rng();
        for i in 0..self.market.companies.len() {
            let buying_probability = self.preference_distribution.buying_probability(i);
            let selling_probability = self.preference_distribution.selling_probability(i);
            self.buy_preferred(&mut rng, i, buying_probability);
            self.sell_preferred(&mut rng, i, selling_probability);
        }
    }
}

fn main() {
    let mut simulation = Simulation::new();
    simulation.add_companies(vec![
        ("Intel".to_string(), "INT".to_string(), 12.23),
        ("Nvidia".to_string(), "NVD".to_string(), 99.15),
        ("Google".to_string(), "GGL".to_string(), 54.63),
    ]);
    simulation.preference_distribution.selling_preference[0] = 5.0;
    simulation.preference_distribution.buying_preference[1] = 5.0;

    simulation.spawn_agents(1000);

    let mut i = 0;
    loop {
        if i == 10 {
            break;
        }
        if i % 1 == 0 {
            println!("{:?}", simulation.market.companies);
            let agent_money: Vec<f32> = simulation
                .market
                .agents
                .iter()
                .map(|agent| agent.money)
                .collect();
            println!(
                "{:?} +/- {:?}",
                stats::mean(agent_money.iter()),
                stats::standard_deviation(agent_money.iter())
            );
        }

        simulation.trade_random();
        i += 1;
    }
}

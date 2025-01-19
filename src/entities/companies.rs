use crate::{
    log, logger::{Log},
    entities::agents::Agents, trade_house::TradeAction, transaction::TodoTransactions,
    SimulationError,
};
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
    pub number_of_lots: u64,
    pub lot_size: u64,
    pub bets: HashMap<u64, u64>,
    pub total_num_of_bets: u64,
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
    pub lot_finalization_times: Vec<u64>,
}

#[derive(Serialize, Deserialize)]
pub struct Company {
    pub id: u64,
    pub market_value: MarketValue,
    pub balance: f64,
    pub expected_profit: f64,
    pub news: f64,
    pub lots: Lots,
    pub lot_finalization_time: u64,
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

fn news_to_probability(news: f64) -> f64 {
    1.0 - (-news * news).exp()
}

impl Lots {
    pub fn new(strike_price: f64, number_of_lots: u64, lot_size: u64) -> Self {
        Self {
            strike_price,
            number_of_lots,
            lot_size,
            bets: HashMap::new(),
            total_num_of_bets: 0,
        }
    }
    pub fn is_blank(&self) -> bool {
        self.strike_price == 0.0 || self.lot_size == 0 || self.number_of_lots == 0
    }
    pub fn close(&mut self) {
        self.strike_price = 0.0;
        self.number_of_lots = 0;
        self.lot_size = 0;
        self.total_num_of_bets = 0;
        self.bets.clear();
    }
    pub fn rand(rng: &mut impl Rng) -> Self {
        Self {
            strike_price: rng.gen_range(10.0..1_000.0),
            number_of_lots: rng.gen_range(1..1_000) * 100, // keep it a multiple of 100,
            lot_size: rng.gen_range(1..10) * 10,           // keep it a multiple of 10
            total_num_of_bets: 0,
            bets: HashMap::new(), // no random bets because agents might not have the money for the bet
                                  // or be uninterested
        }
    }
    pub fn rng_reset(&mut self, rng: &mut impl Rng, appox_price: f64) {
        self.bets.clear();
        self.strike_price = appox_price + rng.gen_range(-1_000.0..1_000.0);
        self.number_of_lots = rng.gen_range(1..1_000) * 100;
        self.lot_size = rng.gen_range(1..10) * 10;
        self.total_num_of_bets = 0;
    }
    pub fn rng_reset_exact_price(&mut self, rng: &mut impl Rng, exact_price: f64) {
        self.bets.clear();
        self.strike_price = exact_price;
        self.number_of_lots = rng.gen_range(1..1_000) * 100;
        self.lot_size = rng.gen_range(1..10) * 10;
        self.total_num_of_bets = 0;
    }
    pub fn add_bet_and_update_agent(
        &mut self,
        agents: &mut Agents,
        agent_id: u64,
        number_of_lots: u64,
    ) -> Result<(), SimulationError> {
        if self.is_blank() {
            return Ok(());
        }
        if self.lot_size == 0 {
            return Ok(());
        }
        agents.balances.add(
            agent_id,
            -(self.strike_price * (self.lot_size * number_of_lots) as f64),
        )?;
        self.bets
            .entry(agent_id)
            .and_modify(|bet| *bet += number_of_lots)
            .or_insert(number_of_lots);
        self.total_num_of_bets += number_of_lots;
        Ok(())
    }
    pub fn add_bet(
        &mut self,
        agent_id: u64,
        number_of_lots: u64,
    ) {
        if self.is_blank() {
            return;
        }
        if self.lot_size == 0 {
            return;
        }
        self.bets
            .entry(agent_id)
            .and_modify(|bet| *bet += number_of_lots)
            .or_insert(number_of_lots);
        self.total_num_of_bets += number_of_lots;
    }
    pub fn fits_agent_price(&self, strike_price: f64, acceptable_deviation: f64) -> bool {
        (self.strike_price - strike_price).abs() < acceptable_deviation
    }
    pub fn remove_bet_and_update_agent(
        &mut self,
        agents: &mut Agents,
        agent_id: u64,
        number_of_lots: u64,
    ) -> Result<(), SimulationError> {
        if self.is_blank() {
            return Ok(());
        }
        let Some(bet) = self.bets.get_mut(&agent_id) else {
            return Err(SimulationError::AgentNotFound(agent_id));
        };
        if *bet < number_of_lots {
            return Err(SimulationError::Unspendable);
        }
        agents.balances.add(
            agent_id,
            self.strike_price * (self.lot_size * number_of_lots) as f64,
        )?;

        *bet -= number_of_lots;
        if *bet == 0 {
            self.bets.remove(&agent_id);
        }
        self.total_num_of_bets -= number_of_lots;
        Ok(())
    }
    pub fn remove_bet(
        &mut self,
        bet: &mut u64,
        agent_id: u64,
        number_of_lots: u64,
    ) {
        if self.is_blank() {
            return;
        }
        *bet -= number_of_lots;
        if *bet == 0 {
            self.bets.remove(&agent_id);
        }
        self.total_num_of_bets -= number_of_lots;
    }
    pub fn get_bet(&self, agent_id: u64) -> u64 {
        self.bets.get(&agent_id).unwrap_or(&0).clone()
    }
    pub fn distribute_shares(&mut self, company_id: u64, agents: &mut Agents) {
        if self.is_blank() {
            return;
        }
        let mut bets = self.bets.iter().collect::<Vec<_>>();
        bets.sort_by(|a, b| b.1.cmp(a.1));
        log!(info "Lot distribution: company_id: {} strike_price: {}", company_id,  self.strike_price);
        for (&agent_id, &number_of_lots) in bets.iter() {
            print!("({}[{}]), ", agent_id, number_of_lots * self.lot_size);
            if self.number_of_lots < number_of_lots {
                continue;
            }
            agents
                .holdings
                .insert(agent_id, company_id, number_of_lots * self.lot_size);
            self.number_of_lots -= number_of_lots;
        }
        println!();
        self.bets.clear();
    }
    pub fn compress_lot_size(&mut self, compress_ratio: f64) -> u64 {
        let new_lot_size = (self.lot_size as f64 * compress_ratio).round() as u64;

        let refund_difference = self.lot_size - new_lot_size;
        self.lot_size = new_lot_size;
        refund_difference
    }
    pub fn compress_shares(&mut self, agents: &mut Agents) -> Result<(), SimulationError> {
        if self.is_blank() {
            return Ok(());
        }
        let compress_ratio = self.total_num_of_bets as f64 / self.number_of_lots as f64;
        if compress_ratio.fract() != 0.0 {
            return Err(SimulationError::UnDoable);
        }
        let refund_difference = self.compress_lot_size(compress_ratio);
        for (bettor, &number_of_lots) in self.bets.iter() {
            agents.balances.add(
                *bettor,
                (refund_difference * number_of_lots) as f64 * self.strike_price,
            )?;
        }
        self.strike_price *= compress_ratio;
        Ok(())
    }
    pub fn finalize(&mut self, company_id: u64, agents: &mut Agents) {
        _ = self.compress_shares(agents); // compress if you can
        self.distribute_shares(company_id, agents);
    }
}

impl Companies {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn rand(number_of_companies: usize, current_time: u64, rng: &mut impl Rng) -> Self {
        let mut market_values = Vec::with_capacity(number_of_companies);
        let mut balances = Vec::with_capacity(number_of_companies);
        let mut expected_profits = Vec::with_capacity(number_of_companies);
        let mut news = Vec::with_capacity(number_of_companies);
        let mut lots = Vec::with_capacity(number_of_companies);
        let mut lot_finalization_times = Vec::with_capacity(number_of_companies);
        for _ in 0..number_of_companies {
            balances.push(rng.gen_range(10_000.0..1_000_000.0));
            market_values.push(MarketValue::rand(rng));
            lots.push(Lots::rand(rng));
            lot_finalization_times.push(current_time + rng.gen_range(5..10));

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
            lot_finalization_times,
        }
    }
    pub fn load(companies: &[Company]) -> Self {
        let num_of_companies = companies.len();
        let mut market_values = Vec::with_capacity(num_of_companies);
        let mut balances = Vec::with_capacity(num_of_companies);
        let mut expected_profits = Vec::with_capacity(num_of_companies);
        let mut news = Vec::with_capacity(num_of_companies);
        let mut lots = Vec::with_capacity(num_of_companies);
        let mut lot_finalization_times = Vec::with_capacity(num_of_companies);
        for company in companies.iter() {
            market_values.push(company.market_value.clone());
            balances.push(company.balance);
            expected_profits.push(company.expected_profit);
            news.push(company.news);
            lots.push(company.lots.clone());
            lot_finalization_times.push(company.lot_finalization_time);
        }
        Self {
            num_of_companies: num_of_companies as u64,
            market_values,
            balances,
            hype: vec![None; MAX_NUM_OF_HYPE_COMPANIES].try_into().unwrap(),
            expected_profits,
            news,
            lots,
            lot_finalization_times,
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
            self.lot_finalization_times
                .push(company.lot_finalization_time);
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
                lot_finalization_time: self.lot_finalization_times[id],
            });
        }
        companies
    }
    pub fn get_current_price(&self, company_id: u64) -> Option<f64> {
        self.market_values
            .get(company_id as usize)
            .map(|market_value| market_value.current_price)
    }
    pub fn iter(&self) -> std::ops::Range<u64> {
        0..self.num_of_companies
    }
    pub fn rand_company_id(&self, rng: &mut impl Rng) -> u64 {
        rng.gen_range(0..self.num_of_companies)
    }
    pub fn rand_release_news(&mut self, agents: &mut Agents, rng: &mut impl Rng) {
        let mut hypeable_companies = Vec::new();
        for id in 0..self.num_of_companies {
            // for now, we distribute shares after news update
            self.lots[id as usize].finalize(id, agents);
            if rng.gen_ratio(1, 10) {
                // 10% chance of re-releasing shares
                let failable_value = rng.gen_range(10.0..2_000.0);
                let current_price = self.get_current_price(id).unwrap_or(failable_value);
                self.lots[id as usize].rng_reset_exact_price(rng, current_price);
            }

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
    pub fn generate_preferences_from_news(&self, rng: &mut impl Rng) -> Vec<(u64, TradeAction)> {
        let mut output = Vec::with_capacity(1000);
        while output.len() != 1000 {
            for (company_id, &news) in self.news.iter().enumerate() {
                let probability = news_to_probability(news);
                if !rng.gen_bool(probability) {
                    continue;
                }
                let action = if news > 0.0 {
                    TradeAction::Buy
                } else {
                    TradeAction::Sell
                };
                output.push((company_id as u64, action));
                break;
            }
        }
        output
    }
    pub fn release_shares(&mut self, company_id: u64, number_of_lots: u64, strike_price: f64) {
        let lots = &mut self.lots[company_id as usize];
        lots.number_of_lots = number_of_lots;
        lots.strike_price = strike_price;
    }
    pub fn check_lot(&self, company_id: u64) -> bool {
        // Ya, this is the way it happens in real life, idk why
        self.lots[company_id as usize].number_of_lots != 0
    }
    pub fn check_lots_from_todotransaction(
        &self,
        todo_transaction: &TodoTransactions,
    ) -> bool {
        self.check_lot(todo_transaction.company_id)
    }
    pub fn add_bet_from_todotransaction(&mut self, todo_transaction: &TodoTransactions) {
        let lot = &mut self.lots[todo_transaction.company_id as usize];
        if lot.lot_size == 0 {
            return;
        }
        lot.add_bet(
            todo_transaction.agent_id,
            (todo_transaction.trade.number_of_shares as f64 / lot.lot_size as f64).round() as u64
        );
    }
}

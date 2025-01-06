use std::collections::HashMap;

use crate::{market::{ActionState, Market, MarketValue}, trade_house::{FailedOffer, StockOption, Trade, TradeAction}, transaction::{Holdings, TodoTransactions, Transaction}, NUM_OF_AGENTS};

use rand::{random, Rng};
use serde::{Deserialize, Serialize};

static MAX_INITIAL_BALANCE: f64 = 100000.0;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
    pub id: u64,
    /// How much money does the agent have
    pub balance: f64,
    /// How many shares does an agent hold in a company
    pub holdings: Holdings,
    pub try_transactions: HashMap<u64, f64>,
}

pub const SYMBOL_LENGTH: usize = 4;
pub const MAX_RANDOM_TOTAL_SHARES: u64 = 16000;

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
        Self::new(
            random(),
            random::<f64>() * MAX_INITIAL_BALANCE,
            Holdings::new(),
        )
    }
    pub fn can_buy(&self, price: f64, quantity: u64) -> bool {
        self.balance >= price * quantity as f64
    }
    pub fn can_sell(&self, company_id: u64, quantity: u64) -> bool {
        self.holdings.get(&company_id) >= quantity
    }
    pub fn add_failed_transaction(
        &mut self,
        company_id: u64,
        failed_price: f64,
        offer_type: &TradeAction,
    ) {
        let price;
        match offer_type {
            TradeAction::Buy => {
                price = failed_price;
            }
            TradeAction::Sell => {
                price = -failed_price;
            }
        }
        self.try_transactions
            .insert(company_id, price + failed_price * 0.25);
    }
}

impl Company {
    pub fn new(
        id: u64,
        name: String,
        code: [char; SYMBOL_LENGTH],
        market_value: MarketValue,
    ) -> Self {
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
            (0..SYMBOL_LENGTH)
                .map(|_| rand_char())
                .collect::<Vec<char>>()
                .try_into()
                .unwrap(),
            MarketValue::rand(),
        )
    }
}

pub struct Agents {
    pub agents: Vec<Agent>,
    pub agents_search: HashMap<u64, usize>,
    pub agent_ids: Vec<u64>
}
impl Agents {
    pub fn new(data: Vec<Agent>) -> Self {
        let agents_search = data.iter().enumerate().map(|(i, x)| (x.id, i)).collect();
        let agent_ids = data.iter().map(|x| x.id).collect();
        Self {
            agents: data,
            agents_search,
            agent_ids,
        }
    }
    pub fn get_agent(&self, id: u64) -> Option<&Agent> {
        self.agents_search.get(&id).map(|&i| &self.agents[i])
    }
    pub fn get_mut_agent(&mut self, id: u64) -> Option<&mut Agent> {
        let idx = self.agents_search.get(&id);
        match idx {
            Some(&i) => Some(&mut self.agents[i]),
            None => None,
        }
    }
    pub fn alert_agents(
        &mut self,
        expired_trades: &HashMap<u64, Vec<FailedOffer<Trade>>>,
        expired_options: &HashMap<u64, Vec<FailedOffer<StockOption>>>,
    ) {
        for (company_id, offers) in expired_trades.iter() {
            for offer in offers.iter() {
                let Some(agent) = self.get_mut_agent(offer.0.offerer_id) else {
                    continue;
                };
                agent.balance += offer.0.strike_price * (offer.0.data.number_of_shares as f64);
                agent.add_failed_transaction(*company_id, offer.0.strike_price, &offer.1);
            }
        }
        for (company_id, offers) in expired_options.iter() {
            for offer in offers {
                let Some(agent) = self.get_mut_agent(offer.0.offerer_id) else {
                    continue;
                };
                agent.balance += offer.0.strike_price * (offer.0.data.number_of_shares as f64);
                agent.add_failed_transaction(*company_id, offer.0.strike_price, &offer.1);
            }
        }
    }
    pub fn give_random_shares_to_half_agents(&mut self, company_ids: &Vec<u64>) {
        for i in 0..(NUM_OF_AGENTS / 2) {
            let agent = &mut self.agents[i as usize];
            let random_company = company_ids[random::<usize>() % company_ids.len()];
            agent
                .holdings
                .insert(random_company, random::<u64>() % 1000);
        }
    }
    pub fn id_iter(&self) -> std::slice::Iter<'_, u64> {
        self.agent_ids.iter()
    }
    pub fn run_through_failed_transactions(&self, transactions: &mut Vec<TodoTransactions>, agent_id: u64, attempting_trade: &Trade) {
        let Some(agent) = self.get_agent(agent_id) else {
            return;
        };
        if agent.try_transactions.is_empty() {
            return;
        }
        for (company_id, new_price) in agent.try_transactions.iter() {
            // 40% chance of retrying
            if random::<f64>() > 0.4 {
                continue;
            }
            let (action, price) = if *new_price > 0.0 {
                (TradeAction::Buy, *new_price)
            } else {
                (TradeAction::Sell, -*new_price)
            };
            let can_transact = match action {
                TradeAction::Buy => agent.can_buy(price, attempting_trade.number_of_shares),
                TradeAction::Sell => agent.can_sell(*company_id, attempting_trade.number_of_shares),
            };
            if !can_transact {
                continue;
            }
            transactions.push(TodoTransactions {
                agent_id,
                company_id: *company_id,
                strike_price: price,
                action,
                trade: attempting_trade.clone(),
            });
        }
    }
    pub fn do_transactions(&mut self, market: &mut Market, transactions: &mut Vec<TodoTransactions>) {
        for todo_transaction in transactions.iter() {
            self.trade(
                market,
                todo_transaction.company_id,
                todo_transaction.agent_id,
                todo_transaction.strike_price,
                5.0,
                &todo_transaction.trade,
                todo_transaction.action.clone(),
            );
        }
    }
    pub fn clear_failed_transactions(&mut self) {
        for agent in self.agents.iter_mut() {
            agent.try_transactions.clear();
        }
    }
    pub fn trade(
        &mut self,
        market: &mut Market,
        company_id: u64,
        agent_id: u64,
        strike_price: f64,
        acceptable_strike_price_deviation: f64,
        trade: &Trade,
        action: TradeAction,
    ) {
        let result = market.trade(
            agent_id,
            company_id,
            strike_price,
            acceptable_strike_price_deviation,
            trade,
            action.clone(),
        );
        let Some(agent) = self.get_mut_agent(agent_id) else {
            return;
        };
        agent.holdings.push(company_id, trade.number_of_shares);
        match result {
            Ok(action_state) => handle_action_state(action_state, market, &mut self.agents, company_id),
            Err(offer_idxs) => handle_offer_idxs(
                offer_idxs,
                market,
                company_id,
                agent_id,
                strike_price,
                trade,
                action,
            ),
        }
    }
}

fn exchange_currency_from_transaction(agents: &mut Vec<Agent>, transaction: &Transaction) {
    let mut buyer: Option<&mut Agent> = None;
    let mut seller: Option<&mut Agent> = None;
    for agent in agents.iter_mut() {
        if agent.id == transaction.buyer_id {
            buyer = Some(agent);
            continue;
        }
        if agent.id == transaction.seller_id {
            seller = Some(agent);
            continue;
        }
    }

    if buyer.is_none() || seller.is_none() {
        return;
    }
    let buyer = buyer.unwrap();
    let seller = seller.unwrap();
    // above this line is a way to getting multiple mutable references to the same vector

    // holdings were updated when the user put up the offers
    buyer.balance -= transaction.strike_price * (transaction.number_of_shares as f64);
    seller.balance += transaction.strike_price * (transaction.number_of_shares as f64);
}

fn handle_action_state(
    action_state: ActionState,
    market: &mut Market,
    agents: &mut Vec<Agent>,
    company_id: u64,
) {
    match action_state {
        ActionState::AddedToOffers => {}
        ActionState::InstantlyResolved(transaction) => {
            market.add_transaction(company_id, transaction.strike_price);
            exchange_currency_from_transaction(agents, &transaction);
        }
        ActionState::PartiallyResolved(transaction) => {
            market.add_transaction(company_id, transaction.strike_price);
            exchange_currency_from_transaction(agents, &transaction);
        }
    }
}

fn handle_offer_idxs(
    offer_idxs: Vec<usize>,
    market: &mut Market,
    company_id: u64,
    agent_id: u64,
    strike_price: f64,
    trade: &Trade,
    action: TradeAction,
) {
    // 30% chance of accept this offer
    if random::<f64>() > 0.3 {
        return;
    }

    let target_offers = match action {
        TradeAction::Buy => &market.house.get_mut_trade_offers(company_id).seller_offers,
        TradeAction::Sell => &market.house.get_mut_trade_offers(company_id).buyer_offers,
    };

    // choose a random offer
    let offer_idx = offer_idxs[random::<usize>() % offer_idxs.len()];
    let offer = target_offers[offer_idx].clone();

    let (transaction, extra_shares_left) =
        market.trade_offer(company_id, &offer, agent_id, trade, action.clone());
    if extra_shares_left > 0 {
        market.house.add_trade_offer(
            agent_id,
            company_id,
            strike_price,
            Trade::new(extra_shares_left),
            action,
        );
    }
    market.add_transaction(company_id, transaction.strike_price);
}

pub struct Companies {
    pub companies: Vec<Company>,
    pub company_ids: Vec<u64>,
}

impl Companies {
    pub fn new(data: Vec<Company>) -> Self {
        let company_ids = data.iter().map(|x| x.id).collect();
        Self {
            companies: data,
            company_ids
        }
    }
    pub fn load_market_values(&self, market_values: &mut HashMap<u64, MarketValue>) {
        for company in self.companies.iter() {
            market_values.insert(company.id, company.market_value.clone());
        }
    }
    pub fn rand_company_id(&self) -> u64 {
        self.company_ids[rand::random::<usize>() % self.companies.len()]
    }
    pub fn dump_market_values(&mut self, market_values: &mut HashMap<u64, MarketValue>) {
        for company in self.companies.iter_mut() {
            company.market_value = market_values
                .get(&company.id)
                .map(|value| value.clone())
                .unwrap_or(MarketValue::new());
        }
    }
}
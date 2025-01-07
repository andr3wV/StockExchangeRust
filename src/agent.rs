use crate::{
    market::{ActionState, Market, MarketValue},
    trade_house::{FailedOffer, StockOption, Trade, TradeAction},
    transaction::{AgentHoldings, TodoTransactions, Transaction},
    NUM_OF_AGENTS, NUM_OF_COMPANIES,
};
use rand::random;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct Holdings(HashMap<u128, u64>);

#[derive(Debug, Clone, Default)]
pub struct Balances(Vec<f64>);

pub const SYMBOL_LENGTH: usize = 4;
pub const MAX_RANDOM_TOTAL_SHARES: u64 = 16000;

#[derive(Default)]
pub struct Agents {
    pub num_of_agents: u64,
    pub holdings: Holdings,
    pub balances: Balances,
    pub try_offers: HashMap<u128, f64>,
}

#[derive(Default)]
pub struct Companies {
    pub num_of_companies: u64,
}

#[derive(Serialize, Deserialize)]
pub struct Agent {
    pub id: u64,
    pub balance: f64,
    pub holding: AgentHoldings,
}

#[derive(Serialize, Deserialize)]
pub struct Company {
    pub id: u64,
}

fn combine(a: u64, b: u64) -> u128 {
    (a as u128) << 64 | b as u128
}

fn get_first(a: u128) -> u64 {
    (a >> 64) as u64
}

fn get_second(a: u128) -> u64 {
    (a & 0xFFFFFFFFFFFFFFFF) as u64
}

impl Holdings {
    pub fn insert(&mut self, agent_id: u64, company_id: u64, number_of_shares: u64) {
        self.0
            .insert(combine(agent_id, company_id), number_of_shares);
    }
    pub fn get(&self, agent_id: u64, company_id: u64) -> u64 {
        match self.0.get(&combine(agent_id, company_id)) {
            Some(share_count) => *share_count,
            None => 0,
        }
    }
    pub fn get_u128(&self, id: u128) -> u64 {
        match self.0.get(&id) {
            Some(share_count) => *share_count,
            None => 0,
        }
    }
    pub fn push_from_txn(&mut self, target_agent_id: u64, transaction: &Transaction) {
        let company = self
            .0
            .get_mut(&combine(target_agent_id, transaction.company_id));
        match company {
            Some(share_count) => {
                *share_count += transaction.number_of_shares;
            }
            None => {
                self.0.insert(
                    combine(target_agent_id, transaction.company_id),
                    transaction.number_of_shares,
                );
            }
        }
    }
    pub fn pop_from_txn(&mut self, target_agent_id: u64, transaction: &Transaction) -> bool {
        let Some(share_count) = self
            .0
            .get_mut(&combine(target_agent_id, transaction.company_id))
        else {
            return false;
        };
        if *share_count < transaction.number_of_shares {
            return false;
        }
        *share_count -= transaction.number_of_shares;
        true
    }
    pub fn push(&mut self, agent_id: u64, company_id: u64, number_of_shares: u64) {
        let Some(share_count) = self.0.get_mut(&combine(agent_id, company_id)) else {
            self.0
                .insert(combine(agent_id, company_id), number_of_shares);
            return;
        };
        *share_count += number_of_shares;
    }
    pub fn pop(&mut self, agent_id: u64, company_id: u64, number_of_shares: u64) -> bool {
        let Some(share_count) = self.0.get_mut(&combine(agent_id, company_id)) else {
            return false;
        };
        if *share_count < number_of_shares {
            return false;
        }

        *share_count -= number_of_shares;
        true
    }
}

impl Balances {
    pub fn get(&self, agent_id: u64) -> f64 {
        self.0[agent_id as usize]
    }
    pub fn add(&mut self, agent_id: u64, amount: f64) {
        self.0[agent_id as usize] += amount;
    }
}

impl Agents {
    pub fn new() -> Self {
        Self {
            num_of_agents: NUM_OF_AGENTS,
            balances: Balances(vec![0.0; NUM_OF_AGENTS as usize]),
            ..Default::default()
        }
    }
    pub fn load(agents: &[Agent]) -> Self {
        let num_of_agents = agents.len() as u64;
        let mut balances = Vec::with_capacity(agents.len());
        let mut holdings = Holdings::default();
        for (i, agent) in agents.iter().enumerate() {
            balances[i] = agent.balance;
            for (company_id, holding) in agent.holding.0.iter() {
                holdings.insert(i as u64, *company_id, *holding);
            }
        }
        Self {
            num_of_agents,
            balances: Balances(balances),
            holdings,
            try_offers: HashMap::new(),
        }
    }
    pub fn save(&self) -> Vec<Agent> {
        let mut agents = Vec::with_capacity(self.num_of_agents as usize);
        for i in 0..self.num_of_agents {
            agents.push(Agent {
                id: i,
                balance: self.balances.get(i),
                holding: AgentHoldings(
                    self.holdings
                        .0
                        .iter()
                        .filter(|(key, _)| get_first(**key) == i)
                        .map(|(key, value)| (get_second(*key), *value))
                        .collect(),
                ),
            });
        }
        agents
    }
    pub fn introduce_new_agents(&mut self, num_of_agents_to_introduce: u64) {
        let mut introduce_ids: Vec<f64> = (self.num_of_agents
            ..(self.num_of_agents + num_of_agents_to_introduce))
            .map(|_| 0.0)
            .collect();
        self.balances.0.append(&mut introduce_ids);
        self.num_of_agents += num_of_agents_to_introduce;
    }
    pub fn can_buy(&self, agent_id: u64, price: f64, quantity: u64) -> bool {
        self.balances.get(agent_id) >= price * quantity as f64
    }
    pub fn can_sell(&self, id: u128, quantity: u64) -> bool {
        self.holdings.get_u128(id) >= quantity
    }
    pub fn iter(&self) -> std::ops::Range<u64> {
        0..self.num_of_agents
    }
    pub fn try_failed_offers(
        &self,
        transactions: &mut Vec<TodoTransactions>,
        attempting_trade: &Trade,
    ) {
        if self.try_offers.is_empty() {
            return;
        }
        for (id, new_price) in self.try_offers.iter() {
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
                TradeAction::Buy => {
                    self.can_buy(get_first(*id), price, attempting_trade.number_of_shares)
                }
                TradeAction::Sell => self.can_sell(*id, attempting_trade.number_of_shares),
            };
            if !can_transact {
                continue;
            }
            transactions.push(TodoTransactions {
                agent_id: get_first(*id),
                company_id: get_second(*id),
                strike_price: price,
                action,
                trade: attempting_trade.clone(),
            });
        }
    }
    pub fn alert_agents(
        &mut self,
        expired_trades: &HashMap<u64, Vec<FailedOffer<Trade>>>,
        expired_options: &HashMap<u64, Vec<FailedOffer<StockOption>>>,
    ) {
        for (company_id, offers) in expired_trades.iter() {
            for offer in offers.iter() {
                // refund
                if offer.1 == TradeAction::Sell {
                    self.holdings.push(
                        offer.0.lifetime,
                        *company_id,
                        offer.0.data.number_of_shares,
                    );
                } else {
                    self.balances.add(
                        offer.0.offerer_id,
                        offer.0.strike_price * (offer.0.data.number_of_shares as f64),
                    );
                }

                self.add_failed_offer(
                    *company_id,
                    offer.0.offerer_id,
                    offer.0.strike_price,
                    &offer.1,
                );
            }
        }
        for (company_id, offers) in expired_options.iter() {
            for offer in offers {
                // refund
                if offer.1 == TradeAction::Sell {
                    self.holdings.push(
                        offer.0.lifetime,
                        *company_id,
                        offer.0.data.number_of_shares,
                    );
                } else {
                    self.balances.add(
                        offer.0.offerer_id,
                        offer.0.strike_price * (offer.0.data.number_of_shares as f64),
                    )
                }

                self.add_failed_offer(
                    *company_id,
                    offer.0.offerer_id,
                    offer.0.strike_price,
                    &offer.1,
                );
            }
        }
    }
    pub fn add_failed_offer(
        &mut self,
        company_id: u64,
        agent_id: u64,
        failed_price: f64,
        offer_type: &TradeAction,
    ) {
        let price = match offer_type {
            TradeAction::Buy => failed_price,
            TradeAction::Sell => -failed_price,
        };
        self.try_offers
            .insert(combine(agent_id, company_id), price + failed_price * 0.25);
    }
    pub fn give_random_shares_to_half_agents(&mut self, company_ids: &[u64]) {
        for i in 0..(NUM_OF_AGENTS / 2) {
            let random_company = company_ids[random::<usize>() % company_ids.len()];
            self.holdings
                .push(i, random_company, random::<u64>() % 1000);
        }
    }
    pub fn do_transactions(&mut self, market: &mut Market, transactions: &mut [TodoTransactions]) {
        for todo_transaction in transactions.iter() {
            self.trade(
                market,
                todo_transaction.company_id,
                todo_transaction.agent_id,
                (todo_transaction.strike_price, 5.0),
                &todo_transaction.trade,
                todo_transaction.action.clone(),
            );
        }
    }
    pub fn roll_action(
        &self,
        agent_id: u64,
        company_id: u64,
        strike_price: f64,
        trade: &Trade,
    ) -> Option<TradeAction> {
        if random::<f64>() > 0.5 {
            if !self.can_buy(agent_id, strike_price, trade.number_of_shares) {
                return None;
            }
            return Some(TradeAction::Buy);
        }
        if !self.can_sell(combine(agent_id, company_id), trade.number_of_shares) {
            return None;
        }
        Some(TradeAction::Sell)
    }
    pub fn trade(
        &mut self,
        market: &mut Market,
        company_id: u64,
        agent_id: u64,
        (strike_price, acceptable_strike_price_deviation): (f64, f64),
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
        if action == TradeAction::Sell {
            if !self
                .holdings
                .pop(agent_id, company_id, trade.number_of_shares)
            {
                return;
            }
        } else {
            if self.balances.get(agent_id) < strike_price * (trade.number_of_shares as f64) {
                return;
            }
            self.balances
                .add(agent_id, -(strike_price * (trade.number_of_shares as f64)));
        }

        match result {
            Ok(action_state) => self.handle_action_state(action_state, market, company_id),
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
    pub fn exchange_currency_from_transaction(&mut self, transaction: &Transaction) {
        // seller's holdings and buyer's money are resolved at the time of offering
        self.holdings
            .pop_from_txn(transaction.buyer_id, transaction);
        self.balances.add(
            transaction.seller_id,
            transaction.strike_price * (transaction.number_of_shares as f64),
        );
    }
    pub fn handle_action_state(
        &mut self,
        action_state: ActionState,
        market: &mut Market,
        company_id: u64,
    ) {
        match action_state {
            ActionState::AddedToOffers => {}
            ActionState::InstantlyResolved(transaction) => {
                market.add_transaction(company_id, transaction.strike_price);
                self.exchange_currency_from_transaction(&transaction);
            }
            ActionState::PartiallyResolved(transaction) => {
                market.add_transaction(company_id, transaction.strike_price);
                self.exchange_currency_from_transaction(&transaction);
            }
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

impl Companies {
    pub fn new() -> Self {
        Self {
            num_of_companies: NUM_OF_COMPANIES,
        }
    }
    pub fn load_market_values(&self, _market_values: &mut HashMap<u64, MarketValue>) {
        // for company in self.companies.iter() {
        //     market_values.insert(company.id, company.market_value.clone());
        // }
    }
    pub fn rand_company_id(&self) -> u64 {
        rand::random::<u64>() % self.num_of_companies
    }
    pub fn dump_market_values(&mut self, _market_values: &mut HashMap<u64, MarketValue>) {
        // for company in self.companies.iter_mut() {
        //     company.market_value = market_values
        //         .get(&company.id)
        //         .map(|value| value.clone())
        //         .unwrap_or(MarketValue::new());
        // }
    }
}

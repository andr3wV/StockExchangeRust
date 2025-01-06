use rand::random;
use stocks::{
    agent::{Agent, Company, SYMBOL_LENGTH}, load, log, logger::Log, market::{ActionState, Market, MarketValue}, max, save, trade_house::{FailedOffer, Offer, OfferAsk, StockOption, Trade}, transaction::Transaction, AGENTS_DATA_FILENAME, COMPANIES_DATA_FILENAME, MIN_STRIKE_PRICE, NUM_OF_AGENTS, NUM_OF_COMPANIES
};
use std::{collections::HashMap, io::BufRead};

fn rand_agents() -> Vec<Agent> {
    (0..NUM_OF_AGENTS).map(|_| Agent::rand()).collect()
}
fn rand_companies() -> Vec<Company> {
    (0..NUM_OF_COMPANIES).map(|_| Company::rand()).collect()
}
fn convert_to_symbol(s: String) -> [char; SYMBOL_LENGTH] {
    let mut symbol = [' '; SYMBOL_LENGTH];
    for (i, c) in s.chars().enumerate() {
        if i < SYMBOL_LENGTH {
            symbol[i] = c;
        }
    }
    symbol
}
fn load_from_file(filename: &str) -> Result<Vec<Company>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(filename)?;

    // read file line by line
    let reader = std::io::BufReader::new(file);
    let companies: Vec<Company> = reader
        .lines()
        .enumerate()
        .map(|(i, line)| {
            let line = line.unwrap();
            let mut iter = line.split('|');
            let name = iter.next().unwrap().to_string();
            let symbol = iter.next().unwrap().to_string();
            Company::new(i as u64, name, convert_to_symbol(symbol), MarketValue::rand())
        })
        .collect();
    
    Ok(companies)
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

fn handle_action_state(action_state: ActionState, market: &mut Market, agents: &mut Vec<Agent>, company_id: u64) {
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

fn handle_offer_idxs(offer_idxs: Vec<usize>, market: &mut Market, company_id: u64, agent_id: u64, strike_price: f64, trade: &Trade, offer_ask: OfferAsk) {
    // 30% chance of accept this offer
    if random::<f64>() > 0.3 {
        return;
    }

    // choose a random offer
    let offer_idx = offer_idxs[random::<usize>() % offer_idxs.len()];
    let offer = market.house.get_mut_trade_offers(company_id).buyer_offers[offer_idx].clone();

    let (transaction, extra_shares_left) = if offer_ask == OfferAsk::Buy {
        market.buy_trade_offer(company_id, &offer, agent_id, trade)
    } else {
        market.sell_trade_offer(company_id, &offer, agent_id, trade)
    };
    if extra_shares_left > 0 {
        market.house.add_trade_offer(
            agent_id,
            company_id,
            strike_price,
            Trade::new(extra_shares_left),
            offer_ask,
        );
    }
    market.add_transaction(company_id, transaction.strike_price);
}

fn buy_random(
    market: &mut Market,
    agents: &mut Vec<Agent>,
    company_id: u64,
    agent_id: u64,
    strike_price: f64,
    acceptable_strike_price_deviation: f64,
    trade: &Trade,
) {
    let result = market.buy_trade(agent_id, company_id, strike_price, acceptable_strike_price_deviation, trade);
    let agent = get_mut_agent(agents, agent_id).unwrap();
    agent.holdings.push(company_id, trade.number_of_shares);
    match result {
        Ok(action_state) => handle_action_state(action_state, market, agents, company_id),
        Err(offer_idxs) => handle_offer_idxs(offer_idxs, market, company_id, agent_id, strike_price, trade, OfferAsk::Buy),
    }
}

fn sell_random(
    market: &mut Market,
    agents: &mut Vec<Agent>,
    company_id: u64,
    agent_id: u64,
    strike_price: f64,
    acceptable_strike_price_deviation: f64,
    trade: &Trade,
) {
    let result = market.sell_trade(agent_id, company_id, strike_price, acceptable_strike_price_deviation, &trade);
    let agent = get_mut_agent(agents, agent_id).unwrap();
    agent.holdings.pop(company_id, trade.number_of_shares);
    match result {
        Ok(action_state) => handle_action_state(action_state, market, agents, company_id),
        Err(offer_idxs) => handle_offer_idxs(offer_idxs, market, company_id, agent_id, strike_price, trade, OfferAsk::Sell),
    }
}

fn get_agent(agents: &Vec<Agent>, agent_id: u64) -> Option<&Agent> {
    for agent in agents.iter() {
        if agent.id == agent_id {
            return Some(agent);
        }
    }
    None
}

fn get_mut_agent(agents: &mut Vec<Agent>, agent_id: u64) -> Option<&mut Agent> {
    for agent in agents.iter_mut() {
        if agent.id == agent_id {
            return Some(agent);
        }
    }
    None
}

fn give_random_shares_to_random_agents(agents: &mut Vec<Agent>, companies: &Vec<Company>) {
    // actually, give half agents some shares to random companies
    for i in 0..(NUM_OF_AGENTS/2) {
        let agent = &mut agents[i as usize];
        let random_company = &companies[random::<usize>() % companies.len()];
        agent.holdings.holdings.insert(random_company.id, random::<u64>() % 1000);
    }
}

fn get_market_value_current_price(market_values: &HashMap<u64, MarketValue>, company_id: &u64) -> f64 {
    match market_values.get(company_id) {
        Some(value) => value.current_price,
        None => 0.0,
    }
}

fn alert_agents(agents: &mut Vec<Agent>, expired_trades: &HashMap<u64, Vec<FailedOffer<Trade>>>, expired_options: &HashMap<u64, Vec<FailedOffer<StockOption>>>) {
    for (company_id, offers) in expired_trades.iter() {
        for offer in offers.iter() {
            let Some(agent) = get_mut_agent(agents, offer.0.offerer_id) else {
                continue;
            };
            agent.balance += offer.0.strike_price * (offer.0.data.number_of_shares as f64);
            agent.add_failed_transaction(*company_id, offer.0.strike_price, &offer.1);
        }
    }
    for (company_id, offers) in expired_options.iter() {
        for offer in offers {
            let Some(agent) = get_mut_agent(agents, offer.0.offerer_id) else {
                continue;
            };
            agent.balance += offer.0.strike_price * (offer.0.data.number_of_shares as f64);
            agent.add_failed_transaction(*company_id, offer.0.strike_price, &offer.1);
        }
    }
}

struct TodoTransactions {
    pub agent_id: u64,
    pub company_id: u64,
    pub strike_price: f64,
    pub action: OfferAsk,
    pub trade: Trade,
}

fn previously_failed_transactions(
    transactions: &mut Vec<TodoTransactions>,
    agents: &Vec<Agent>,
    agent_id: u64,
    trade: &Trade,
) {
    let agent = get_agent(agents, agent_id).unwrap();
    if agent.try_transactions.is_empty() {
        return;
    }
    for (company_id, new_price) in agent.try_transactions.iter() {
        // 40% chance of retrying
        if random::<f64>() > 0.4 {
            continue;
        }
        let action: OfferAsk;
        let price: f64;
        if *new_price > 0.0 {
            action = OfferAsk::Buy;
            price = *new_price;
        } else {
            action = OfferAsk::Sell;
            price = -*new_price;
        }
        match action {
            OfferAsk::Buy => {
                if !agent.can_buy(price, trade.number_of_shares) {
                    continue;
                }
                transactions.push(TodoTransactions {
                    agent_id,
                    company_id: *company_id,
                    strike_price: price,
                    action,
                    trade: trade.clone(),
                });
            }
            OfferAsk::Sell => {
                if !agent.can_sell(*company_id, trade.number_of_shares) {
                    continue;
                }
                transactions.push(TodoTransactions {
                    agent_id,
                    company_id: *company_id,
                    strike_price: price,
                    action,
                    trade: trade.clone(),
                });
            }
        }
    }
}

fn run_todo_transactions(todo_transactions: &mut Vec<TodoTransactions>, market: &mut Market, agents: &mut Vec<Agent>) {
    for todo_transaction in todo_transactions.iter() {
        match todo_transaction.action {
            OfferAsk::Buy => {
                buy_random(
                    market,
                    agents,
                    todo_transaction.company_id,
                    todo_transaction.agent_id,
                    todo_transaction.strike_price,
                    5.0,
                    &todo_transaction.trade,
                );
            }
            OfferAsk::Sell => {
                sell_random(
                    market,
                    agents,
                    todo_transaction.company_id,
                    todo_transaction.agent_id,
                    todo_transaction.strike_price,
                    5.0,
                    &todo_transaction.trade,
                );
            }
        }
    }
}

fn main() {
    let agent_file = load(AGENTS_DATA_FILENAME);
    let company_file = load(COMPANIES_DATA_FILENAME);

    let mut flag_give_random_stocks_to_random_agents = false;

    if agent_file.is_ok() {
        log!(info "Loaded agents");
    } else {
        log!(warn "Agents file not found");
        flag_give_random_stocks_to_random_agents = true;
    }
    if company_file.is_ok() {
        log!(info "Loaded companies");
    } else {
        log!(warn "Company file not found");
    }

    let mut agents: Vec<Agent> = agent_file.unwrap_or(rand_agents());
    let mut companies: Vec<Company> = company_file.unwrap_or(load_from_file("company_data.txt")
        .unwrap_or(rand_companies()));

    let mut market = Market::new();

    let agent_ids: Vec<u64> = agents.iter().map(|agent| agent.id).collect();
    let company_ids: Vec<u64> = companies.iter().map(|company| company.id).collect();

    // CURRENT SETUP:
    // 1. The market will ticked 1000 times, and each time every agent will do a random trade
    // 2. The trade will be either buy or sell, and the company will be random
    // 3. The strike price will be 100.0 +- 10.0, and the acceptable strike price deviation will be 5.0
    // 4. Give random agents some shares to start the buying and selling process IF the agents data file is not found

    if flag_give_random_stocks_to_random_agents {
        give_random_shares_to_random_agents(&mut agents, &companies);
    }

    let mut market_values: HashMap<u64, MarketValue> = HashMap::new();
    let mut expired_trades: HashMap<u64, Vec<FailedOffer<Trade>>>;
    let mut expired_options: HashMap<u64, Vec<FailedOffer<StockOption>>>;
    let mut todo_transactions: Vec<TodoTransactions> = Vec::new();

    for company in companies.iter() {
        market_values.insert(company.id, company.market_value.clone());
    }

    market.load_market_values(&market_values);

    for _ in 0..1000 {
        (market_values, expired_trades, expired_options) = market.tick();
        alert_agents(&mut agents, &expired_trades, &expired_options);

        for agent_id in agent_ids.iter() {
            let company_id = &company_ids[rand::random::<usize>() % companies.len()];
            let strike_price = max(MIN_STRIKE_PRICE, get_market_value_current_price(&market_values, company_id) + (random::<f64>() - 0.5) * 20.0);
            let trade = Trade::new(12);

            previously_failed_transactions(&mut todo_transactions, &agents, *agent_id, &trade);
            let agent = get_agent(&agents, *agent_id).unwrap();
            if random::<f64>() > 0.5 {
                if !agent.can_buy(strike_price, trade.number_of_shares) {
                    continue;
                }
                todo_transactions.push(TodoTransactions {
                    agent_id: *agent_id,
                    company_id: *company_id,
                    strike_price,
                    action: OfferAsk::Buy,
                    trade: trade.clone(),
                });
                continue;
            }
            if !agent.can_sell(*company_id, trade.number_of_shares) {
                continue;
            }
            todo_transactions.push(TodoTransactions {
                agent_id: *agent_id,
                company_id: *company_id,
                strike_price,
                action: OfferAsk::Sell,
                trade: trade.clone(),
            });
        }
        run_todo_transactions(&mut todo_transactions, &mut market, &mut agents);
        todo_transactions.clear();
    }

    // update market values to companies data
    for company in companies.iter_mut() {
        company.market_value = market_values
            .get(&company.id)
            .map(|value| value.clone())
            .unwrap_or(MarketValue::new());
    }

    if let Err(e) = save(agents, AGENTS_DATA_FILENAME) {
        log!(warn "Failed to save agents data\n{:?}", e);
    } else {
        log!(info "Saved agents");
    }
    if let Err(e) = save(companies, COMPANIES_DATA_FILENAME) {
        log!(warn "Failed to save company data\n{:?}", e);
    } else {
        log!(info "Saved companies");
    }
    log!(info "Exit");
}

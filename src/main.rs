use rand::random;
use stocks::{
    agent::{Agent, Company, SYMBOL_LENGTH},
    load,
    log,
    logger::Log,
    market::{ActionState, Market},
    save,
    trade_house::{OfferAsk, Trade},
    transaction::Transaction,
};
use std::io::BufRead;

static NUM_OF_AGENTS: u64 = 1000;
static NUM_OF_COMPANIES: u64 = 100;

static AGENTS_DATA_FILENAME: &str = "data/agents.yaml";
static COMPANIES_DATA_FILENAME: &str = "data/companies.yaml";

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
            Company::new(i as u64, name, convert_to_symbol(symbol), 0)
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

    buyer.balance -= transaction.strike_price * (transaction.number_of_shares as f64);
    buyer.holdings.push_from_txn(&transaction);
    seller.balance += transaction.strike_price * (transaction.number_of_shares as f64);
    seller.holdings.pop_from_txn(&transaction);
}

fn buy_random(
    market: &mut Market,
    agents: &mut Vec<Agent>,
    company_id: u64,
    agent_id: &u64,
    strike_price: f64,
    acceptable_strike_price_deviation: f64,
    trade: &Trade,
) {
    let result = market.buy_trade(*agent_id, company_id, strike_price, acceptable_strike_price_deviation, trade);
    match result {
        Ok(action_state) => {
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
        Err(offer_idxs) => {
            // 30% chance of accept this offer
            if random::<f64>() > 0.3 {
                return;
            }

            // choose a random offer
            let offer_idx = offer_idxs[random::<usize>() % offer_idxs.len()];
            let offer = market.house.get_mut_trade_offers(company_id).seller_offers[offer_idx].clone();

            let (transaction, extra_shares_left) = market.buy_trade_offer(
                company_id,
                &offer,
                *agent_id,
                trade
            );
            if extra_shares_left > 0 {
                market.house.add_trade_offer(
                    *agent_id,
                    company_id,
                    strike_price,
                    Trade::new(extra_shares_left),
                    OfferAsk::Buy,
                );
            }
            market.add_transaction(company_id, transaction.strike_price);
            exchange_currency_from_transaction(agents, &transaction);
        }
    }
}
fn sell_random(
    market: &mut Market,
    agents: &mut Vec<Agent>,
    company_id: u64,
    agent_id: &u64,
    strike_price: f64,
    acceptable_strike_price_deviation: f64,
    trade: &Trade,
) {
    let result = market.buy_trade(*agent_id, company_id, strike_price, acceptable_strike_price_deviation, &trade);
    match result {
        Ok(action_state) => {
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
        Err(offer_idxs) => {
            // 30% chance of accept this offer
            if random::<f64>() > 0.3 {
                return;
            }

            // choose a random offer
            let offer_idx = offer_idxs[random::<usize>() % offer_idxs.len()];
            let offer = market.house.get_mut_trade_offers(company_id).seller_offers[offer_idx].clone();

            let (transaction, extra_shares_left) = market.sell_trade_offer(
                company_id,
                &offer,
                *agent_id,
                trade
            );
            if extra_shares_left > 0 {
                market.house.add_trade_offer(
                    *agent_id,
                    company_id,
                    strike_price,
                    Trade::new(extra_shares_left),
                    OfferAsk::Sell,
                );
            }
            market.add_transaction(company_id, transaction.strike_price);
        }
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

fn give_random_shares_to_random_agents(agents: &mut Vec<Agent>, companies: &Vec<Company>) {
    // actually, give half agents some shares to random companies
    for i in 0..(NUM_OF_AGENTS/2) {
        let agent = &mut agents[i as usize];
        let random_company = &companies[random::<usize>() % companies.len()];
        agent.holdings.holdings.insert(random_company.id, random::<u64>() % 1000);
    }
}

fn main() {
    let agent_file = load(AGENTS_DATA_FILENAME);
    let company_file = load(COMPANIES_DATA_FILENAME);

    if agent_file.is_ok() {
        log!(info "Loaded agents");
    } else {
        log!(warn "Agents file not found");
    }
    if company_file.is_ok() {
        log!(info "Loaded companies");
    } else {
        log!(warn "Company file not found");
    }

    let mut agents: Vec<Agent> = agent_file.unwrap_or(rand_agents());
    let companies: Vec<Company> = company_file.unwrap_or(load_from_file("company_data.txt")
        .unwrap_or(rand_companies()));


    let mut market = Market::new();

    let agent_ids: Vec<u64> = agents.iter().map(|agent| agent.id).collect();
    let company_ids: Vec<u64> = companies.iter().map(|company| company.id).collect();

    // CURRENT SETUP:
    // 1. The market will ticked 1000 times, and each time every agent will do a random trade
    // 2. The trade will be either buy or sell, and the company will be random
    // 3. The strike price will be 100.0 +- 10.0, and the acceptable strike price deviation will be 5.0
    // 4. Give random agents some shares to start the buying and selling process

    give_random_shares_to_random_agents(&mut agents, &companies);


    for _ in 0..10 {
        let _market_values = market.tick();
        for agent_id in agent_ids.iter() {
            let company_id = &company_ids[rand::random::<usize>() % companies.len()];
            let price = 100.0 + (random::<f64>() - 0.5) * 20.0;
            let trade = Trade::new(12);
            let strike_price = price;
            let acceptable_strike_price_deviation = 5.0;

            let agent = get_agent(&agents, *agent_id).unwrap();
            if random::<f64>() > 0.5 {
                if agent.can_buy(strike_price, trade.number_of_shares) {
                    buy_random(&mut market, &mut agents, *company_id, agent_id, strike_price, acceptable_strike_price_deviation, &trade);
                    println!("Agent {} bought {} shares of company {} at price {}", agent_id, trade.number_of_shares, company_id, strike_price);
                }
            } else {
                if agent.can_sell(*company_id, trade.number_of_shares) {
                    sell_random(&mut market, &mut agents, *company_id, agent_id, strike_price, acceptable_strike_price_deviation, &trade);
                    println!("Agent {} sold {} shares of company {} at price {}", agent_id, trade.number_of_shares, company_id, strike_price);
                }
            }
        }
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

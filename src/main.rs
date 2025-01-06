use rand::random;
use std::{collections::HashMap, io::BufRead};
use stocks::{
    agent::{Agent, Agents, Companies, Company, SYMBOL_LENGTH},
    load, log,
    logger::Log,
    market::{Market, MarketValue},
    max, save,
    trade_house::{FailedOffer, StockOption, Trade, TradeAction},
    transaction::TodoTransactions,
    AGENTS_DATA_FILENAME, COMPANIES_DATA_FILENAME, MIN_STRIKE_PRICE, NUM_OF_AGENTS,
    NUM_OF_COMPANIES,
};

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
            Company::new(
                i as u64,
                name,
                convert_to_symbol(symbol),
                MarketValue::rand(),
            )
        })
        .collect();

    Ok(companies)
}

fn get_market_value_current_price(
    market_values: &HashMap<u64, MarketValue>,
    company_id: &u64,
) -> f64 {
    match market_values.get(company_id) {
        Some(value) => value.current_price,
        None => 0.0,
    }
}

fn roll_action(agent: &Agent, company_id: &u64, strike_price: f64, trade: &Trade) -> Option<TradeAction> {
    if random::<f64>() > 0.5 {
        if !agent.can_buy(strike_price, trade.number_of_shares) {
            return None;
        }
        return Some(TradeAction::Buy);
    }
    if !agent.can_sell(*company_id, trade.number_of_shares) {
        return None
    }
    return Some(TradeAction::Sell);
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

    let mut agents = Agents::new(agent_file.unwrap_or(rand_agents()));

    let mut companies = Companies::new(company_file
        .unwrap_or(load_from_file("company_data.txt")
        .unwrap_or(rand_companies()))
    );

    let mut market = Market::new();


    // CURRENT SETUP:
    // 1. The market will ticked 100 times, and each time every agent will do a random trade
    // 2. The trade will be either buy or sell, and the company will be random
    // 3. The strike price will be 100.0 +- 10.0, and the acceptable strike price deviation will be 5.0
    // 4. Give random agents some shares to start the buying and selling process IF the agents data file is not found

    if flag_give_random_stocks_to_random_agents {
        agents.give_random_shares_to_half_agents(&companies.company_ids);
    }

    let mut expired_trades: HashMap<u64, Vec<FailedOffer<Trade>>>;
    let mut expired_options: HashMap<u64, Vec<FailedOffer<StockOption>>>;

    let mut market_values: HashMap<u64, MarketValue> = HashMap::new();
    companies.load_market_values(&mut market_values);
    market.dump_market_values(&market_values);

    let mut todo_transactions: Vec<TodoTransactions> = Vec::new();
    for i in 0..100 {
        println!("{}", i);
        (market_values, expired_trades, expired_options) = market.tick();
        agents.alert_agents(&expired_trades, &expired_options);

        for agent_id in agents.id_iter() {
            let company_id = companies.rand_company_id();
            let strike_price = max(
                MIN_STRIKE_PRICE,
                get_market_value_current_price(&market_values, &company_id)
                    + (random::<f64>() - 0.5) * 20.0,
            );
            let trade = Trade::new(12);

            let Some(agent) = agents.get_agent(*agent_id) else {
                continue;
            };
            agent.try_failed_offers(&mut todo_transactions, *agent_id, &trade);
            let action = match roll_action(agent, &company_id, strike_price, &trade) {
                Some(action) => action,
                None => continue,
            };
            todo_transactions.push(TodoTransactions {
                agent_id: *agent_id,
                company_id,
                strike_price,
                action,
                trade,
            });
        }
        agents.do_transactions(&mut market, &mut todo_transactions);
        agents.clear_failed_transactions();
        todo_transactions.clear();
    }

    companies.dump_market_values(&mut market_values);

    if let Err(e) = save(agents.agents, AGENTS_DATA_FILENAME) {
        log!(warn "Failed to save agents data\n{:?}", e);
    } else {
        log!(info "Saved agents");
    }
    if let Err(e) = save(companies.companies, COMPANIES_DATA_FILENAME) {
        log!(warn "Failed to save company data\n{:?}", e);
    } else {
        log!(info "Saved companies");
    }
    log!(info "Exit");
}

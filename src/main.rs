use rand::{thread_rng, Rng};
use std::collections::HashMap;
use stocks::{
    entities::{
        agents::{Agent, Agents},
        companies::{Companies, Company},
    },
    load, log,
    logger::Log,
    market::Market,
    max, save,
    trade_house::{FailedOffer, StockOption, Trade, TradeAction},
    transaction::{CompanyTransaction, TodoTransactions},
    SimulationError, AGENTS_DATA_FILENAME, COMPANIES_DATA_FILENAME, MIN_STRIKE_PRICE,
    NUM_OF_AGENTS, NUM_OF_COMPANIES,
};

fn main() {
    let mut rng = thread_rng();
    log!(info "Loading local file data");
    let agent_file = load::<Vec<Agent>>(AGENTS_DATA_FILENAME);
    let company_file = load::<Vec<Company>>(COMPANIES_DATA_FILENAME);

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

    let mut companies = if let Ok(company_data) = company_file {
        Companies::load(company_data.as_slice())
    } else {
        Companies::rand(NUM_OF_COMPANIES as usize, &mut rng)
    };

    let mut agents = if let Ok(agent_data) = agent_file {
        Agents::load(agent_data.as_slice())
    } else {
        let mut a = Agents::new();
        a.introduce_new_rand_agents(&mut rng, NUM_OF_AGENTS, companies.num_of_companies)
            .unwrap();
        a
    };

    let mut market = Market::new();

    // CURRENT SETUP:
    // 1. The market will ticked 100 times, and each time every agent will do a random trade
    // 2. The trade will be either buy or sell, and the company will be random
    // 3. The strike price will be 100.0 +- 10.0, and the acceptable strike price deviation will be 5.0
    // 4. Give random agents some shares to start the buying and selling process IF the agents data file is not found

    if flag_give_random_stocks_to_random_agents {
        agents
            .give_random_preferences(&mut rng, companies.num_of_companies)
            .unwrap();
        agents.give_random_assets(&mut rng, &companies).unwrap();
    }

    let mut expired_trades: HashMap<u64, Vec<FailedOffer<Trade>>> = HashMap::new();
    let mut expired_options: HashMap<u64, Vec<FailedOffer<StockOption>>> = HashMap::new();

    let mut todo_transactions: Vec<TodoTransactions> = Vec::new();
    let mut todo_company_transactions: Vec<CompanyTransaction> = Vec::new();

    let trade = Trade::new(10);
    agents
        .try_failed_offers(&mut rng, &mut todo_transactions, &trade)
        .unwrap();
    for i in 0..100 {
        agents.try_offers.clear();
        todo_transactions.clear();
        expired_trades.clear();
        expired_options.clear();
        println!("{}", i);
        if i % 5 == 0 {
            market.tick_failures(&mut expired_trades, &mut expired_options);
            for company_id in companies.iter() {
                let Some(market_value) = companies.market_values.get_mut(company_id as usize)
                else {
                    continue;
                };
                market.tick_individual_company(company_id, market_value);
            }
        }
        if i % 20 == 0 {
            companies.release_budget(&mut rng);
        }
        agents
            .alert_agents(&expired_trades, &expired_options)
            .unwrap();

        for agent_id in agents.iter() {
            let (company_id, action) = agents
                .preferences
                .get_preferred_random(agent_id, &mut rng)
                .unwrap();
            let strike_price = max(
                MIN_STRIKE_PRICE,
                companies.get_current_price(company_id) + rng.gen_range(-10.0..10.0),
            );

            if action == TradeAction::Buy {
                todo_company_transactions.push(CompanyTransaction::new(
                    agent_id,
                    company_id,
                    trade.number_of_shares,
                    strike_price,
                ));
            }

            todo_transactions.push(TodoTransactions {
                agent_id,
                company_id,
                strike_price,
                action,
                trade: trade.clone(),
            });
        }
        let Err(e) = agents.do_transactions(&mut market, &mut rng, &mut todo_transactions) else {
            continue;
        };
        match e {
            SimulationError::AgentNotFound(agent_id) => {
                log!(warn "Agent not found: {}", agent_id);
            }
            SimulationError::NoData => {
                log!(warn "No data");
            }
            SimulationError::Unspendable => {
                continue;
            }
        }
    }

    log!(info "Saving data");

    if let Err(e) = save(agents.save().unwrap(), AGENTS_DATA_FILENAME) {
        log!(warn "Failed to save agents data\n{:?}", e);
    } else {
        log!(info "Saved agents");
    }
    if let Err(e) = save(companies.save(), COMPANIES_DATA_FILENAME) {
        log!(warn "Failed to save company data\n{:?}", e);
    } else {
        log!(info "Saved companies");
    }
    log!(info "Exit");
}

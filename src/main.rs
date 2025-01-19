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
        Companies::rand(NUM_OF_COMPANIES as usize, 0, &mut rng)
    };

    let mut agents = if let Ok(agent_data) = agent_file {
        Agents::load(agent_data.as_slice())
    } else {
        let mut a = Agents::new();
        let rng1 = thread_rng();
        let rng2 = thread_rng();
        a.rand_introduce_new_agents(rng1, rng2, NUM_OF_AGENTS, companies.num_of_companies)
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
        let rng1 = thread_rng();
        agents
            .rand_give_preferences(rng1, companies.num_of_companies)
            .unwrap();
        // agents.rand_give_assets(&mut rng, &companies).unwrap();
    }

    let mut expired_trades: HashMap<u64, Vec<FailedOffer<Trade>>> = HashMap::new();
    let mut expired_options: HashMap<u64, Vec<FailedOffer<StockOption>>> = HashMap::new();

    let mut todo_transactions: Vec<TodoTransactions> = Vec::new();

    let trade = Trade::new(10);
    agents
        .try_failed_offers(&mut rng, &mut todo_transactions, &trade)
        .unwrap();
    for i in 0..100 {
        agents.try_offers.clear();
        println!("{}", i);
        if i % 5 == 0 {
            for company_id in companies.iter() {
                let Some(market_value) = companies.market_values.get_mut(company_id as usize)
                else {
                    continue;
                };
                market.tick_individual_company(company_id, market_value);
            }
            market.tick_failures(&mut expired_trades, &mut expired_options);
        }
        if i % 20 == 0 {
            companies.rand_release_news(&mut agents, &mut rng);
        }
        agents
            .alert_agents(&expired_trades, &expired_options)
            .unwrap();
        expired_trades.clear();
        expired_options.clear();

        for agent_id in agents.iter() {
            let (company_id, mut action) = agents
                .preferences
                .get_preferred_random(agent_id, &mut rng)
                .unwrap();

            // small portion of people who sell low and buy high, because .... IDK WHY
            if rng.gen_ratio(5, 100) {
                action = action.complement();
            }

            let failable_value = rng.gen_range(10.0..2_000.0);
            let current_price = companies.get_current_price(company_id).unwrap_or(failable_value);
            companies.market_values[company_id as usize].current_price = current_price;
            let strike_price = max(
                MIN_STRIKE_PRICE,
                current_price + rng.gen_range(-10.0..10.0),
            );

            todo_transactions.push(TodoTransactions {
                agent_id,
                company_id,
                strike_price,
                action,
                trade: trade.clone(),
            });
        }
        let news_probability_distribution = 
            &companies.generate_preferences_from_news(&mut rng);
        agents.rand_give_preferences_from_news(
            &mut rng,
            &news_probability_distribution
        );
        let Err(e) =
            market.rand_do_trade(&mut rng, &mut agents, &mut companies, &mut todo_transactions)
        else {
            todo_transactions.clear();
            continue;
        };
        todo_transactions.clear();
        match e {
            SimulationError::AgentNotFound(agent_id) => {
                log!(warn "Agent not found: {}", agent_id);
            }
            SimulationError::NoData => {
                log!(warn "No data");
            }
            SimulationError::Unspendable | SimulationError::UnDoable => {
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

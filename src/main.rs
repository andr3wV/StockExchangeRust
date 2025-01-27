// Main thing to do now is for agents to hold long for certain companies

use rand::{thread_rng, Rng};
use rand_distr::{Normal, Distribution};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use ctrlc;
use stocks::{
    entities::{
        agents::{Agent, Agents},
        companies::{Companies, Company},
    },
    load, log,
    logger::Log,
    market::Market,
    max, save,
    trade_house::{FailedOffer, StockOption, Trade},
    transaction::TodoTransaction,
    SimulationError, AGENTS_DATA_FILENAME, COMPANIES_DATA_FILENAME, MIN_STRIKE_PRICE,
    NUM_OF_AGENTS, NUM_OF_COMPANIES,
};

fn spend_function(x: f64) -> f64 {
    // went off feeling
    0.99 * (1.0 - (-0.01 * x * x).exp()) + 0.01
}

fn rand_spend_portion_wealth(rng: &mut impl Rng) -> f64 {
    let Ok(normal) = Normal::new(0.0, 1.0) else {
        // If the normal distribution fails, fuck it then
        return 0.01;
    };
    spend_function(normal.sample(rng))
}

fn main() {
    let mut rng = thread_rng();
    log!(info "Loading local file data");
    let agent_file = load::<Vec<Agent>>(AGENTS_DATA_FILENAME);
    let company_file = load::<Vec<Company>>(COMPANIES_DATA_FILENAME);

    let mut flag_give_random_stocks_to_random_agents = false;

    if let Err(ref e) = agent_file {
        log!(warn "Agents file not found\n{:?}", e);
        flag_give_random_stocks_to_random_agents = true;
    } else {
        log!(info "Loaded agents");
    }
    if let Err(ref e) = company_file {
        log!(warn "Company file not found\n{:?}", e);
    } else {
        log!(info "Loaded companies");
    }

    if company_file.is_ok() {
    } else {
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

    /*
    let len = agents.balances.0.len() as f64;
    println!("{:?}", agents.balances.0.clone().into_iter().max_by(|a, b| a.partial_cmp(b).unwrap()));
    println!("{:?}", agents.balances.0.clone().into_iter().min_by(|a, b| a.partial_cmp(b).unwrap()));
    println!("{:?}", agents.balances.0.into_iter().sum::<f64>() / len);

    std::process::exit(0);
    */

    let mut market = Market::new();

    if flag_give_random_stocks_to_random_agents {
        let rng1 = thread_rng();
        agents
            .rand_give_preferences(rng1, companies.num_of_companies)
            .unwrap();
    }

    let mut expired_trades: HashMap<u64, Vec<FailedOffer<Trade>>> = HashMap::new();
    let mut expired_options: HashMap<u64, Vec<FailedOffer<StockOption>>> = HashMap::new();

    let mut todo_transactions: Vec<TodoTransaction> = Vec::new();

    let trade = Trade::new(10);
    agents
        .try_failed_offers(&mut rng, &mut todo_transactions, &trade)
        .unwrap();

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let mut i: i128 = 0;
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    }).expect("Error setting Ctrl-C handler");
    while running.load(Ordering::SeqCst) {
        i += 1;
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
            let current_price = companies
                .get_current_price(company_id)
                .unwrap_or(failable_value);
            companies.market_values[company_id as usize].current_price = current_price;
            let strike_price = max(MIN_STRIKE_PRICE, current_price + rng.gen_range(-10.0..10.0));
            let want_to_spend = agents.balances.get(agent_id).unwrap() * rand_spend_portion_wealth(&mut rng);
            let rough_amount_of_stocks = (want_to_spend / strike_price).floor() as u64;
            if rough_amount_of_stocks == 0 {
                // bruh, just don't trade anything
                continue;
            }

            todo_transactions.push(TodoTransaction {
                agent_id,
                company_id,
                strike_price,
                action,
                trade: Trade::new(rough_amount_of_stocks),
            });
        }
        let news_probability_distribution = &companies.generate_preferences_from_news(&mut rng);
        agents.rand_give_preferences_from_news(&mut rng, &news_probability_distribution);
        let Err(e) = market.rand_do_trade(
            &mut rng,
            &mut agents,
            &mut companies,
            &mut todo_transactions,
        ) else {
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
    log!(info "Exiting at index {:?}", i);
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

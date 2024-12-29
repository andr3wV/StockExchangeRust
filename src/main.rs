use stocks::{
    agent::{Agent, Company, SYMBOL_LENGTH},
    load, log,
    log::Log,
    save,
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

fn main() {
    let agents: Vec<Agent> = load(AGENTS_DATA_FILENAME).unwrap_or(rand_agents());
    let companies: Vec<Company> = load(COMPANIES_DATA_FILENAME)
        .unwrap_or(load_from_file("company_data.txt")
        .unwrap_or(rand_companies()));

    if let Err(e) = save(agents, AGENTS_DATA_FILENAME) {
        log!(warn "Failed to save agents data\n{:?}", e);
    }
    if let Err(e) = save(companies, COMPANIES_DATA_FILENAME) {
        log!(warn "Failed to save company data\n{:?}", e);
    }
    println!("Hello World");
}

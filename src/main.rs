use stocks::{
    agent::{Agent, Company},
    load, log,
    log::Log,
    save,
};

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

fn main() {
    let agents: Vec<Agent> = load(AGENTS_DATA_FILENAME).unwrap_or(rand_agents());
    let companies: Vec<Company> = load(COMPANIES_DATA_FILENAME).unwrap_or(rand_companies());

    if let Err(e) = save(agents, AGENTS_DATA_FILENAME) {
        log!(warn "Failed to save agents data\n{:?}", e);
    }
    if let Err(e) = save(companies, COMPANIES_DATA_FILENAME) {
        log!(warn "Failed to save company data\n{:?}", e);
    }
    println!("Hello World");
}

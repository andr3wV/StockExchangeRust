use crate::transaction::Holding;

pub struct Agent {
    id: u64,
    /// How much money does the agent have
    balance: f64,
    /// How many shares does an agent hold in a company
    holdings: Vec<Holding>,
}

pub struct Company {
    id: u64,
    /// Price per share
    market_rate: f64,
    /// Number of total shares
    total_shares: u64,
    /// Number of available shares
    /// ! MAYBE??
    available_shares: u64,
}

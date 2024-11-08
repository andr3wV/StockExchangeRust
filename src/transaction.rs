use serde::{Deserialize, Serialize};

/// Represents an exchange of captial between 2 agents
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    /// The agent which gave away his shares
    buyer_id: u64,
    /// The agent which bought the shares
    seller_id: u64,

    company_id: u64,
    number_of_shares: u64,
    /// The price per share at which the exchange was done
    strike_price: f64,
}

/// Represents the number of shares held by an agent for a particular company
#[derive(Serialize, Deserialize, Debug)]
pub struct Holding {
    company_id: u64,
    number_of_shares: u64,
}

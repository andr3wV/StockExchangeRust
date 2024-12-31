use serde::{Deserialize, Serialize};

// Todo Decide if you want to store the Transaction
/// Represents an exchange of captial between 2 agents
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    /// The agent which gave away his shares
    pub buyer_id: u64,
    /// The agent which bought the shares
    pub seller_id: u64,

    pub company_id: u64,
    pub number_of_shares: u64,
    /// The price per share at which the exchange was done
    pub strike_price: f64,
}

impl Transaction {
    pub fn new(buyer_id: u64, seller_id: u64, company_id: u64, number_of_shares: u64, strike_price: f64) -> Self {
        Self {
            buyer_id,
            seller_id,
            company_id,
            number_of_shares,
            strike_price,
        }
    }
}

/// Represents the number of shares held by an agent for a particular company
#[derive(Serialize, Deserialize, Debug)]
pub struct Holding {
    company_id: u64,
    number_of_shares: u64,
}

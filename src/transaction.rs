use crate::{log, logger::Log};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub fn new(
        buyer_id: u64,
        seller_id: u64,
        company_id: u64,
        number_of_shares: u64,
        strike_price: f64,
    ) -> Self {
        log!(info "Transaction: buyer_id: {}, seller_id: {}, company_id: {}, number_of_shares: {}, strike_price: {}", buyer_id, seller_id, company_id, number_of_shares, strike_price);
        Self {
            buyer_id,
            seller_id,
            company_id,
            number_of_shares,
            strike_price,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Holdings(HashMap<u64, u64>);

impl Holdings {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn insert(&mut self, company_id: u64, number_of_shares: u64) {
        self.0.insert(company_id, number_of_shares);
    }
    pub fn get(&self, company_id: &u64) -> u64 {
        match self.0.get(company_id) {
            Some(share_count) => *share_count,
            None => 0,
        }
    }
    pub fn push_from_txn(&mut self, transaction: &Transaction) {
        let company = self.0.get_mut(&transaction.company_id);
        match company {
            Some(share_count) => {
                *share_count += transaction.number_of_shares;
            }
            None => {
                self.0
                    .insert(transaction.company_id, transaction.number_of_shares);
            }
        }
    }
    pub fn pop_from_txn(&mut self, transaction: &Transaction) -> bool {
        let company = self.0.get_mut(&transaction.company_id);
        match company {
            Some(share_count) => {
                *share_count -= transaction.number_of_shares;
                return true;
            }
            None => {
                return false;
            }
        }
    }

    pub fn push(&mut self, company_id: u64, number_of_shares: u64) {
        let company = self.0.get_mut(&company_id);
        match company {
            Some(share_count) => {
                *share_count += number_of_shares;
            }
            None => {
                self.0.insert(company_id, number_of_shares);
            }
        }
    }

    pub fn pop(&mut self, company_id: u64, number_of_shares: u64) -> bool {
        let company = self.0.get_mut(&company_id);
        match company {
            Some(share_count) => {
                *share_count -= number_of_shares;
                return true;
            }
            None => {
                return false;
            }
        }
    }
}

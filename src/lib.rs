use std::{
    fs::File,
    io::{BufReader, BufWriter},
};

use serde::{de::DeserializeOwned, Serialize};

pub mod entities;
pub mod logger;
pub mod market;
pub mod trade_house;
pub mod transaction;

pub static NUM_OF_AGENTS: u64 = 10_000;
pub static NUM_OF_COMPANIES: u64 = 100;

pub static AGENTS_DATA_FILENAME: &str = "data/agents.bin";
pub static COMPANIES_DATA_FILENAME: &str = "data/companies.bin";

pub static MIN_STRIKE_PRICE: f64 = 5.0;
pub static OFFER_LIFETIME: u64 = 10;
pub static TIMELINE_SIZE_LIMIT: usize = 1000;

#[derive(Debug)]
pub enum SerializationError {
    FailedToCreateFile,
    FailedToSerialize,
    FailedToWrite,
}

#[derive(Debug)]
pub enum DeserializationError {
    FileNotFound,
    FailedToSerialize,
    FailedToReadFile,
}

#[derive(Debug)]
pub enum SimulationError {
    AgentNotFound(u64),
    Unspendable,
    NoData,
    UnDoable,
}

pub fn save<T: Serialize>(data: T, file_path: &str) -> Result<(), SerializationError> {
    let Ok(file) = File::create(file_path) else {
        return Err(SerializationError::FailedToCreateFile);
    };
    let writer = BufWriter::new(file);
    let Ok(data) = bincode::serialize_into(writer, &data) else {
        return Err(SerializationError::FailedToSerialize);
    };
    Ok(data)
}

pub fn load<T: DeserializeOwned>(file_path: &str) -> Result<T, DeserializationError> {
    let Ok(file) = File::open(file_path) else {
        return Err(DeserializationError::FileNotFound);
    };
    let mut reader = BufReader::new(file);
    let Ok(data) = bincode::deserialize_from(&mut reader) else {
        return Err(DeserializationError::FailedToSerialize);
    };
    Ok(data)
}

pub fn max<T: PartialOrd>(a: T, b: T) -> T {
    if a > b {
        a
    } else {
        b
    }
}
pub fn min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

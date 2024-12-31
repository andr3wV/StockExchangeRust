use std::{
    fs::File,
    io::{Read, Write},
};

use serde::{de::DeserializeOwned, Serialize};

pub mod agent;
pub mod logger;
pub mod market;
pub mod trade_house;
pub mod transaction;

#[derive(Debug)]
pub enum SerializationError {
    FailedToCreateFile,
    FailedToSerializeData,
    FailedToWrite,
}

#[derive(Debug)]
pub enum DeserializationError {
    FileNotFound,
    FailedToSerialize,
    FailedToReadFile,
}

pub fn save<T: Serialize>(data: T, file_path: &str) -> Result<(), SerializationError> {
    let Ok(mut file) = File::create(file_path) else {
        return Err(SerializationError::FailedToCreateFile);
    };
    let Ok(str_data) = serde_yaml::to_string(&data) else {
        return Err(SerializationError::FailedToSerializeData);
    };
    if let Err(_) = file.write_all(str_data.as_bytes()) {
        return Err(SerializationError::FailedToWrite);
    }
    return Ok(());
}

pub fn load<T: DeserializeOwned>(file_path: &str) -> Result<T, DeserializationError> {
    let Ok(mut file) = File::open(file_path) else {
        return Err(DeserializationError::FileNotFound);
    };
    let mut str_data = String::new();
    if let Err(_) = file.read_to_string(&mut str_data) {
        return Err(DeserializationError::FailedToReadFile);
    }
    let Ok(data) = serde_yaml::from_str(&str_data) else {
        return Err(DeserializationError::FailedToSerialize);
    };
    return Ok(data);
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
use std::collections::HashMap;

/// Basically stores all the requested trades that weren't immediately resolved
struct House {
    stock_trade: HashMap<u64, OpenTrades>,
    option_trade: HashMap<u64, OpenOptions>,
}

/// All the trades of the certain company
struct OpenTrades {
    trades: Vec<Trade>,
    lowest_strike_price: f64,
    highest_strike_price: f64,
}

/// A specific trade offer
struct Trade {
    id: u64,
    transactor_id: u64,
    strike_price: f64,
    number_of_shares: u64,
}

/// All the options of the certain company
struct OpenOptions {
    options: Vec<Option>,
    // todo: figure shit out here
}

/// A specific option offer
struct Option {
    // todo: figure shit out here
}

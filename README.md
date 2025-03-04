Stock Market Simulation
=======================

A Rust-based stock market simulation framework designed to model realistic trading behaviors and market dynamics.

Technical Features
------------------

*   **Agent-Based Trading System**: Simulates thousands of autonomous agents with individual preferences, balances, and investment strategies
    
*   **Dynamic Market Mechanisms**: Implements price discovery, order matching, and trade execution systems
    
*   **Realistic Price Movements**: Uses statistical distributions to model market volatility and price changes
    
*   **News Event Simulation**: Generates company news that affects stock prices and influences agent behavior
    
*   **Company Share Offerings**: Models IPOs and share releases with configurable lot sizes and strike prices
    
*   **Order Book Implementation**: Maintains buy/sell offers with lifetime management and automatic expiration
    
*   **Trade Matching Algorithm**: Pairs compatible buy/sell orders based on configurable price tolerances
    
*   **Asset Transfer System**: Handles the exchange of shares and capital between agents
    
*   **Preference-Based Agent Behavior**: Agents develop preferences for specific companies based on performance
    
*   **Persistence Layer**: Saves and loads simulation state via binary serialization
    

Technical Specifications
------------------------
    
*   **Concurrency**: Thread-safe design with atomic operations
    
*   **Error Handling**: Comprehensive error types and propagation
    
*   **Testing**: Unit tests for core functionality including asset transfers and trade execution
    
*   **Serialization**: Binary serialization using bincode for state persistence
    

This project was created to better understand stock market dynamics through simulation and experimentation.

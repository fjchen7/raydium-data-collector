
# Solana Raydium Data Collector

This project is an implementation to collect Raydium market data in real time, including trade and price data for any CLMM (concentrated liquidity market making) pool.

## Environment Setup

Before running the project, you need Rust installed. Following the instructions on the [Rust official website](https://www.rust-lang.org/tools/install).

This project gets data from Solana websocket endpoint. You can get a free Solana websocket endpoint from [Alchemy](https://www.alchemy.com/) or [QuikNode](https://quiknode.io/). For simplicity, just apply for mainnet endpoint as the default environment settings are for the mainnet.

## How to Run

Copy the `.env.example` file to `.env` and fill in the `WS_URL` of Solana mainnet and leave the rest as default.

Execute the following command:

```shell
cargo run
```

It will start collecting trade data and save it to a CSV file per second. Each row represents a trade, and the columns are as follows:
- `timestamp`: The timestamp of the trade data.
- `symbol`: The symbol of the pool, e.g. USDC-SQL.1bp
- `trade_price`: The price (with decimals) of the trade, e.g. the number of USDC for 1 SOL.
- `trade_quantity`: The quantity (with decimals) of the trade, e.g. the number of SOL traded in this transaction.
- `trade_side`: The side of the trade, either `BUY` or `SELL`.

The default file path is `data.csv`, but you can change it in the `.env` file.
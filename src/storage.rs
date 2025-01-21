use std::fs;
use std::fs::{File, OpenOptions};
use csv::Writer;
use raydium_amm_v3::states::SwapEvent;

pub trait SwapEventHandler {
    fn handle_swap_event(&mut self, event: SwapEvent, timestamp: i64) -> anyhow::Result<()>;
}


pub struct CsvSwapEventHandler {
    symbol: String,
    writer: Writer<File>,
}

impl CsvSwapEventHandler {
    pub fn new(file_path: &str, symbol: &str) -> anyhow::Result<Self> {
        let file_exists = fs::metadata(file_path).is_ok();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let headers = ["timestamp", "symbol", "trade_price", "trade_quantity", "trade_side"];
        let mut writer = Writer::from_writer(file);
        if !file_exists {
            writer.write_record(&headers)?;
            writer.flush()?;
        }

        Ok(Self {
            writer,
            symbol: symbol.to_string(),
        })
    }
}


pub const Q_RATIO: f64 = 1.0001;
pub fn tick_to_price(tick: i32) -> f64 {
    Q_RATIO.powi(tick)
}

pub const SOL_DECIMAL: u32 = 9;
pub const USDC_DECIMAL: u32 = 6;

// SOL：精度2
//
impl SwapEventHandler for CsvSwapEventHandler {
    fn handle_swap_event(&mut self, event: SwapEvent, timestamp: i64) -> anyhow::Result<()> {
        // CSV column
        // - timestamp
        // - symbol: always USDC-SOL.1bp
        // - trade_price: the number of USDC for 1 SOL
        // - trade_quantity: the number of SOL traded in this transaction
        // - trade_side: BUY or SELL
        let timestamp = timestamp.to_string();
        let (trade_quantity, trade_side) = if event.zero_for_one {
            (event.amount_0, "BUY")
        } else {
            (event.amount_1, "SELL")
        };
        let trade_quantity = trade_quantity as f64 / 10u64.pow(SOL_DECIMAL) as f64;
        let trade_price = tick_to_price(event.tick);
        let trade_price = trade_price * (10u64.pow(SOL_DECIMAL - USDC_DECIMAL) as f64);
        let data = vec![
            vec![timestamp.to_string(), self.symbol.clone(), trade_price.to_string(), trade_quantity.to_string(), trade_side.to_string()]
        ];
        for row in &data {
            self.writer.write_record(row)?;
        }

        self.writer.flush()?;
        log::info!("timestamp {} write event {:#?}", timestamp, event);
        Ok(())
    }
}

pub struct DummySwapEventHandler {}

impl SwapEventHandler for DummySwapEventHandler {
    fn handle_swap_event(&mut self, event: SwapEvent, timestamp: i64) -> anyhow::Result<()> {
        log::info!("timestamp: {}, event: {:#?}", timestamp, event);
        Ok(())
    }
}

//
// /**
// [2025-01-21T07:22:16Z INFO  raydium_data_collector] SwapEvent {
//         pool_state: 8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj,
//         sender: HV1KXxWFaSeriyFvXyx48FqG9BoFbfinB8njCJonqP7K,
//         token_account_0: 2rikd7tzPbmowhUJzPNVtX7fuUGcnBa8jqJnx6HbtHeE,
//         token_account_1: muUJotr5nCNuBEd6aj25stTX8oe8M9mqotcjushGBZP,
//         amount_0: 2608897681,
//         transfer_fee_0: 0,
//         amount_1: 617359418,
//         transfer_fee_1: 0,
//         zero_for_one: true,
//         sqrt_price_x64: 8973873876726606866,
//         liquidity: 138036922165946,
//         tick: -14413,
//     }
// */
// // pub fn handle_swap_event(event: SwapEvent, timestamp: i64) {
// //     log::info!("{:#?}", event);
// //     // todo!()
// // }
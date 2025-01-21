use std::fs;
use std::fs::{File, OpenOptions};
use csv::Writer;
use raydium_amm_v3::states::SwapEvent;

pub trait SwapEventHandler {
    fn handle_swap_event(&mut self, event: SwapEvent, timestamp: i64) -> anyhow::Result<()>;
}


pub struct CsvSwapEventHandler {
    symbol: String,
    token_a_decimal: u32,
    token_b_decimal: u32,
    writer: Writer<File>,
}

pub const HEADERS: [&str; 5] = ["timestamp", "symbol", "trade_price", "trade_quantity", "trade_side"];
impl CsvSwapEventHandler {
    pub fn new(file_path: &str, symbol: &str, token_a_decimal: u32, token_b_decimal: u32) -> anyhow::Result<Self> {
        let file_exists = fs::metadata(file_path).is_ok();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)?;

        let mut writer = Writer::from_writer(file);
        if !file_exists {
            writer.write_record(&HEADERS)?;
            writer.flush()?;
        }

        Ok(Self {
            symbol: symbol.to_string(),
            token_a_decimal,
            token_b_decimal,
            writer,
        })
    }
}


pub const Q_RATIO: f64 = 1.0001;
pub fn tick_to_price(tick: i32) -> f64 {
    Q_RATIO.powi(tick)
}

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
        // let trade_quantity = trade_quantity as f64 / 10u64.pow(SOL_DECIMAL) as f64;
        let trade_quantity = trade_quantity as f64 / 10u64.pow(self.token_b_decimal) as f64;
        let trade_price = tick_to_price(event.tick);
        let trade_price = trade_price * (10u64.pow(self.token_b_decimal - self.token_a_decimal) as f64);
        let data = vec![
            vec![timestamp.to_string(), self.symbol.clone(), trade_price.to_string(), trade_quantity.to_string(), trade_side.to_string()]
        ];
        for row in &data {
            self.writer.write_record(row)?;
        }

        self.writer.flush()?;
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
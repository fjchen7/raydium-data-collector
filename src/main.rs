mod event_fetcher;
mod storage;
mod utils;

use crate::event_fetcher::EventFetcher;
use crate::storage::{CsvSwapEventHandler, SwapEventHandler};
use anchor_lang::Discriminator;
use futures_util::StreamExt;
use raydium_amm_v3::states::SwapEvent;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    env_logger::init();

    let ws_url = env::var("WS_URL")?;
    let pool_address = env::var("POOL_ADDRESS")?;
    let event_fetcher = event_fetcher::EventFetcher::connect(&ws_url, &pool_address)
        .await
        .unwrap();

    // let mut handler = DummySwapEventHandler {};
    let symbol = env::var("POOL_SYMBOL")?;
    let file_path = env::var("DATA_FILE_PATH")?;
    let token_a_decimal = env::var("POOL_TOKEN_A_DECIMAL")?.parse::<u32>().unwrap();
    let token_b_decimal = env::var("POOL_TOKEN_B_DECIMAL")?.parse::<u32>().unwrap();
    let mut handler =
        CsvSwapEventHandler::new(&file_path, &symbol, token_a_decimal, token_b_decimal)?;

    let interval = Duration::from_secs(1);
    fetch_market_data_and_store_periodically(event_fetcher, interval, &mut handler).await;
    Ok(())
}

pub async fn fetch_market_data_and_store_periodically<T: SwapEventHandler>(
    event_fetcher: EventFetcher,
    interval: Duration,
    handler: &mut T,
) {
    let mut latest_swap_event = Option::<(SwapEvent, i64)>::None;
    let (mut stream, _) = event_fetcher.subscribe().await.unwrap();
    log::info!(
        "Subscribed to swap_event event for pool_address: {}",
        event_fetcher.pool_address
    );
    let mut interval = tokio::time::interval(interval);
    loop {
        tokio::select! {
            // Handle event every interval
            _ = interval.tick() => {
                if let Some(latest_swap_event) = latest_swap_event.take() {
                    let (event, timestamp) = latest_swap_event;
                    log::info!("timestamp {} save event {:#?}", timestamp, event);
                    handler.handle_swap_event(event, timestamp).unwrap();
                }
            }
            // Get latest event from stream
            Some(response) = stream.next() => {
                let logs = response.value.logs;
                // Overwrite latest_swap_event if a new event is found
                if let Some(event) = filter_latest_swap_event(&logs) {
                    let now = time::OffsetDateTime::now_utc();
                    let timestamp = now.unix_timestamp();
                    latest_swap_event = Some((event, timestamp));
                }
            }
        }
    }
    // unsubscriber().await?
}

// const PROGRAM_LOG: &str = "Program log: ";
const PROGRAM_DATA: &str = "Program data: ";

// Reference: https://github.com/raydium-io/raydium-clmm/blob/master/client/src/instructions/events_instructions_parse.rs#L181-L183
pub fn filter_latest_swap_event(logs: &[String]) -> Option<SwapEvent> {
    let event = logs.iter().rev().find_map(|log: &String| {
        if let Some(log) = log.strip_prefix(PROGRAM_DATA) {
            let borsh_bytes = match anchor_lang::__private::base64::decode(log) {
                Ok(borsh_bytes) => borsh_bytes,
                _ => {
                    log::warn!("Could not base64 decode log: {}", log);
                    return None;
                }
            };
            let mut slice: &[u8] = &borsh_bytes[..];
            let disc: [u8; 8] = {
                let mut disc = [0; 8];
                disc.copy_from_slice(&borsh_bytes[..8]);
                slice = &slice[8..];
                disc
            };
            // A SwapEvent will be emitted when a trade occurs
            if matches!(disc, SwapEvent::DISCRIMINATOR) {
                return match decode_event(&mut slice) {
                    Ok(event) => Some(event),
                    Err(e) => {
                        log::warn!("Could not decode event: {}, log {}", e, log);
                        None
                    }
                };
            } else {
                None
            }
        } else {
            None
        }
    });
    event
}

fn decode_event<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
    slice: &mut &[u8],
) -> anyhow::Result<T> {
    let event: T = anchor_lang::AnchorDeserialize::deserialize(slice)?;
    Ok(event)
}

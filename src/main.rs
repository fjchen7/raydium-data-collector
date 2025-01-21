mod swap_event_fetcher;

use futures_util::StreamExt;
use std::time::Duration;
use anchor_lang::Discriminator;
use raydium_amm_v3::states::SwapEvent;
use crate::swap_event_fetcher::SwapEventFetcher;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    env_logger::init();

    let ws_url = std::env::var("WS_URL")?;
    let pool_address = std::env::var("POOL_ADDRESS")?;
    let event_fetcher = swap_event_fetcher::SwapEventFetcher::connect(&ws_url, &pool_address).await.unwrap();
    let interval = Duration::from_secs(1);
    fetch_market_data_and_store_periodically(event_fetcher, interval).await;
    Ok(())
}

pub async fn fetch_market_data_and_store_periodically(event_fetcher: SwapEventFetcher, interval: Duration) {
    let mut latest_swap_event = Option::<(SwapEvent, i64)>::None;
    let (mut stream, unsubscriber) = event_fetcher
        .subscribe()
        .await.unwrap();
    log::info!("Subscribed to swap_event event updates");
    let mut interval = tokio::time::interval(interval);
    loop {
        tokio::select! {
            // Handle event every interval
            _ = interval.tick() => {
                if let Some(latest_swap_event) = latest_swap_event.take() {
                    let now = time::OffsetDateTime::now_utc();
                    let timestamp_sec = now.unix_timestamp();
                    log::info!("Handle SwapEvent, timestamp {}", timestamp_sec);
                    handle_swap_event(latest_swap_event.0, latest_swap_event.1);
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

pub fn handle_swap_event(event: SwapEvent, timestamp: i64) {
    log::info!("{:#?}", event);
    // todo!()
}

// const PROGRAM_LOG: &str = "Program log: ";
const PROGRAM_DATA: &str = "Program data: ";


// Reference: https://github.com/raydium-io/raydium-clmm/blob/master/client/src/instructions/events_instructions_parse.rs#L181-L183
pub fn filter_latest_swap_event(
    logs: &[String],
) -> Option<SwapEvent> {
    let event = logs.iter().rev().find_map(|log: &String|
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
            if matches!(disc, SwapEvent::DISCRIMINATOR) {
                match decode_event(&mut slice) {
                    Ok(event) => {
                        return Some(event);
                    }
                    Err(e) => {
                        log::warn!("Could not decode event: {}, log {}", e, log);
                        return None;
                    }
                }
            } else {
                None
            }
        } else { None }
    );
    event
}

fn decode_event<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
    slice: &mut &[u8],
) -> anyhow::Result<T> {
    let event: T = anchor_lang::AnchorDeserialize::deserialize(slice)?;
    Ok(event)
}
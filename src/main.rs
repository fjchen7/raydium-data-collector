mod swap_event_fetcher;

use futures_util::StreamExt;
use std::str::FromStr;
use std::sync::Mutex;
use std::time::Duration;
use anchor_lang::Discriminator;
use raydium_amm_v3::states::SwapEvent;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::pubkey::Pubkey;
use anyhow::Error;
use tokio::time::Interval;
use log::log;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let ws_url = String::from(
    //     "wss://frequent-quiet-field.solana-mainnet.quiknode.pro/1e449a7aa48c912003ee07c443e5514d31cf1395",
    // );
    // let pool_address = "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj";
    // println!("{}", pool_address);
    // let pool_address = Pubkey::from_str(pool_address)?;
    //
    // let clmm_address = "devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH";
    // let clmm_address = Pubkey::from_str(clmm_address)?;
    //
    // let client = PubsubClient::new(&ws_url).await?;
    //
    // let config = RpcTransactionLogsConfig {
    //     commitment: Some(CommitmentConfig {
    //         commitment: CommitmentLevel::Confirmed,
    //     }),
    // };
    // let filter = RpcTransactionLogsFilter::Mentions(
    //     vec![
    //         "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj".to_string(),
    //         // "devi51mZmdwUJGU9hjN27vEz64Gps7uUefqxg27EAtH".to_string()
    //     ]
    // );
    // let (mut accounts, unsubscriber) = client
    //     .logs_subscribe(filter, config)
    //     .await?;

    // let (mut accounts, unsubscriber) = client.account_subscribe(&pool_address, None).await?;
    // let ws_url =
    //     "wss://frequent-quiet-field.solana-mainnet.quiknode.pro/1e449a7aa48c912003ee07c443e5514d31cf1395";
    // let pool_address = "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj";
    // let event_fetcher = swap_event_fetcher::SwapEventFetcher::connect(&ws_url, &pool_address).await.unwrap();
    // let (mut stream, unsubscriber) = event_fetcher
    //     .subscribe()
    //     .await.unwrap();
    // println!("Subscribed to swap_event event updates");
    // while let Some(response) = stream.next().await {
    //     for log in response.value.logs.iter() {
    //         let event = parse_swap_events(&log).unwrap();
    //         if let Some(event) = event {
    //             println!("{:#?}", event);
    //         }
    //     }
    // }
    // unsubscriber().await;
    let interval = Duration::from_secs(1);
    fetch_market_data_and_store_periodically(interval).await;
    Ok(())
}

pub async fn fetch_market_data_and_store_periodically(interval: Duration) {
    let mut interval = tokio::time::interval(interval);
    let mut latest_swap_event = None;
    if let Some(latest_swap_event) = latest_swap_event.take() {
        store_market_data(latest_swap_event);
    }
    let ws_url =
        "wss://frequent-quiet-field.solana-mainnet.quiknode.pro/1e449a7aa48c912003ee07c443e5514d31cf1395";
    let pool_address = "8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj";
    let event_fetcher = swap_event_fetcher::SwapEventFetcher::connect(&ws_url, &pool_address).await.unwrap();
    let (mut stream, unsubscriber) = event_fetcher
        .subscribe()
        .await.unwrap();
    println!("Subscribed to swap_event event updates");

    loop {
        tokio::select! {
        v = interval.tick() => {
            println!("tick next");
            if let Some(latest_swap_event) = latest_swap_event.take() {
                let now = time::OffsetDateTime::now_utc();
                let timestamp_sec = now.unix_timestamp();
                println!("Current timestamp in milliseconds: {}", timestamp_sec);
                store_market_data(latest_swap_event);
            }
        }
        Some(response) = stream.next() => {
            println!("stream next");
            let logs = response.value.logs;
            if let Some(event) = get_latest_swap_event(&logs) {
                latest_swap_event = Some(event);
                // store_market_data(latest_swap_event);
            }
            // for log in response.value.logs.iter() {
            //     let event = parse_swap_events(&log).unwrap();
            //     if let Some(event) = event {
            //         println!("{:#?}", event);
            // }
            // }
        }
        }
    }
    // unsubscriber().await?
}

pub fn store_market_data(event: SwapEvent) {
    println!("{:#?}", event);
    // todo!()
}


const PROGRAM_LOG: &str = "Program log: ";
const PROGRAM_DATA: &str = "Program data: ";


// Reference: https://github.com/raydium-io/raydium-clmm/blob/master/client/src/instructions/events_instructions_parse.rs#L181-L183
pub fn get_latest_swap_event(
    logs: &[String],
) -> Option<SwapEvent> {
    let event = logs.iter().rev().find_map(|log: &String|
        if let Some(log) = log.strip_prefix(PROGRAM_DATA) {
            let borsh_bytes = match anchor_lang::__private::base64::decode(log) {
                Ok(borsh_bytes) => borsh_bytes,
                _ => {
                    println!("Could not base64 decode log: {}", log);
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
                        println!("Could not decode event: {}, log {}", e, log);
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


pub fn parse_swap_events(
    l: &str,
    // ) -> Result<Option<SwapEvent>, Error> {
) -> anyhow::Result<Option<SwapEvent>> {
    // Log emitted from the current program.
    if let Some(log) = l.strip_prefix(PROGRAM_DATA) {
        let borsh_bytes = anchor_lang::__private::base64::decode(log)?;
        // let borsh_bytes = match anchor_lang::__private::base64::decode(log) {
        //     Ok(borsh_bytes) => borsh_bytes,
        //     _ => {
        //         panic!("Could not base64 decode log: {}", log);
        //     }
        // };
        let mut slice: &[u8] = &borsh_bytes[..];
        let disc: [u8; 8] = {
            let mut disc = [0; 8];
            disc.copy_from_slice(&borsh_bytes[..8]);
            slice = &slice[8..];
            disc
        };
        match disc {
            SwapEvent::DISCRIMINATOR => {
                let event = decode_event::<SwapEvent>(&mut slice)?;
                // println!("{:#?}", event);
                return Ok(Some(event));
            }
            _ => {
                // println!("unknow event: {}", l);
                return Ok(None);
            }
        }
        Ok(None)
    } else {
        Ok(None)
    }
    // if with_prefix {
    //      l.strip_prefix(PROGRAM_LOG)
    //          .or_else(|| l.strip_prefix(PROGRAM_DATA))
}

// pub fn handle_program_log(
//     self_program_str: &str,
//     l: &str,
//     with_prefix: bool,
// ) -> Result<(Option<String>, bool), ()> {
//     // Log emitted from the current program.
//     if let Some(log) = if with_prefix {
//         l.strip_prefix(PROGRAM_LOG)
//             .or_else(|| l.strip_prefix(PROGRAM_DATA))
//     } else {
//         Some(l)
//     }
//     {
//         if l.starts_with(&format!("Program log:")) {
//             // not log event
//             return Ok((None, false));
//         }
//         println!("----------");
//         let borsh_bytes = match anchor_lang::__private::base64::decode(log) {
//             Ok(borsh_bytes) => borsh_bytes,
//             _ => {
//                 println!("Could not base64 decode log: {}", log);
//                 return Ok((None, false));
//             }
//         };
//
//         let mut slice: &[u8] = &borsh_bytes[..];
//         let disc: [u8; 8] = {
//             let mut disc = [0; 8];
//             disc.copy_from_slice(&borsh_bytes[..8]);
//             slice = &slice[8..];
//             disc
//         };
//         match disc {
//             SwapEvent::DISCRIMINATOR => {
//                 let mut num = GLOBAL_VAR.lock().unwrap();
//                 *num += 1;
//                 println!("swapEvent");
//                 println!("{:#?}", decode_event::<SwapEvent>(&mut slice)?);
//             }
//             _ => {
//                 println!("unknow event: {}", l);
//             }
//         }
//         return Ok((None, false));
//     } else {
//         let (program, did_pop) = handle_system_log(self_program_str, l);
//         return Ok((program, did_pop));
//     }
// }

// fn handle_system_log(this_program_str: &str, log: &str) -> (Option<String>, bool) {
//     // println!("handle_system_log");
//     (None, true)
// }

fn decode_event<T: anchor_lang::Event + anchor_lang::AnchorDeserialize>(
    slice: &mut &[u8],
) -> anyhow::Result<T> {
    let event: T = anchor_lang::AnchorDeserialize::deserialize(slice)?;
    Ok(event)
}
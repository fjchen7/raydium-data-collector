use raydium_amm_v3::states::SwapEvent;

pub trait SwapEventHandler {
    fn handle_swap_event(&self, event: SwapEvent, timestamp: i64);
}


pub struct CsvSwapEventHandler {
    file: std::fs::File,
}

pub struct DummySwapEventHandler {}

impl SwapEventHandler for DummySwapEventHandler {
    fn handle_swap_event(&self, event: SwapEvent, timestamp: i64) {
        log::info!("timestamp: {}, event: {:#?}", timestamp, event);
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
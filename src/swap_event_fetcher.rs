use std::str::FromStr;
use anchor_lang::prelude::Pubkey;
use thiserror::Error;
use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::pubsub_client::PubsubClientError;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_client::rpc_response::RpcLogsResponse;
use solana_rpc_client_api::response::Response;

pub type SwapEventFetcherResult<T = ()> = Result<T, SwapEventFetcherError>;

#[derive(Debug, Error)]
pub enum SwapEventFetcherError {
    #[error("client error for subscription")]
    ClientError(#[from] PubsubClientError),
}

pub struct SwapEventFetcher {
    pub client: PubsubClient,
    pub pool_address: Pubkey,
}

type UnsubscribeFn = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>;
type LogBoxStream<'a> = BoxStream<'a, Response<RpcLogsResponse>>;

impl SwapEventFetcher {
    pub async fn connect(client: &str, pool_address: &str) -> SwapEventFetcherResult<Self> {
        let client = PubsubClient::new(client).await.map_err(SwapEventFetcherError::ClientError)?;
        Ok(Self {
            client,
            pool_address: Pubkey::from_str(pool_address).unwrap(),
        })
    }

    pub async fn subscribe(&self) -> SwapEventFetcherResult<(LogBoxStream<'_>, UnsubscribeFn)>
    {
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        };
        let filter = RpcTransactionLogsFilter::Mentions(
            vec![self.pool_address.to_string(), ]
        );
        let (stream, unsubscriber) = self.client
            .logs_subscribe(filter, config)
            .await.map_err(SwapEventFetcherError::ClientError)?;
        Ok((stream, unsubscriber))
    }
}


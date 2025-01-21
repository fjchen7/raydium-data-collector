use anchor_lang::prelude::Pubkey;
use futures_util::future::BoxFuture;
use futures_util::stream::BoxStream;
use solana_client::nonblocking::pubsub_client::PubsubClient;
use solana_client::pubsub_client::PubsubClientError;
use solana_client::rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter};
use solana_client::rpc_response::RpcLogsResponse;
use solana_rpc_client_api::response::Response;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use std::str::FromStr;
use thiserror::Error;

pub type EventFetcherResult<T = ()> = Result<T, EventFetcherError>;

#[derive(Debug, Error)]
pub enum EventFetcherError {
    #[error("client error for subscription")]
    ClientError(#[from] PubsubClientError),
}

pub struct EventFetcher {
    pub client: PubsubClient,
    pub pool_address: Pubkey,
}

type UnsubscribeFn = Box<dyn FnOnce() -> BoxFuture<'static, ()> + Send>;
type LogBoxStream<'a> = BoxStream<'a, Response<RpcLogsResponse>>;

impl EventFetcher {
    pub async fn connect(client: &str, pool_address: &str) -> EventFetcherResult<Self> {
        let client = PubsubClient::new(client)
            .await
            .map_err(EventFetcherError::ClientError)?;
        Ok(Self {
            client,
            pool_address: Pubkey::from_str(pool_address).unwrap(),
        })
    }

    pub async fn subscribe(&self) -> EventFetcherResult<(LogBoxStream<'_>, UnsubscribeFn)> {
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig {
                commitment: CommitmentLevel::Confirmed,
            }),
        };
        let filter = RpcTransactionLogsFilter::Mentions(vec![self.pool_address.to_string()]);
        let (stream, unsubscriber) = self
            .client
            .logs_subscribe(filter, config)
            .await
            .map_err(EventFetcherError::ClientError)?;
        Ok((stream, unsubscriber))
    }
}

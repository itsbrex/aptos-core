// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

use crate::grpc_manager::GrpcManager;
use anyhow::Result;
use aptos_indexer_grpc_server_framework::RunnableConfig;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::OnceCell;

static GRPC_MANAGER: OnceCell<GrpcManager> = OnceCell::const_new();

pub(crate) type GrpcAddress = String;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ServiceConfig {
    pub(crate) listen_address: SocketAddr,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IndexerGrpcManagerConfig {
    pub(crate) chain_id: u64,
    pub(crate) service_config: ServiceConfig,
    pub(crate) self_advertised_address: GrpcAddress,
    pub(crate) grpc_manager_addresses: Vec<GrpcAddress>,
    pub(crate) fullnode_addresses: Vec<GrpcAddress>,
}

#[async_trait::async_trait]
impl RunnableConfig for IndexerGrpcManagerConfig {
    async fn run(&self) -> Result<()> {
        GRPC_MANAGER
            .get_or_init(|| async { GrpcManager::new(self).await })
            .await
            .start(&self.service_config)
    }

    fn get_server_name(&self) -> String {
        "grpc_manager".to_string()
    }
}

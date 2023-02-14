// Copyright (c) Aptos
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

use crate::{
    metrics, PeerMonitoringServiceNetworkEvents, PeerMonitoringServiceServer,
    PEER_MONITORING_SERVER_VERSION,
};
use aptos_channels::{aptos_channel, message_queues::QueueStyle};
use aptos_config::{
    config::{BaseConfig, NodeConfig, PeerMonitoringServiceConfig},
    network_id::NetworkId,
};
use aptos_logger::Level;
use aptos_network::{
    application::{interface::NetworkServiceEvents, storage::PeersAndMetadata},
    peer_manager::PeerManagerNotification,
    protocols::{
        network::{NetworkEvents, NewNetworkEvents},
        rpc::InboundRpcRequest,
        wire::handshake::v1::ProtocolId,
    },
};
use aptos_peer_monitoring_service_types::{
    LatencyPingRequest, PeerMonitoringServiceError, PeerMonitoringServiceMessage,
    PeerMonitoringServiceRequest, PeerMonitoringServiceResponse, ServerProtocolVersionResponse,
};
use aptos_time_service::{MockTimeService, TimeService};
use aptos_types::PeerId;
use claims::assert_matches;
use futures::channel::oneshot;
use rand::{rngs::OsRng, Rng};
use std::{collections::HashMap, sync::Arc};

#[tokio::test]
async fn test_get_server_protocol_version() {
    // Create the peer monitoring client and server
    let (mut mock_client, service, _, _) = MockClient::new(None, None);
    tokio::spawn(service.start());

    // Process a request to fetch the protocol version
    let request = PeerMonitoringServiceRequest::GetServerProtocolVersion;
    let response = mock_client.send_request(request).await.unwrap();

    // Verify the response is correct
    let expected_response =
        PeerMonitoringServiceResponse::ServerProtocolVersion(ServerProtocolVersionResponse {
            version: PEER_MONITORING_SERVER_VERSION,
        });
    assert_eq!(response, expected_response);
}

#[tokio::test]
async fn test_latency_ping_request() {
    // Create the peer monitoring client and server
    let (mut mock_client, service, _, _) = MockClient::new(None, None);
    tokio::spawn(service.start());

    // Process several requests to perform latency pings
    for i in 0..10 {
        let request =
            PeerMonitoringServiceRequest::LatencyPing(LatencyPingRequest { ping_counter: i });
        let response = mock_client.send_request(request).await.unwrap();
        match response {
            PeerMonitoringServiceResponse::LatencyPing(latecy_ping_response) => {
                assert_eq!(latecy_ping_response.ping_counter, i);
            },
            _ => panic!("Expected latency ping response but got: {:?}", response),
        }
    }
}

#[tokio::test]
async fn test_unsupported() {
    // Create the peer monitoring client and server
    let (mut mock_client, service, _, _) = MockClient::new(None, None);
    tokio::spawn(service.start());

    // Process a request to fetch the network information
    let request = PeerMonitoringServiceRequest::GetNetworkInformation;
    let response = mock_client.send_request(request).await.unwrap_err();

    // Verify an error is returned
    assert_matches!(response, PeerMonitoringServiceError::InvalidRequest(_));
}

// A wrapper around the inbound network interface/channel for easily sending
/// mock client requests to a peer monitoring service server.
struct MockClient {
    peer_manager_notifiers:
        HashMap<NetworkId, aptos_channel::Sender<(PeerId, ProtocolId), PeerManagerNotification>>,
}

impl MockClient {
    fn new(
        base_config: Option<BaseConfig>,
        peer_monitoring_config: Option<PeerMonitoringServiceConfig>,
    ) -> (
        Self,
        PeerMonitoringServiceServer,
        MockTimeService,
        Arc<PeersAndMetadata>,
    ) {
        initialize_logger();

        // Create the node config
        let base_config = base_config.unwrap_or_default();
        let peer_monitoring_config = peer_monitoring_config.unwrap_or_default();
        let node_config = NodeConfig {
            base: base_config,
            peer_monitoring_service: peer_monitoring_config.clone(),
            ..Default::default()
        };

        // Setup the networks and the network events
        let network_ids = vec![NetworkId::Validator, NetworkId::Vfn, NetworkId::Public];
        let peers_and_metadata = PeersAndMetadata::new(&network_ids);
        let mut network_and_events = HashMap::new();
        let mut peer_manager_notifiers = HashMap::new();
        for network_id in network_ids {
            let queue_cfg = aptos_channel::Config::new(
                peer_monitoring_config.max_network_channel_size as usize,
            )
            .queue_style(QueueStyle::FIFO)
            .counters(&metrics::PENDING_PEER_MONITORING_SERVER_NETWORK_EVENTS);
            let (peer_manager_notifier, peer_manager_notification_receiver) = queue_cfg.build();
            let (_, connection_notification_receiver) = queue_cfg.build();

            let network_events = NetworkEvents::new(
                peer_manager_notification_receiver,
                connection_notification_receiver,
            );
            network_and_events.insert(network_id, network_events);
            peer_manager_notifiers.insert(network_id, peer_manager_notifier);
        }
        let peer_monitoring_network_events =
            PeerMonitoringServiceNetworkEvents::new(NetworkServiceEvents::new(network_and_events));

        // Create the storage service
        let executor = tokio::runtime::Handle::current();
        let mock_time_service = TimeService::mock();
        let peer_monitoring_server = PeerMonitoringServiceServer::new(
            node_config,
            executor,
            peer_monitoring_network_events,
            peers_and_metadata.clone(),
        );

        // Create the client
        let mock_client = Self {
            peer_manager_notifiers,
        };

        (
            mock_client,
            peer_monitoring_server,
            mock_time_service.into_mock(),
            peers_and_metadata,
        )
    }

    /// Sends the specified request and returns the response from the server
    async fn send_request(
        &mut self,
        request: PeerMonitoringServiceRequest,
    ) -> Result<PeerMonitoringServiceResponse, PeerMonitoringServiceError> {
        let peer_id = PeerId::random();
        let protocol_id = ProtocolId::PeerMonitoringServiceRpc;
        let network_id = get_random_network_id();

        // Create an inbound RPC request
        let request_data = protocol_id
            .to_bytes(&PeerMonitoringServiceMessage::Request(request))
            .unwrap();
        let (request_sender, request_receiver) = oneshot::channel();
        let inbound_rpc = InboundRpcRequest {
            protocol_id,
            data: request_data.into(),
            res_tx: request_sender,
        };
        let request_notification = PeerManagerNotification::RecvRpc(peer_id, inbound_rpc);

        // Send the request to the peer monitoring service
        self.peer_manager_notifiers
            .get(&network_id)
            .unwrap()
            .push((peer_id, protocol_id), request_notification)
            .unwrap();

        // Wait for the response from the peer monitoring service
        let response_data = request_receiver.await.unwrap().unwrap();
        let response = protocol_id
            .from_bytes::<PeerMonitoringServiceMessage>(&response_data)
            .unwrap();
        match response {
            PeerMonitoringServiceMessage::Response(response) => response,
            _ => panic!("Unexpected response message: {:?}", response),
        }
    }
}

/// Initializes the Aptos logger for tests
pub fn initialize_logger() {
    aptos_logger::Logger::builder()
        .is_async(false)
        .level(Level::Debug)
        .build();
}

/// Returns a random network ID
fn get_random_network_id() -> NetworkId {
    let mut rng = OsRng;
    let random_number: u8 = rng.gen();
    match random_number % 3 {
        0 => NetworkId::Validator,
        1 => NetworkId::Vfn,
        2 => NetworkId::Public,
        num => panic!("This shouldn't be possible! Got num: {:?}", num),
    }
}

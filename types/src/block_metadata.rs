// Copyright © Aptos Labs
// SPDX-License-Identifier: Apache-2.0

use aptos_crypto::HashValue;
use move_core_types::{account_address::AccountAddress, value::MoveValue};
use serde::{Deserialize, Serialize};

/// Struct that will be persisted on chain to store the information of the current block.
///
/// The flow will look like following:
/// 1. The executor will pass this struct to VM at the end of a block proposal.
/// 2. The VM will use this struct to create a special system transaction that will emit an event
///    represents the information of the current block. This transaction can't
///    be emitted by regular users and is generated by each of the validators on the fly. Such
///    transaction will be executed before all of the user-submitted transactions in the blocks.
/// 3. Once that special resource is modified, the other user transactions can read the consensus
///    info by calling into the read method of that resource, which would thus give users the
///    information such as the current leader.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockMetadata {
    id: HashValue,
    epoch: u64,
    round: u64,
    proposer: AccountAddress,
    #[serde(with = "serde_bytes")]
    previous_block_votes_bitvec: Vec<u8>,
    failed_proposer_indices: Vec<u32>,
    timestamp_usecs: u64,
}

impl BlockMetadata {
    pub fn new(
        id: HashValue,
        epoch: u64,
        round: u64,
        proposer: AccountAddress,
        previous_block_votes_bitvec: Vec<u8>,
        failed_proposer_indices: Vec<u32>,
        timestamp_usecs: u64,
    ) -> Self {
        Self {
            id,
            epoch,
            round,
            proposer,
            previous_block_votes_bitvec,
            failed_proposer_indices,
            timestamp_usecs,
        }
    }

    pub fn id(&self) -> HashValue {
        self.id
    }

    pub fn get_prologue_move_args(self, signer: AccountAddress) -> Vec<MoveValue> {
        vec![
            MoveValue::Signer(signer),
            MoveValue::Address(AccountAddress::from_bytes(self.id.to_vec()).unwrap()),
            MoveValue::U64(self.epoch),
            MoveValue::U64(self.round),
            MoveValue::Address(self.proposer),
            MoveValue::Vector(
                self.failed_proposer_indices
                    .into_iter()
                    .map(u64::from)
                    .map(MoveValue::U64)
                    .collect(),
            ),
            MoveValue::Vector(
                self.previous_block_votes_bitvec
                    .into_iter()
                    .map(MoveValue::U8)
                    .collect(),
            ),
            MoveValue::U64(self.timestamp_usecs),
        ]
    }

    pub fn timestamp_usecs(&self) -> u64 {
        self.timestamp_usecs
    }

    pub fn proposer(&self) -> AccountAddress {
        self.proposer
    }

    pub fn previous_block_votes_bitvec(&self) -> &Vec<u8> {
        &self.previous_block_votes_bitvec
    }

    pub fn failed_proposer_indices(&self) -> &Vec<u32> {
        &self.failed_proposer_indices
    }

    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn round(&self) -> u64 {
        self.round
    }
}

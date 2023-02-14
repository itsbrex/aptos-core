// Copyright © Aptos Labs
// SPDX-License-Identifier: Apache-2.0

use super::*;
use aptos_schemadb::{schema::fuzzing::assert_encode_decode, test_no_panic_decoding};
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_encode_decode(txn in any::<Transaction>()) {
        assert_encode_decode::<TransactionSchema>(&0u64, &txn);
    }
}

test_no_panic_decoding!(TransactionSchema);

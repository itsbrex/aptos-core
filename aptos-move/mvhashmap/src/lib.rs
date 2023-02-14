// Copyright © Aptos Labs
// SPDX-License-Identifier: Apache-2.0

use aptos_aggregator::{delta_change_set::DeltaOp, transaction::AggregatorValue};
use aptos_infallible::Mutex;
use aptos_types::write_set::TransactionWrite;
use crossbeam::utils::CachePadded;
use dashmap::DashMap;
use std::{
    collections::btree_map::BTreeMap,
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

#[cfg(test)]
mod unit_tests;

// TODO: re-use definitions with the scheduler.
pub type TxnIndex = usize;
pub type Incarnation = usize;
pub type Version = (TxnIndex, Incarnation);

const FLAG_DONE: usize = 0;
const FLAG_ESTIMATE: usize = 1;

/// Every entry in shared multi-version data-structure has an "estimate" flag
/// and some content.
pub struct Entry<V> {
    /// Used to mark the entry as a "write estimate".
    flag: AtomicUsize,
    /// Actual content.
    pub cell: EntryCell<V>,
}

/// Represents the content of a single entry in multi-version data-structure.
pub enum EntryCell<V> {
    /// Recorded in the shared multi-version data-structure for each write. It
    /// has: 1) Incarnation number of the transaction that wrote the entry (note
    /// that TxnIndex is part of the key and not recorded here), 2) actual data
    /// stored in a shared pointer (to ensure ownership and avoid clones).
    Write(Incarnation, Arc<V>),
    /// Recorded in the shared multi-version data-structure for each delta.
    Delta(DeltaOp),
}

impl<V> Entry<V> {
    pub fn new_write_from(flag: usize, incarnation: Incarnation, data: V) -> Entry<V> {
        Entry {
            flag: AtomicUsize::new(flag),
            cell: EntryCell::Write(incarnation, Arc::new(data)),
        }
    }

    pub fn new_delta_from(flag: usize, data: DeltaOp) -> Entry<V> {
        Entry {
            flag: AtomicUsize::new(flag),
            cell: EntryCell::Delta(data),
        }
    }

    pub fn flag(&self) -> usize {
        self.flag.load(Ordering::SeqCst)
    }

    pub fn mark_estimate(&self) {
        self.flag.store(FLAG_ESTIMATE, Ordering::SeqCst);
    }
}

pub(crate) struct VersionedValue<V> {
    pub(crate) versioned_map: BTreeMap<TxnIndex, CachePadded<Entry<V>>>,
    // Note: this can cache base (storage) value in Option<u128> to facilitate
    // aggregator validation & reading in the future, if needed.
    pub(crate) contains_delta: bool,
}

impl<V: TransactionWrite> VersionedValue<V> {
    pub fn new() -> Self {
        Self {
            versioned_map: BTreeMap::new(),
            contains_delta: false,
        }
    }
}

impl<V: TransactionWrite> Default for VersionedValue<V> {
    fn default() -> Self {
        VersionedValue::new()
    }
}

/// Main multi-version data-structure used by threads to read/write during parallel
/// execution. Maps each access path to an interal BTreeMap that contains the indices
/// of transactions that write at the given access path alongside the corresponding
/// entries of WriteCell type.
///
/// Concurrency is managed by DashMap, i.e. when a method accesses a BTreeMap at a
/// given key, it holds exclusive access and doesn't need to explicitly synchronize
/// with other reader/writers.
pub struct MVHashMap<K, V> {
    data: DashMap<K, VersionedValue<V>>,
    delta_keys: Mutex<Vec<K>>,
}

/// Returned as Err(..) when failed to read from the multi-version data-structure.
#[derive(Debug, PartialEq, Eq)]
pub enum MVHashMapError {
    /// No prior entry is found.
    NotFound,
    /// Read resulted in an unresolved delta value.
    Unresolved(DeltaOp),
    /// A dependency on other transaction has been found during the read.
    Dependency(TxnIndex),
    /// Delta application failed, txn execution should fail.
    DeltaApplicationFailure,
}

/// Returned as Ok(..) when read successfully from the multi-version data-structure.
#[derive(Debug, PartialEq, Eq)]
pub enum MVHashMapOutput<V> {
    /// Result of resolved delta op, always u128. Unlike with `Version`, we return
    /// actual data because u128 is cheap to copy amd validation can be done correctly
    /// on values as well (ABA is not a problem).
    Resolved(u128),
    /// Information from the last versioned-write. Note that the version is returned
    /// and not the data to avoid passing big values around.
    Version(Version, Arc<V>),
}

impl<K: Hash + Clone + Eq, V: TransactionWrite> MVHashMap<K, V> {
    pub fn new() -> MVHashMap<K, V> {
        MVHashMap {
            data: DashMap::new(),
            delta_keys: Mutex::new(Vec::new()),
        }
    }

    /// For processing outputs - removes the BTreeMap from the MVHashMap.
    pub fn entry_map_for_key(&self, key: &K) -> Option<BTreeMap<TxnIndex, CachePadded<Entry<V>>>> {
        self.data
            .remove(key)
            .map(|(_, v)| v)
            .map(|p| p.versioned_map)
    }

    /// Returns the list of keys that had an associated delta entry at any prior point.
    pub fn aggregator_keys(&self) -> Vec<K> {
        std::mem::take(&mut self.delta_keys.lock())
    }

    /// Add a write of versioned data at a specified key. If the entry is overwritten, asserts
    /// that the new incarnation is strictly higher.
    pub fn add_write(&self, key: &K, version: Version, data: V) {
        let (txn_idx, incarnation) = version;

        let mut v = self.data.entry(key.clone()).or_default();
        let prev_entry = v.versioned_map.insert(
            txn_idx,
            CachePadded::new(Entry::new_write_from(FLAG_DONE, incarnation, data)),
        );

        // Assert that the previous entry for txn_idx, if present, had lower incarnation.
        assert!(prev_entry.map_or(true, |entry| -> bool {
            if let EntryCell::Write(i, _) = entry.cell {
                i < incarnation
            } else {
                true
            }
        }));
    }

    /// Add a delta at a specified key.
    pub fn add_delta(&self, key: &K, txn_idx: usize, delta: DeltaOp) {
        let mut v = self.data.entry(key.clone()).or_default();
        v.versioned_map.insert(
            txn_idx,
            CachePadded::new(Entry::new_delta_from(FLAG_DONE, delta)),
        );

        if !v.contains_delta {
            v.contains_delta = true;
            self.delta_keys.lock().push(key.clone());
        }
    }

    /// Mark an entry from transaction 'txn_idx' at access path 'key' as an estimated write
    /// (for future incarnation). Will panic if the entry is not in the data-structure.
    pub fn mark_estimate(&self, key: &K, txn_idx: TxnIndex) {
        let v = self.data.get(key).expect("Path must exist");
        v.versioned_map
            .get(&txn_idx)
            .expect("Entry by txn must exist")
            .mark_estimate();
    }

    /// Delete an entry from transaction 'txn_idx' at access path 'key'. Will panic
    /// if the access path has never been written before.
    pub fn delete(&self, key: &K, txn_idx: TxnIndex) {
        // TODO: investigate logical deletion.
        let mut v = self.data.get_mut(key).expect("Path must exist");
        v.versioned_map.remove(&txn_idx);
    }

    /// Read entry from transaction 'txn_idx' at access path 'key'.
    pub fn read(
        &self,
        key: &K,
        txn_idx: TxnIndex,
    ) -> anyhow::Result<MVHashMapOutput<V>, MVHashMapError> {
        use MVHashMapError::*;
        use MVHashMapOutput::*;

        match self.data.get(key) {
            Some(v) => {
                let mut iter = v.versioned_map.range(0..txn_idx);

                // If read encounters a delta, it must traverse the block of transactions
                // (top-down) until it encounters a write or reaches the end of the block.
                // During traversal, all aggregator deltas have to be accumulated together.
                let mut accumulator: Option<Result<DeltaOp, ()>> = None;
                while let Some((idx, entry)) = iter.next_back() {
                    let flag = entry.flag();

                    if flag == FLAG_ESTIMATE {
                        // Found a dependency.
                        return Err(Dependency(*idx));
                    }

                    // The entry should be populated.
                    debug_assert!(flag == FLAG_DONE);

                    match (&entry.cell, accumulator.as_mut()) {
                        (EntryCell::Write(incarnation, data), None) => {
                            // Resolve to the write if no deltas were applied in between.
                            let write_version = (*idx, *incarnation);
                            return Ok(Version(write_version, data.clone()));
                        },
                        (EntryCell::Write(incarnation, data), Some(accumulator)) => {
                            // Deltas were applied. We must deserialize the value
                            // of the write and apply the aggregated delta accumulator.

                            // None if data represents deletion. Otherwise, panics if the
                            // data can't be resolved to an aggregator value.
                            let maybe_value = AggregatorValue::from_write(data.as_ref());

                            if maybe_value.is_none() {
                                // Resolve to the write if the WriteOp was deletion
                                // (MoveVM will observe 'deletion'). This takes precedence
                                // over any speculative delta accumulation errors on top.
                                let write_version = (*idx, *incarnation);
                                return Ok(Version(write_version, data.clone()));
                            }
                            return accumulator.map_err(|_| DeltaApplicationFailure).and_then(
                                |a| {
                                    // Apply accumulated delta to resolve the aggregator value.
                                    a.apply_to(maybe_value.unwrap().into())
                                        .map(|result| Resolved(result))
                                        .map_err(|_| DeltaApplicationFailure)
                                },
                            );
                        },
                        (EntryCell::Delta(delta), Some(accumulator)) => {
                            *accumulator = accumulator.and_then(|mut a| {
                                // Read hit a delta during traversing the block and aggregating
                                // other deltas. Merge two deltas together. If Delta application
                                // fails, we record an error, but continue processing (to e.g.
                                // account for the case when the aggregator was deleted).
                                if a.merge_onto(*delta).is_err() {
                                    Err(())
                                } else {
                                    Ok(a)
                                }
                            });
                        },
                        (EntryCell::Delta(delta), None) => {
                            // Read hit a delta and must start accumulating.
                            // Initialize the accumulator and continue traversal.
                            accumulator = Some(Ok(*delta))
                        },
                    }
                }

                // It can happen that while traversing the block and resolving
                // deltas the actual written value has not been seen yet (i.e.
                // it is not added as an entry to the data-structure).
                match accumulator {
                    Some(Ok(accumulator)) => Err(Unresolved(accumulator)),
                    Some(Err(_)) => Err(DeltaApplicationFailure),
                    None => Err(NotFound),
                }
            },
            None => Err(NotFound),
        }
    }
}

impl<K: Hash + Clone + Eq, V: TransactionWrite> Default for MVHashMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

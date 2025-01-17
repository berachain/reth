//! Compatibility functions for rpc `Transaction` type.

use std::fmt;

use alloy_consensus::Transaction as _;
use alloy_rpc_types::{
    request::{TransactionInput, TransactionRequest},
    TransactionInfo,
};
use reth_primitives::TransactionSignedEcRecovered;
use serde::{Deserialize, Serialize};

/// Create a new rpc transaction result for a mined transaction, using the given block hash,
/// number, and tx index fields to populate the corresponding fields in the rpc result.
///
/// The block hash, number, and tx index fields should be from the original block where the
/// transaction was mined.
pub fn from_recovered_with_block_context<T: TransactionCompat>(
    tx: TransactionSignedEcRecovered,
    tx_info: TransactionInfo,
    resp_builder: &T,
) -> T::Transaction {
    resp_builder.fill(tx, tx_info)
}

/// Create a new rpc transaction result for a _pending_ signed transaction, setting block
/// environment related fields to `None`.
pub fn from_recovered<T: TransactionCompat>(
    tx: TransactionSignedEcRecovered,
    resp_builder: &T,
) -> T::Transaction {
    resp_builder.fill(tx, TransactionInfo::default())
}

/// Builds RPC transaction w.r.t. network.
pub trait TransactionCompat: Send + Sync + Unpin + Clone + fmt::Debug {
    /// RPC transaction response type.
    type Transaction: Serialize
        + for<'de> Deserialize<'de>
        + Send
        + Sync
        + Unpin
        + Clone
        + fmt::Debug;

    /// Create a new rpc transaction result for a _pending_ signed transaction, setting block
    /// environment related fields to `None`.
    fn fill(&self, tx: TransactionSignedEcRecovered, tx_inf: TransactionInfo) -> Self::Transaction;

    /// Truncates the input of a transaction to only the first 4 bytes.
    // todo: remove in favour of using constructor on `TransactionResponse` or similar
    // <https://github.com/alloy-rs/alloy/issues/1315>.
    fn otterscan_api_truncate_input(tx: &mut Self::Transaction);
}

/// Convert [`TransactionSignedEcRecovered`] to [`TransactionRequest`]
pub fn transaction_to_call_request(tx: TransactionSignedEcRecovered) -> TransactionRequest {
    let from = tx.signer();
    let to = Some(tx.transaction.to().into());
    let gas = tx.transaction.gas_limit();
    let value = tx.transaction.value();
    let input = tx.transaction.input().clone();
    let nonce = tx.transaction.nonce();
    let chain_id = tx.transaction.chain_id();
    let access_list = tx.transaction.access_list().cloned();
    let max_fee_per_blob_gas = tx.transaction.max_fee_per_blob_gas();
    let authorization_list = tx.transaction.authorization_list().map(|l| l.to_vec());
    let blob_versioned_hashes = tx.transaction.blob_versioned_hashes();
    let tx_type = tx.transaction.tx_type();

    // fees depending on the transaction type
    let (gas_price, max_fee_per_gas) = if tx.is_dynamic_fee() {
        (None, Some(tx.max_fee_per_gas()))
    } else {
        (Some(tx.max_fee_per_gas()), None)
    };
    let max_priority_fee_per_gas = tx.transaction.max_priority_fee_per_gas();

    TransactionRequest {
        from: Some(from),
        to,
        gas_price,
        max_fee_per_gas,
        max_priority_fee_per_gas,
        gas: Some(gas),
        value: Some(value),
        input: TransactionInput::new(input),
        nonce: Some(nonce),
        chain_id,
        access_list,
        max_fee_per_blob_gas,
        blob_versioned_hashes,
        transaction_type: Some(tx_type.into()),
        sidecar: None,
        authorization_list,
    }
}

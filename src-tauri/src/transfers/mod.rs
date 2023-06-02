//! # Offline transfers
//! This module contains the functions for creating an offline transfer of tokens.
//! This is done by emptying the input dbcs, and creating new dbcs to the recipients
//! (and a change dbc if any) containing the transferred tokens.
//!
//! The transfer is created by selecting from the available input dbcs, and creating the necessary
//! spends to do so. The input dbcs are selected by the user, and the spends are created by this
//! module. The user can select the input dbcs by specifying the amount of tokens they want to
//! transfer, and the module will select the necessary dbcs to transfer that amount. The user can
//! also specify the amount of tokens they want to transfer to each recipient, and the module will
//! select the necessary dbcs to transfer that amount to each recipient.
//!
//! # Online transfers
//! When a transfer is created, it is not yet registered in the network, i.e. the emptied dbcs are
//! not yet rendered spent. The signed spends of the transfer are found in the new dbcs, and must be
//! uploaded to the network for the transfer to take effect.
//! The peers will validate each signed spend they receive, before accepting it.
//! Once enough peers have accepted all the spends of the transaction, and serve them upon request,
//! the transfer is completed and globally recognised.
//!
//! # On the difference between a transfer and a transaction.
//! The difference is subtle, but very much there. A transfer is a higher level concept, it is the
//! sending of tokens from one address to another. Or many.
//! A dbc transaction is the lower layer concept where the blinded inputs and outputs are specified.

mod error;
mod transfer;

pub(crate) use self::{
    error::{Error, Result},
    transfer::create_transfer,
};

use sn_dbc::{
    Dbc, DbcIdSource, DbcTransaction, DerivedKey, PublicAddress, RevealedAmount, SignedSpend, Token,
};

/// The input details necessary to
/// carry out a transfer of tokens.
#[derive(Debug)]
pub struct Inputs {
    /// The selected dbcs to spend, with the necessary amounts contained
    /// to transfer the below specified amount of tokens to each recipients.
    pub dbcs_to_spend: Vec<(Dbc, DerivedKey)>,
    /// The amounts and dbc ids for the dbcs that will be created to hold the transferred tokens.
    pub recipients: Vec<(Token, DbcIdSource)>,
    /// Any surplus amount after spending the necessary input dbcs.
    pub change: (Token, PublicAddress),
}

/// The created dbcs and change dbc from a transfer
/// of tokens from one or more dbcs, into one or more new dbcs.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Outputs {
    /// This is the hash of the transaction
    /// where all the below spends were made
    /// and dbcs created.
    pub tx_hash: sn_dbc::Hash,
    /// The dbcs that were created containing
    /// the tokens sent to respective recipient.
    pub created_dbcs: Vec<CreatedDbc>,
    /// The dbc holding surplus tokens after
    /// spending the necessary input dbcs.
    pub change_dbc: Option<Dbc>,
    /// The parameters necessary to send all spend requests to the network.
    pub all_spend_requests: Vec<SpendRequest>,
}

/// The parameters necessary to send a spend request to the network.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpendRequest {
    /// The dbc to register in the network as spent.
    pub signed_spend: SignedSpend,
    /// The dbc transaction that the spent dbc was created in.
    pub parent_tx: DbcTransaction,
}

/// A resulting dbc from a token transfer.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CreatedDbc {
    /// The dbc that was created.
    pub dbc: Dbc,
    /// This is useful for the sender to know how much they sent to each recipient.
    /// They can't know this from the dbc itself, as the amount is encrypted.
    pub amount: RevealedAmount,
}

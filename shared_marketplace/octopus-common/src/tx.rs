use serde::{Deserialize, Serialize};

/// A transaction type. Transactions should be able to rebuild a ledger's state
/// when they are applied in the same sequence to an empty state.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub enum Tx {
    /// Currency was added to the account
    Deposit { account: String, amount: u64 },

    /// Currency was withdrawn from the account
    Withdraw { account: String, amount: u64 },
}

/// A transaction type. Transactions should be able to rebuild a ledger's state
/// when they are applied in the same sequence to an empty state.
#[derive(Debug)]
pub enum Tx {
    // Add variants for storing withdraw/deposit transactions
    Deposit { account: String, amount: u64 },
    Withdraw { account: String, amount: u64 },
}

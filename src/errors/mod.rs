//An application specific type of error
#[derive(Debug)]
pub enum AccountingError {
    // Add variants here for account not found, account underfunded and account overfunded
    AccountNotFound(String),
    AccountUnderFunded(String, u64),
    AccountOverFunded(String, u64),
}

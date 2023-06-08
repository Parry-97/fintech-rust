use crate::{errors::AccountingError, tx::Tx};
use std::collections::HashMap;

/// A type for managing accounts and their current currency balance
#[derive(Debug)]
pub struct Accounts {
    // Add a property `accounts` here
    accounts: HashMap<String, u64>,
}
impl Accounts {
    /// Returns an empty instance of the [`Accounts`] type
    pub fn new() -> Self {
        Accounts {
            accounts: Default::default(),
        }
    }

    /// Either deposits the `amount` provided into the `signer` account or adds the amount to the existing account.
    /// # Errors
    /// Attempted overflow
    pub fn deposit(&mut self, signer: &str, amount: u64) -> Result<Tx, AccountingError> {
        if let Some(account) = self.accounts.get_mut(signer) {
            (*account)
                .checked_add(amount)
                .and_then(|r| {
                    *account = r;
                    Some(r)
                })
                .ok_or(AccountingError::AccountOverFunded(
                    signer.to_string(),
                    amount,
                ))
                // Using map() here is an easy way to only manipulate the non-error result
                .map(|_| Tx::Deposit {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            self.accounts.insert(signer.to_string(), amount);
            Ok(Tx::Deposit {
                account: signer.to_string(),
                amount,
            })
        }
    }

    /// Withdraws the `amount` from the `signer` account.
    /// # Errors
    /// Attempted overflow
    pub fn withdraw(&mut self, signer: &str, amount: u64) -> Result<Tx, AccountingError> {
        if let Some(balance) = self.accounts.get_mut(signer) {
            balance
                .checked_sub(amount)
                .and_then(|r| {
                    *balance = r;
                    Some(r)
                })
                .ok_or_else(|| {
                    AccountingError::AccountUnderFunded(
                        format!("Underfunded account for {}", signer.to_string()),
                        amount,
                    )
                })
                .map(|_| Tx::Withdraw {
                    account: signer.to_string(),
                    amount,
                })
        } else {
            return Err(AccountingError::AccountNotFound(format!(
                "Account not found for {}",
                signer.to_string()
            )));
        }
    }

    /// Withdraws the amount from the sender account and deposits it in the recipient account.
    ///
    /// # Errors
    /// The account doesn't exist
    pub fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(Tx, Tx), AccountingError> {
        let withdrawal = self.withdraw(sender, amount)?;
        let deposit = self.deposit(recipient, amount)?;

        Ok((withdrawal, deposit))
    }
}

#[cfg(test)]
mod tests {

    use crate::tx::Tx;

    use super::Accounts;

    #[test]
    fn test_missing_account() {
        let mut ledger = Accounts::new();
        let result = ledger.withdraw("non existing", 20);
        assert!(result.is_err());
    }

    #[test]
    fn test_over_funded_account() {
        let mut ledger = Accounts::new();
        let deposit = ledger.deposit("new account", 20);
        assert!(deposit.is_ok());
        let deposit = ledger.deposit("new account", u64::MAX);
        assert!(deposit.is_err());
    }

    #[test]
    fn test_underfunded_account() {
        let mut ledger = Accounts::new();
        let deposit = ledger.deposit("new account", 20);
        assert!(deposit.is_ok());
        let withdraw = ledger.withdraw("new account", 21);
        assert!(withdraw.is_err());
    }

    #[test]
    fn test_transaction_type_is_correct() {
        let mut ledger = Accounts::new();
        let result = ledger.deposit("new account", 20);
        assert!(matches!(
            result.unwrap(),
            Tx::Deposit {
                account: _,
                amount: _
            }
        ));
    }
}

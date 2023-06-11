use std::{cmp::Ordering, collections::HashMap};

use crate::{
    accounting::Accounts,
    core::{MatchingEngine, Order, PartialOrder, Receipt, Side},
    errors::ApplicationError,
    tx::Tx,
};

/// The core of the core: the [`TradingPlatform`]. Manages accounts, validates-, and orchestrates the processing of each order.
pub struct TradingPlatform {
    matching_engine: MatchingEngine,
    accounts: Accounts,
    tx_log: Vec<Tx>,
}

impl TradingPlatform {
    /// Creates a new instance without any data.
    pub fn new() -> Self {
        TradingPlatform {
            accounts: Accounts::new(),
            matching_engine: MatchingEngine::new(),
            tx_log: vec![],
        }
    }

    /// Fetches the complete order book at this time
    pub fn orderbook(&self) -> Vec<PartialOrder> {
        // let orderbook = vec![];
        let mut asks = self
            .matching_engine
            .asks
            .values()
            .cloned()
            .flatten()
            .collect::<Vec<PartialOrder>>();

        let mut bids = self
            .matching_engine
            .bids
            .values()
            .cloned()
            .flatten()
            .collect::<Vec<PartialOrder>>();
        asks.append(&mut bids);
        asks
    }

    /// Withdraw funds
    pub fn balance_of(&mut self, signer: &str) -> Result<&u64, ApplicationError> {
        self.accounts.balance_of(signer)
    }

    /// Deposit funds
    pub fn deposit(&mut self, signer: &str, amount: u64) -> Result<Tx, ApplicationError> {
        let deposit = self.accounts.deposit(signer, amount)?;
        let log = deposit.clone();
        self.tx_log.push(log);
        Ok(deposit)
    }

    /// Withdraw funds
    pub fn withdraw(&mut self, signer: &str, amount: u64) -> Result<Tx, ApplicationError> {
        let withdraw = self.accounts.withdraw(signer, amount)?;
        let log = withdraw.clone();
        self.tx_log.push(log);
        Ok(withdraw)
    }

    /// Transfer funds between sender and recipient
    pub fn send(
        &mut self,
        sender: &str,
        recipient: &str,
        amount: u64,
    ) -> Result<(Tx, Tx), ApplicationError> {
        let (tx1, tx2) = self.accounts.send(sender, recipient, amount)?;
        let log1 = tx1.clone();
        let log2 = tx2.clone();
        self.tx_log.push(log1);
        self.tx_log.push(log2);
        Ok((tx1, tx2))
    }

    /// Process a given order and apply the outcome to the accounts involved. Note that there are very few safeguards in place.
    pub fn order(&mut self, order: Order) -> Result<Receipt, ApplicationError> {
        let balance = self.accounts.balance_of(&order.signer)?;
        if order.side == Side::Sell && balance < &(order.amount * order.price) {
            return Err(ApplicationError::AccountUnderFunded(
                String::from("Not enough solvency"),
                order.amount * order.price,
            ));
        } else {
            let order_signer = order.signer.clone();
            let order_side = order.side.clone();
            let receipt = self.matching_engine.process(order)?;
            let total_realized: u64 = receipt.matches.iter().map(|m| m.price * m.amount).sum();

            match order_side {
                Side::Buy => receipt.matches.iter().for_each(|m| {
                    self.send(order_signer.as_str(), &m.signer, m.price * m.amount)
                        .unwrap();
                }),
                Side::Sell => receipt.matches.iter().for_each(|m| {
                    self.send(&m.signer, order_signer.as_str(), m.price * m.amount)
                        .unwrap();
                }),
            }

            Ok(receipt)
        }
    }
}

#[cfg(test)]
mod tests {
    // reduce the warnings for naming tests
    #![allow(non_snake_case)]

    use super::*;

    #[test]
    // #[ignore]
    fn test_TradingPlatform_order_requires_deposit_to_order() {
        let mut trading_platform = TradingPlatform::new();

        assert_eq!(
            trading_platform.order(Order {
                price: 10,
                amount: 1,
                side: Side::Sell,
                signer: "ALICE".to_string(),
            }),
            Err(ApplicationError::AccountNotFound("ALICE".to_string()))
        );
        assert!(trading_platform.matching_engine.asks.is_empty());
        assert!(trading_platform.matching_engine.bids.is_empty());
    }

    #[test]
    fn test_TradingPlatform_order_partially_match_order_updates_accounts() {
        let mut trading_platform = TradingPlatform::new();

        // Set up accounts
        assert!(trading_platform.accounts.deposit("ALICE", 100).is_ok());
        assert!(trading_platform.accounts.deposit("BOB", 100).is_ok());

        let alice_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 1,
                side: Side::Sell,
                signer: "ALICE".to_string(),
            })
            .unwrap();
        assert_eq!(alice_receipt.matches, vec![]);
        assert_eq!(alice_receipt.ordinal, 1);

        let bob_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 2,
                side: Side::Buy,
                signer: "BOB".to_string(),
            })
            .unwrap();

        assert_eq!(
            bob_receipt.matches,
            vec![PartialOrder {
                price: 10,
                amount: 1,
                remaining: 0,
                side: Side::Sell,
                signer: "ALICE".to_string(),
                ordinal: 1
            }]
        );
        assert!(trading_platform.matching_engine.asks.is_empty());
        assert_eq!(trading_platform.matching_engine.bids.len(), 1);

        // Check the account balances
        assert_eq!(trading_platform.accounts.balance_of("ALICE"), Ok(&110));
        assert_eq!(trading_platform.accounts.balance_of("BOB"), Ok(&90));
    }

    #[test]
    fn test_TradingPlatform_order_fully_match_order_updates_accounts() {
        let mut trading_platform = TradingPlatform::new();

        // Set up accounts
        assert!(trading_platform.accounts.deposit("ALICE", 100).is_ok());
        assert!(trading_platform.accounts.deposit("BOB", 100).is_ok());

        let alice_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 2,
                side: Side::Sell,
                signer: "ALICE".to_string(),
            })
            .unwrap();
        assert_eq!(alice_receipt.matches, vec![]);
        assert_eq!(alice_receipt.ordinal, 1);

        let bob_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 2,
                side: Side::Buy,
                signer: "BOB".to_string(),
            })
            .unwrap();

        assert_eq!(
            bob_receipt.matches,
            vec![PartialOrder {
                price: 10,
                amount: 2,
                remaining: 0,
                side: Side::Sell,
                signer: "ALICE".to_string(),
                ordinal: 1
            }]
        );

        // A fully matched order doesn't remain in the book
        assert!(trading_platform.matching_engine.asks.is_empty());
        assert!(trading_platform.matching_engine.bids.is_empty());

        // Check the account balances
        assert_eq!(trading_platform.accounts.balance_of("ALICE"), Ok(&120));
        assert_eq!(trading_platform.accounts.balance_of("BOB"), Ok(&80));
    }

    #[test]
    fn test_TradingPlatform_order_fully_match_order_multi_match_updates_accounts() {
        let mut trading_platform = TradingPlatform::new();

        // Set up accounts
        assert!(trading_platform.accounts.deposit("ALICE", 100).is_ok());
        assert!(trading_platform.accounts.deposit("BOB", 100).is_ok());
        assert!(trading_platform.accounts.deposit("CHARLIE", 100).is_ok());

        let alice_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 1,
                side: Side::Sell,
                signer: "ALICE".to_string(),
            })
            .unwrap();
        assert_eq!(alice_receipt.matches, vec![]);
        assert_eq!(alice_receipt.ordinal, 1);

        let charlie_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 1,
                side: Side::Sell,
                signer: "CHARLIE".to_string(),
            })
            .unwrap();
        assert_eq!(charlie_receipt.matches, vec![]);
        assert_eq!(charlie_receipt.ordinal, 2);

        let bob_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 2,
                side: Side::Buy,
                signer: "BOB".to_string(),
            })
            .unwrap();

        assert_eq!(
            bob_receipt.matches,
            vec![
                PartialOrder {
                    price: 10,
                    amount: 1,
                    remaining: 0,
                    side: Side::Sell,
                    signer: "ALICE".to_string(),
                    ordinal: 1
                },
                PartialOrder {
                    price: 10,
                    amount: 1,
                    remaining: 0,
                    side: Side::Sell,
                    signer: "CHARLIE".to_string(),
                    ordinal: 2
                }
            ]
        );
        // A fully matched order doesn't remain in the book
        assert!(trading_platform.matching_engine.asks.is_empty());
        assert!(trading_platform.matching_engine.bids.is_empty());

        // Check account balances
        assert_eq!(trading_platform.accounts.balance_of("ALICE"), Ok(&110));
        assert_eq!(trading_platform.accounts.balance_of("BOB"), Ok(&80));
        assert_eq!(trading_platform.accounts.balance_of("CHARLIE"), Ok(&110));
    }

    #[test]
    fn test_TradingPlatform_order_fully_match_order_no_self_match_updates_accounts() {
        let mut trading_platform = TradingPlatform::new();

        // Set up accounts
        assert!(trading_platform.accounts.deposit("ALICE", 100).is_ok());
        assert!(trading_platform.accounts.deposit("CHARLIE", 100).is_ok());

        let alice_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 1,
                side: Side::Sell,
                signer: "ALICE".to_string(),
            })
            .unwrap();
        assert_eq!(alice_receipt.matches, vec![]);
        assert_eq!(alice_receipt.ordinal, 1);

        let charlie_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 1,
                side: Side::Sell,
                signer: "CHARLIE".to_string(),
            })
            .unwrap();
        assert_eq!(charlie_receipt.matches, vec![]);
        assert_eq!(charlie_receipt.ordinal, 2);

        let bob_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 2,
                side: Side::Buy,
                signer: "ALICE".to_string(),
            })
            .unwrap();

        assert_eq!(
            bob_receipt.matches,
            vec![PartialOrder {
                price: 10,
                amount: 1,
                remaining: 0,
                side: Side::Sell,
                signer: "CHARLIE".to_string(),
                ordinal: 2
            }]
        );
        // A fully matched order doesn't remain in the book
        assert_eq!(trading_platform.matching_engine.asks.len(), 1);
        assert_eq!(trading_platform.matching_engine.bids.len(), 1);
        // Check account balances
        assert_eq!(trading_platform.accounts.balance_of("ALICE"), Ok(&90));
        assert_eq!(trading_platform.accounts.balance_of("CHARLIE"), Ok(&110));
    }

    #[test]
    fn test_TradingPlatform_order_no_match_updates_accounts() {
        let mut trading_platform = TradingPlatform::new();

        // Set up accounts
        assert!(trading_platform.accounts.deposit("ALICE", 100).is_ok());
        assert!(trading_platform.accounts.deposit("BOB", 100).is_ok());

        let alice_receipt = trading_platform
            .order(Order {
                price: 10,
                amount: 2,
                side: Side::Sell,
                signer: "ALICE".to_string(),
            })
            .unwrap();
        assert_eq!(alice_receipt.matches, vec![]);
        assert_eq!(alice_receipt.ordinal, 1);

        let bob_receipt = trading_platform
            .order(Order {
                price: 11,
                amount: 2,
                side: Side::Sell,
                signer: "BOB".to_string(),
            })
            .unwrap();

        assert_eq!(bob_receipt.matches, vec![]);
        assert_eq!(trading_platform.orderbook().len(), 2);

        // Check the account balances
        assert_eq!(trading_platform.accounts.balance_of("ALICE"), Ok(&100));
        assert_eq!(trading_platform.accounts.balance_of("BOB"), Ok(&100));
    }
}

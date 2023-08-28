use std::io::stdin;

use accounting_with_fundamental_rust_1::{accounts::Accounts, errors::AccountingError};

fn read_from_stdin(label: &str) -> String {
    println!("Please input the {label}: ");
    let mut read = String::new();
    stdin()
        .read_line(&mut read)
        .expect("Input string should be retrieved from the user input");

    read.trim().to_string()
}

fn main() {
    let mut ledger = Accounts::new();
    let mut tx_log = vec![];
    println!("Welcome to Fintech");

    loop {
        let command = read_from_stdin("command");
        match command.as_str() {
            "deposit" => {
                let signer = read_from_stdin("signer");
                let amount: u64 = read_from_stdin("amount")
                    .parse()
                    .expect("The input should be a positive number");

                match ledger.deposit(signer.as_str(), amount) {
                    Ok(tx) => tx_log.push(tx),
                    Err(err) => {
                        if let AccountingError::AccountOverFunded(msg, _) = err {
                            println!("Error occured: {msg}")
                        }
                    }
                }
            }
            "withdraw" => {
                let signer = read_from_stdin("signer");
                let amount: u64 = read_from_stdin("amount")
                    .parse()
                    .expect("The input should be a positive number");
                match ledger.withdraw(signer.as_str(), amount) {
                    Ok(tx) => tx_log.push(tx),
                    Err(err) => {
                        if let AccountingError::AccountUnderFunded(msg, _)
                        | AccountingError::AccountNotFound(msg) = err
                        {
                            println!("Error occured: {msg}")
                        }
                    }
                }
            }

            "send" => {
                let sender = read_from_stdin("sender");
                let recipient = read_from_stdin("recipient");
                let amount: u64 = read_from_stdin("amount")
                    .parse()
                    .expect("The input should be a positive number");
                match ledger.send(sender.as_str(), recipient.as_str(), amount) {
                    Ok((tx1, tx2)) => {
                        tx_log.push(tx1);
                        tx_log.push(tx2);
                    }
                    Err(err) => match err {
                        AccountingError::AccountUnderFunded(msg, _)
                        | AccountingError::AccountOverFunded(msg, _)
                        | AccountingError::AccountNotFound(msg) => println!("Error occured: {msg}"),
                    },
                }
            }
            "print" => {
                println!("Ledger : {:#?}", ledger);
                println!("The TX log: {:#?}", tx_log);
            }

            "quit" => break,

            _ => println!("Command {command} not found"),
        }
    }
    // println!("Hello, accounting world!");
    //
    // // We are using simple &str instances as keys
    // // for more sophisticated keys (e.g. hashes)
    // // the data type could remain the same
    //
    // // Deposit an amount to each account
    // let bob = "bob";
    // let alice = "alice";
    // let charlie = "charlie";
    // let initial_amount = 100;
    //
    // let mut ledger = Accounts::new();
    // let mut tx_log = vec![];
    // for signer in &[bob, alice, charlie] {
    //     let status = ledger.deposit(*signer, initial_amount);
    //     println!("Depositing {} for {}: {:?}", signer, initial_amount, status);
    //     // Add the resulting transaction to a list of transactions
    //     // .unwrap() will crash the program if the status is an error.
    //     tx_log.push(status.unwrap());
    // }
    //
    // // Send currency from one account (bob) to the other (alice)
    // let send_amount = 10_u64;
    // let status = ledger.send(bob, alice, send_amount);
    // println!(
    //     "Sent {} from {} to {}: {:?}",
    //     send_amount, bob, alice, status
    // );
    //
    // // Add both transactions to the transaction log
    // let (tx1, tx2) = status.unwrap();
    // tx_log.push(tx1);
    // tx_log.push(tx2);
    //
    // // Withdraw everything from the accounts
    // let tx = ledger.withdraw(charlie, initial_amount).unwrap();
    // tx_log.push(tx);
    // let tx = ledger
    //     .withdraw(alice, initial_amount + send_amount)
    //     .unwrap();
    // tx_log.push(tx);
    //
    // // Here we are withdrawing too much and there won't be a transaction
    // println!(
    //     "Withdrawing {} from {}: {:?}",
    //     initial_amount,
    //     bob,
    //     ledger.withdraw(bob, initial_amount)
    // );
    // // Withdrawing the expected amount results in a transaction
    // let tx = ledger.withdraw(bob, initial_amount - send_amount).unwrap();
    // tx_log.push(tx);
    //
    // // {:?} prints the Debug implementation, {:#?} pretty-prints it
    // println!("Ledger empty: {:?}", ledger);
    // println!("The TX log: {:#?}", tx_log);
}

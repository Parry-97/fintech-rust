use std::io;
mod accounting;
mod core;
mod errors;
mod trading_platform;
mod tx;
use crate::{
    accounting::Accounts,
    core::{Order, Side},
    trading_platform::TradingPlatform,
};

fn read_from_stdin(label: &str) -> String {
    let mut buffer = String::new();
    println!("{}", label);
    io::stdin()
        .read_line(&mut buffer)
        .expect("Couldn't read from stdin");
    buffer.trim().to_owned()
}

fn main() {
    println!("Hello, accounting world!");

    let mut trading_platform = TradingPlatform::new();
    loop {
        let input = read_from_stdin(
            "Choose operation [deposit, withdraw, send, order, txlog, orderbook, print, quit], confirm with return:",
        );
        match input.as_str() {
            "deposit" => {
                let account = read_from_stdin("Account:");

                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let _ = trading_platform.deposit(&account, amount);
                    println!("Deposited {} into account '{}'", amount, account)
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "withdraw" => {
                let account = read_from_stdin("Account:");
                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let _ = trading_platform.withdraw(&account, amount);
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "send" => {
                let sender = read_from_stdin("Sender Account:");
                let recipient = read_from_stdin("Recipient Account:");
                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let _ = trading_platform.send(&sender, &recipient, amount);
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "order" => {
                let signer = read_from_stdin("Order Signer");
                let mut raw_amount = read_from_stdin("Order Amount").parse();
                let amount: u64;
                while raw_amount.is_err() {
                    raw_amount = read_from_stdin("Order Amount").parse();
                }
                amount = raw_amount.unwrap();

                let mut raw_price = read_from_stdin("Order Price").parse();
                let price: u64;
                while raw_price.is_err() {
                    raw_price = read_from_stdin("Order Price").parse();
                }
                price = raw_price.unwrap();

                let side: Side;

                loop {
                    let raw_side = read_from_stdin("Order side");
                    match raw_side.as_str() {
                        "Buy" => {
                            side = Side::Buy;
                            break;
                        }
                        "Sell" => {
                            side = Side::Sell;
                            break;
                        }
                        _ => continue,
                    }
                }

                if let Err(
                    errors::ApplicationError::AccountUnderFunded(msg, _)
                    | errors::ApplicationError::AccountNotFound(msg),
                ) = trading_platform.order(Order {
                    price,
                    amount,
                    side,
                    signer,
                }) {
                    eprintln!("Error occured during order: {}", msg);
                }
            }
            "orderbook" => {
                println!("Orderbook: {:?}", trading_platform.orderbook());
            }

            "txlog" => {
                println!("Transaction Log: {:?}", trading_platform.txlog());
            }

            "print" => {
                println!("Orderbook: {:?}", trading_platform.orderbook());
                println!("Transaction Log: {:?}", trading_platform.txlog());
            }
            "quit" => {
                println!("Quitting...");
                break;
            }
            _ => {
                eprintln!("Invalid option: '{}'", input);
            }
        }
    }
}

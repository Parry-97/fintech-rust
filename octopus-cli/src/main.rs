use octopus_common::types::{Order, Side};
use octopus_web::trading_platform::TradingPlatform;
use std::{io, num::ParseIntError};

fn read_order_parameters() -> Result<Order, String> {
    let account = read_from_stdin("Account:");
    let side = match read_from_stdin("Buy or Sell?:").to_lowercase().as_ref() {
        "buy" => Ok(Side::Buy),
        "sell" => Ok(Side::Sell),
        _ => Err("Unsupported order side"),
    }?;

    let amount = read_from_stdin("Amount:")
        .parse()
        .map_err(|e: ParseIntError| e.to_string())?;
    let price = read_from_stdin("Price:")
        .parse()
        .map_err(|e: ParseIntError| e.to_string())?;
    Ok(Order {
        price,
        amount,
        side,
        signer: account,
    })
}

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

    let mut ledger = TradingPlatform::new();
    loop {
        let input = read_from_stdin(
            "Choose operation [deposit, withdraw, send, print, txlog, order, orderbook, quit], confirm with return:",
        );
        match input.as_str() {
            "deposit" => {
                let account = read_from_stdin("Account:");

                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let _ = ledger.deposit(&account, amount);
                    println!("Deposited {} into account '{}'", amount, account)
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "withdraw" => {
                let account = read_from_stdin("Account:");
                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let _ = ledger.withdraw(&account, amount);
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "send" => {
                let sender = read_from_stdin("Sender Account:");
                let recipient = read_from_stdin("Recipient Account:");
                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let _ = ledger.send(&sender, &recipient, amount);
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "order" => match read_order_parameters() {
                Ok(order) => {
                    println!("{:?}", ledger.order(order));
                }
                Err(msg) => {
                    eprintln!("Invalid Order parameters: '{:?}'", msg);
                }
            },
            "orderbook" => {
                println!("The orderbook: {:#?}", ledger.orderbook());
            }
            "txlog" => {
                println!("The TX log: {:#?}", ledger.transactions);
            }
            "print" => {
                println!("The ledger: {:?}", ledger.accounts);
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

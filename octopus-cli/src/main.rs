use octopus_common::{
    tx::Tx,
    types::{AccountUpdateRequest, ErrorMessage, Order, PartialOrder, Receipt, SendRequest, Side},
};
use reqwest::Url;
use std::{env, io, num::ParseIntError};

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
#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let client = reqwest::Client::new();
    let service_path = args.get(1);

    if service_path.is_none() {
        eprintln!("Please specify the service path to connect to");
        return;
    }

    let service_path = service_path.unwrap();
    if Url::parse(service_path).is_err() {
        eprintln!("Please specify the serice path to connect to");
        return;
    }

    loop {
        let input = read_from_stdin(
            "Choose operation [deposit, withdraw, send, history, order, orderbook, quit], confirm with return:",
        );
        match input.as_str() {
            "deposit" => {
                let account = read_from_stdin("Account:");

                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let deposit = client
                        .post(format!("{}/account/deposit", service_path))
                        .json(&AccountUpdateRequest {
                            signer: account,
                            amount,
                        })
                        .send()
                        .await
                        .expect("The deposit request should be directed to the trading platform service")
                        .json::<serde_json::Value>()
                        .await;
                    match deposit {
                        Ok(deposit) => {
                            // if let Tx::Deposit {
                            //     account: signer,
                            //     amount: deposited,
                            // } = deposit.
                            // {
                            //     println!("Deposited {} into account '{}'", deposited, signer)
                            // }
                            println!("{:#?}", deposit)
                        }
                        Err(inner) => eprintln!("Error occured: {}", inner),
                    }
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "withdraw" => {
                let account = read_from_stdin("Account:");
                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let withdraw = client
                        .post(format!("{}/account/withdraw", service_path))
                        .json(&AccountUpdateRequest {
                            signer:account,
                            amount,
                        })
                        .send()
                        .await
                        .expect("The withdraw request should be directed to the trading platform service")
                        .json::<serde_json::Value>()
                        .await;
                    match withdraw {
                        Ok(withdraw) => {
                            // if let Tx::Withdraw {
                            //     account: signer,
                            //     amount: withdrawn,
                            // } = withdraw
                            // {
                            //     println!("Withdrawn {} from account '{}'", withdrawn, signer)
                            // }
                            println!("{:#?}", withdraw)
                        }
                        Err(inner) => eprintln!("Error occured: {}", inner),
                    }
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "send" => {
                let sender = read_from_stdin("Sender Account:");
                let recipient = read_from_stdin("Recipient Account:");
                let raw_amount = read_from_stdin("Amount:").parse();
                if let Ok(amount) = raw_amount {
                    let response = client
                        .post(format!("{}/account/send", service_path))
                        .json(&SendRequest {
                            sender,
                            recipient,
                            amount
                        })
                        .send()
                        .await
                        .expect("The withdraw request should be directed to the trading platform service")
                        .json::<serde_json::Value>()
                        .await;

                    match response {
                        Ok(send_response) => {
                            // if let (
                            //     Tx::Withdraw {
                            //         account: signer,
                            //         amount: withdrawn,
                            //     },
                            //     Tx::Deposit {
                            //         account: receiver,
                            //         amount: received,
                            //     },
                            // ) = send_response
                            // {
                            //     println!("Withdrawn {} from account '{}'", withdrawn, signer);
                            //     println!("Deposited {} to account '{}'", received, receiver);
                            // }

                            println!("{:#?}", send_response)
                        }
                        Err(inner) => eprintln!("Error occured: {}", inner),
                    }
                } else {
                    eprintln!("Not a number: '{:?}'", raw_amount);
                }
            }
            "order" => {
                match read_order_parameters() {
                    Ok(order) => {
                        let response = client
                        .post(format!("{}/order", service_path))
                        .json(&order)
                        .send()
                        .await
                        .expect("The order request should be directed to the trading platform service")
                        .json::<serde_json::Value>()
                        .await;

                        match response {
                            Ok(receipt) => {
                                // println!("Receipt from the order: {:#?}", receipt)
                                println!("{:#?}", receipt)
                            }
                            Err(inner) => eprintln!("Error occured: {}", inner),
                        }
                    }
                    Err(msg) => {
                        eprintln!("Invalid Order parameters: '{:?}'", msg);
                    }
                }
            }
            "orderbook" => {
                let response = client
                    .post(format!("{}/orderbook", service_path))
                    .send()
                    .await
                    .expect(
                        "The orderbook request should be directed to the trading platform service",
                    )
                    .json::<serde_json::Value>()
                    .await;
                match response {
                    Ok(orderbook) => {
                        // println!("Current orderbook : {:#?}", orderbook);
                        println!("{:#?}", orderbook)
                    }

                    Err(inner) => eprintln!("Error occured: {}", inner),
                }
            }
            // "txlog" => {
            //     println!("The TX log: {:#?}", ledger.transactions);
            // }
            // "print" => {
            //     println!("The ledger: {:?}", ledger.accounts);
            // }
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

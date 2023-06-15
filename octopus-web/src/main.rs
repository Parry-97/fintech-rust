use std::{convert::Infallible, error::Error, sync::Arc};

use octopus_common::{
    errors::ApplicationError,
    types::{
        AccountBalanceRequest, AccountUpdateRequest, ErrorMessage, OctopusError, Order, SendRequest,
    },
};
use octopus_web::trading_platform::TradingPlatform;
use tokio::sync::Mutex;
use warp::{body, hyper::StatusCode, Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    let platform = TradingPlatform::new();
    let db = Arc::new(Mutex::new(platform));

    let account_path = warp::path("account");

    let balance_route = account_path
        .and(warp::path::end())
        .and(warp::get())
        .and(with_db(Arc::clone(&db)))
        .and(body::json::<AccountBalanceRequest>())
        .and_then(account);

    let withdraw_route = account_path
        .and(warp::path("withdraw"))
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and(body::json::<AccountUpdateRequest>())
        .and_then(withdraw);

    let deposit_route = account_path
        .and(warp::path("deposit"))
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and(body::json::<AccountUpdateRequest>())
        .and_then(deposit);

    let send_route = account_path
        .and(warp::path("send"))
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and(body::json::<SendRequest>())
        .and_then(send);

    let order_path = warp::path("order");
    let order_route = order_path
        .and(warp::path::end())
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and(body::json::<Order>())
        .and_then(order);

    let history_route = order_path
        .and(warp::path("history"))
        .and(warp::get())
        .and(with_db(Arc::clone(&db)))
        .and_then(history);

    let orderbook_path = warp::path("orderbook");
    let orderbook_route = orderbook_path
        .and(warp::path::end())
        .and(warp::get())
        .and(with_db(Arc::clone(&db)))
        .and_then(orderbook);

    let account_route = balance_route
        .or(withdraw_route)
        .or(deposit_route)
        .or(send_route)
        .or(order_route)
        .or(history_route)
        .or(orderbook_route)
        .recover(error_handler);

    warp::serve(account_route).run(([127, 0, 0, 1], 3000)).await;
}
async fn account(db: Db, req: AccountBalanceRequest) -> Result<impl Reply, Rejection> {
    match db.lock().await.balance_of(&req.signer) {
        Ok(amount) => Ok(warp::reply::json(amount)),
        Err(msg) => Err(warp::reject::custom(OctopusError::new(msg))),
    }
}
async fn deposit(db: Db, req: AccountUpdateRequest) -> Result<impl Reply, Rejection> {
    match db.lock().await.deposit(&req.signer, req.amount) {
        Ok(tx) => Ok(warp::reply::json(&tx)),
        Err(msg) => Err(warp::reject::custom(OctopusError::new(msg))),
    }
}
async fn withdraw(db: Db, req: AccountUpdateRequest) -> Result<impl Reply, Rejection> {
    match db.lock().await.withdraw(&req.signer, req.amount) {
        Ok(tx) => Ok(warp::reply::json(&tx)),
        Err(msg) => Err(warp::reject::custom(OctopusError::new(msg))),
    }
}
async fn send(db: Db, req: SendRequest) -> Result<impl Reply, Rejection> {
    match db
        .lock()
        .await
        .send(&req.sender, &req.recipient, req.amount)
    {
        Ok(tx) => Ok(warp::reply::json(&tx)),
        Err(msg) => Err(warp::reject::custom(OctopusError::new(msg))),
    }
}
async fn order(db: Db, order: Order) -> Result<impl Reply, Rejection> {
    match db.lock().await.order(order) {
        Ok(receipt) => Ok(warp::reply::json(&receipt)),
        Err(msg) => Err(warp::reject::custom(OctopusError::new(msg))),
    }
}

async fn history(db: Db) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(
        &(db.lock().await.matching_engine.history),
    ))
}

async fn orderbook(db: Db) -> Result<impl Reply, Infallible> {
    Ok(warp::reply::json(&(db.lock().await.orderbook())))
}

async fn error_handler(err: Rejection) -> Result<impl Reply, Infallible> {
    let code;
    let message: String;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "NOT_FOUND".to_owned();
    } else if let Some(e) = err.find::<warp::filters::body::BodyDeserializeError>() {
        // This error happens if the body could not be deserialized correctly
        // We can use the cause to analyze the error and customize the error message
        message = match e.source() {
            Some(cause) => {
                if cause.to_string().contains("denom") {
                    "FIELD_ERROR: denom".to_owned()
                } else {
                    "BAD_REQUEST".to_owned()
                }
            }
            None => "BAD_REQUEST".to_owned(),
        };
        code = StatusCode::BAD_REQUEST;
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        // We can handle a specific error, here METHOD_NOT_ALLOWED,
        // and render it however we want
        code = StatusCode::METHOD_NOT_ALLOWED;
        message = "METHOD_NOT_ALLOWED".to_owned();
    } else if let Some(octopus_error) = err.find::<OctopusError>() {
        match octopus_error {
            OctopusError(ApplicationError::AccountNotFound(signer)) => {
                code = StatusCode::NOT_FOUND;
                message = format!("Cannot find account {}", signer);
            }

            OctopusError(ApplicationError::AccountOverFunded(signer, amount)) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = format!(
                    "Cannot exceed maximum with deposit of {} to  account {}",
                    amount, signer
                );
            }
            OctopusError(ApplicationError::AccountUnderFunded(signer, amount)) => {
                code = StatusCode::INTERNAL_SERVER_ERROR;
                message = format!(
                    "Cannot withdraw from {}  from underfunded  account {}",
                    amount, signer
                );
            }
        }
    } else {
        // We should have expected this... Just log and say its a 500
        eprintln!("unhandled rejection: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "UNHANDLED_REJECTION".to_owned();
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message,
    });

    Ok(warp::reply::with_status(json, code))
}

type Db = Arc<Mutex<TradingPlatform>>;
fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

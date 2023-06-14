use std::sync::Arc;

use octopus_common::types::{AccountBalanceRequest, AccountUpdateRequest, Order, SendRequest};
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

    let withdraw_route = warp::path("withdraw")
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and((body::json::<AccountUpdateRequest>()))
        .and_then(withdraw);

    let deposit_route = warp::path("deposit")
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and((body::json::<AccountUpdateRequest>()))
        .and_then(deposit);

    let send_route = warp::path("send")
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and((body::json::<SendRequest>()))
        .and_then(send);

    let order_route = warp::path("order")
        .and(warp::post())
        .and(with_db(Arc::clone(&db)))
        .and((body::json::<Order>()))
        .and_then(order);

    let account_route = balance_route
        .or(withdraw_route)
        .or(deposit_route)
        .or(send_route)
        .or(order_route);

    warp::serve(account_route).run(([127, 0, 0, 1], 3000)).await;
}
async fn account(db: Db, req: AccountBalanceRequest) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}
async fn deposit(db: Db, req: AccountUpdateRequest) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}
async fn withdraw(db: Db, req: AccountUpdateRequest) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}
async fn send(db: Db, req: SendRequest) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}
async fn order(db: Db, order: Order) -> Result<impl Reply, Rejection> {
    Ok(StatusCode::OK)
}

pub type Db = Arc<Mutex<TradingPlatform>>;
fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

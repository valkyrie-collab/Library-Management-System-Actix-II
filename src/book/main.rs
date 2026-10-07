use std::str::FromStr;
use actix_web::{App, HttpResponse, HttpServer};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{EncodableKey, Keypair};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use lms::handlers::env_file_operation::load_env_file_data;
use lms::models::app_state::{AppState, Database};
use crate::operation::{borrow_book, check_book_presence, get_private_book_details, get_public_book_details, remove_book, remove_book_copy, remove_book_image, return_book, save_book, save_book_image};

pub mod operation;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //appstate needed to make
    load_env_file_data()?;
    let rpc_client: RpcClient = RpcClient::new(String::from("http://127.0.0.1:8899"));
    let program_id: Pubkey = Pubkey::from_str("8mizEDSu4MfS9cudYyquARbTkGVguRWDMHnLaNfd7ooR").unwrap();
    let app_state: actix_web::web::Data<AppState> = actix_web::web::Data::new(AppState::new(rpc_client, program_id));

    let url: String = std::env::var("DEFAULT_URL").unwrap();
    let book_url: String = format!("{}/book", url);
    let pg_pool: PgPool = PgPoolOptions::default().max_connections(2).connect(&book_url).await.unwrap();
    let dbms: actix_web::web::Data<Database> = actix_web::web::Data::new(Database::set_database(pg_pool));
    
    HttpServer::new(move || {
        let json_config: actix_web::web::JsonConfig = actix_web::web::JsonConfig::default().limit(262000)
            .error_handler(|xer, _req| actix_web::error::InternalError::from_response(
                xer, HttpResponse::Conflict().finish()
            ).into());
        App::new().app_data(json_config).app_data(actix_web::web::Data::clone(&dbms))
            .app_data(actix_web::web::Data::clone(&app_state))
            .service(
                actix_web::web::scope("/book")
                    .service(save_book).service(get_public_book_details)
                    .service(get_private_book_details).service(save_book_image)
                    .service(check_book_presence).service(remove_book)
                    .service(remove_book_image).service(remove_book_copy)
                    .service(borrow_book).service(return_book)
            )
    }).bind(("localhost", 8082))?.run().await
}
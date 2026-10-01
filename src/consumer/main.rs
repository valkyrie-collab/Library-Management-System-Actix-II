pub mod operation;

use actix_web::{web, App, HttpServer, HttpResponse};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use lms::handlers::env_file_operation::load_env_file_data;
use lms::models::app_state::Database;
use crate::operation::{add_consumer_image, check_consumer, get_consumer_details, remove_consumer, remove_consumer_image, sign_in, sign_up};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //app state
    load_env_file_data()?;
    let url: String = std::env::var("DEFAULT_URL").unwrap();
    let dbms_url: String = format!("{}/consumer", url);
    let initialize_pg_pool: PgPool = PgPoolOptions::default().max_connections(2).connect(&dbms_url).await.unwrap();
    let database: web::Data<Database> = web::Data::new(Database::set_database(initialize_pg_pool));

    HttpServer::new(move || {
        let json_config: web::JsonConfig = web::JsonConfig::default().limit(262000).error_handler(
            |xer, _xreq| {
                let response: HttpResponse = HttpResponse::Conflict().finish();
                actix_web::error::InternalError::from_response(xer, response).into()
            }
        );
        App::new().app_data(json_config).app_data(web::Data::clone(&database))
            .service(
                web::scope("/consumer")
                    .service(sign_up).service(sign_in).service(check_consumer)
                    .service(get_consumer_details).service(add_consumer_image)
                    .service(remove_consumer).service(remove_consumer_image)
            )
    }).bind(("localhost", 8081))?.run().await
}
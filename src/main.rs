use actix_web::{web, App, HttpServer, http, HttpResponse};
use actix_web_httpauth::middleware::HttpAuthentication;
use lms::auth::validate_token;
use lms::handlers::book_operation::{add_image, get_book_details, get_book_details_private, remove_book, remove_book_copy, remove_book_image, save_book};
use lms::handlers::consumer_operation::{add_consumer_image, get_consumer_details, remove_consumer, remove_consumer_image, sign_in, sign_up};
use lms::handlers::env_file_operation::load_env_file_data;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    load_env_file_data()?;
    let client: web::Data<reqwest::Client> = web::Data::new(reqwest::Client::new());

    HttpServer::new(move || {
        let cors: actix_cors::Cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_headers([http::header::CONTENT_TYPE, http::header::AUTHORIZATION])
            .allowed_methods(["POST", "GET", "DELETE"])
            .max_age(3600);

        let json_config: web::JsonConfig = web::JsonConfig::default().limit(262000).error_handler(
            |err, _req| actix_web::error::InternalError::from_response(
                err, HttpResponse::Conflict().finish()
            ).into()
        );

        App::new().wrap(cors).app_data(json_config).app_data(web::Data::clone(&client))
            .service(
                web::scope("/api").service(
                    web::scope("/authorization")
                        .service(sign_up).service(sign_in)
                ).service(
                    web::scope("/book-consumer").wrap(HttpAuthentication::bearer(validate_token))
                        .service(
                            web::scope("/book")
                                .service(save_book).service(add_image).service(get_book_details).service(remove_book_copy)
                                .service(get_book_details_private).service(remove_book).service(remove_book_image)
                        )
                        .service(
                            web::scope("/consumer")
                                .service(add_consumer_image).service(get_consumer_details)
                                .service(remove_consumer).service(remove_consumer_image)
                        )
                )
            )
    }).bind(("localhost", 8080))?.run().await
}

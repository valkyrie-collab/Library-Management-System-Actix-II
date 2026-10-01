use actix_multipart::{Field, Multipart};
use actix_web::{post, get, delete, web, HttpResponse};
use actix_web::http::header::ContentDisposition;
use futures_util::StreamExt;
use sqlx::{Postgres, QueryBuilder};
use sqlx::postgres::PgQueryResult;
use lms::auth::generate_token;
use lms::models::app_state::Database;
use lms::models::consumer::{Consumer, ConsumerDisplay};

async fn res_check_consumer(db: &web::Data<Database>, name: &String) -> bool {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT EXISTS (
            SELECT 1
            FROM consumer
            WHERE name = $1
        )
        "#
    );

    let query_result: Result<Option<bool>, sqlx::Error> = query_builder.build_query_scalar()
        .bind(name).fetch_optional(db.get_database()).await;

    match query_result {
        Ok(Some(xob)) => xob,
        Ok(None) => false,
        Err(xer) => {
            println!("Error! query error, Exited with error {}", xer);
            false
        }
    }

}

async fn res_sign_up(db: &web::Data<Database>, cnr: &web::Json<Consumer>) -> (bool, String) {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        INSERT INTO consumer ( name, password, roles )
        VALUES ( $1, $2, $3 )
        "#
    );

    let password: String = match bcrypt::hash(cnr.get_password(), bcrypt::DEFAULT_COST) {
        Ok(xbh) => xbh,
        Err(xer) => {
            let xer: String = format!("Error! cannot excrypt password, Exited with error {}", xer);
            return (false, xer);
        }
    };

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build().bind(cnr.get_name())
        .bind(password).bind(cnr.get_roles()).execute(db.get_database()).await;

    match query_result {
        Ok(xpqr) => {
            let msg: String = format!("Success! rows affected {}", xpqr.rows_affected());
            (true, msg)
        }
        Err(xer) => {
            let xer: String = format!("Error! query did not run, Exited with error {}", xer);
            (false, xer)
        }
    }

}

async fn res_sign_in(db: &web::Data<Database>, cnr: &web::Json<Consumer>) -> Option<String> {

    if !res_check_consumer(db, cnr.get_name()).await {
        return None;
    };

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT password
        FROM consumer
        WHERE name = $1
    "#);

    let query_result: Result<Option<String>, sqlx::Error> = query.build_query_scalar()
        .bind(cnr.get_name()).fetch_optional(db.get_database()).await;
    let pass: String = if let Ok(Some(xqr)) = query_result { xqr } else { return None; };

    match bcrypt::verify(cnr.get_password(), &pass) {
        Ok(xbl) => {
            if xbl { generate_token(cnr.get_name(), cnr.get_roles()).await } else { None }
        }
        Err(xer) => {
            println!("Error! while generating token, Extied with error {}", xer);
            None
        }
    }

}

async fn res_add_image(db: &web::Data<Database>, name: &String, img: &mut Multipart) -> (bool, String) {

    if !res_check_consumer(db, name).await {
        return (false, format!("Error! consumer not present {}, sign in first", name));
    }

    let mut file_name: String = String::new();
    let mut file_extension: String = String::new();
    let mut file_data: Vec<u8> = Vec::new();

    while let Some(xrf) = img.next().await {
        let mut xin: Field = match xrf {
            Ok(xrif) => xrif,
            Err(xer) => {
                let xer: String = format!("Error! recived corrupted mulitpart {}", xer);
                return (false, xer);
            }
        };

        let content_dis: &ContentDisposition = if let Some(xcd) = xin.content_disposition() { xcd } else {
            return (false, String::new());
        };

        file_name = String::from(content_dis.get_filename().unwrap_or("unknown"));
        file_extension = xin.content_type().map(|xim| xim.to_string()).unwrap_or(String::from("unknown"));

        while let Some(xrby) = xin.next().await {

            match xrby {
                Ok(xby) => file_data.extend_from_slice(&xby),
                Err(xer) => {
                    let xer: String = format!("Error! read corrupted data of image, Exited with error {}", xer);
                    return (false, xer);
                }
            }

        }

        break;
    }

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        INSERT INTO consumer_image ( name, image_name, image_extension, image_data )
        VALUES ( $1, $2, $3, $4 )
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build()
        .bind(name).bind(&file_name).bind(&file_extension).bind(&file_data).execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => {
            let msg: String = format!("Success! rows affected {}", xqr.rows_affected());
            (true, msg)
        }
        Err(xer) => {
            let xer: String = format!("Error! Sql did not run successfully {}", xer);
            (false, xer)
        }
    }

}

async fn res_get_consumer_details(db: &web::Data<Database>, name: &String) -> Option<ConsumerDisplay> {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT c.name, c.roles, i.image_name, i.image_extension, i.image_data
        FROM consumer c
        INNER JOIN consumer_image i
        ON c.name = i.name
        WHERE c.name = $1
    "#);

    let query_result: Result<Option<ConsumerDisplay>, sqlx::Error> = query.build_query_as().bind(name)
        .fetch_optional(db.get_database()).await;

    query_result.unwrap_or_else(|xer| {
        println!("Error! query failed, Exited with {}", xer);
        None
    })

}

async fn res_remove_consumer(db: &web::Data<Database>, name: &String) -> (bool, String) {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        DELETE FROM consumer
        WHERE name = $1
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build().bind(name)
        .execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => {
            let msg: String = if xqr.rows_affected() > 0 {
                format!("Success! Consumer with name {} deleted successfully, rows affected {}", name, xqr.rows_affected())
            } else {
                format!("Error! There is no consumer with name {}", name)
            };
            (true, msg)
        }
        Err(xer) => {
            let xer: String = format!("Error! query failed to run, Exited with error {}", xer);
            (false, xer)
        }
    }

}

async fn res_remove_consumer_image(db: &web::Data<Database>, name: &String) -> (bool, String) {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        DELETE FROM consumer_image
        WHERE name = $1
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build().bind(name)
        .execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => {
            let msg: String = if xqr.rows_affected() > 0 {
                format!("Success! Consumer with name {} deleted successfully, rows affected {}", name, xqr.rows_affected())
            } else {
                format!("Error! There is no consumer with name {}", name)
            };
            (true, msg)
        }
        Err(xer) => {
            let xer: String = format!("Error! query failed to run, Exited with error {}", xer);
            (false, xer)
        }
    }
}

#[post("/sign-up")]
pub async fn sign_up(db: web::Data<Database>, cnr: web::Json<Consumer>) -> HttpResponse {
    let result_sign_up: (bool, String) = res_sign_up(&db, &cnr).await;

    if result_sign_up.0 {
        HttpResponse::Accepted().body(result_sign_up.1)
    } else {
        HttpResponse::InternalServerError().body(result_sign_up.1)
    }

}

#[post("/sign-in")]
pub async fn sign_in(db: web::Data<Database>, cnr: web::Json<Consumer>) -> HttpResponse {
    let result_sign_in: Option<String> = res_sign_in(&db, &cnr).await;

    if let Some(xtk) = result_sign_in {
        HttpResponse::Ok().body(xtk)
    } else {
        HttpResponse::BadRequest().body("Token did not generate")
    }

}

#[post("/add-consumer-image/{consumer_name}")]
pub async fn add_consumer_image(db: web::Data<Database>, name: web::Path<String>, img: Multipart) -> HttpResponse {
    let consumer_name: String = name.into_inner();
    let mut img: Multipart = img;
    let result_image: (bool, String) = res_add_image(&db, &consumer_name, &mut img).await;

    if result_image.0 {
        HttpResponse::Ok().body(result_image.1)
    } else {
        HttpResponse::InternalServerError().body(result_image.1)
    }

}

#[get("/check-consumer-presence/{consumer_name}")]
pub async fn check_consumer(db: web::Data<Database>, name: web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().body(res_check_consumer(&db, &name.into_inner()).await.to_string())
}


#[get("/consumer-details/{consumer_name}")]
pub async fn get_consumer_details(db: web::Data<Database>, name: web::Path<String>) -> HttpResponse {
    let consumer_name: String = name.into_inner();
    let result_details: Option<ConsumerDisplay> = res_get_consumer_details(&db, &consumer_name).await;

    if let Some(xrd) = result_details {
        HttpResponse::Ok().json(xrd)
    } else {
        HttpResponse::InternalServerError().json(None::<ConsumerDisplay>)
    }

}

#[delete("/remove-consumer/{consumer_username}")]
pub async fn remove_consumer(db: web::Data<Database>, name: web::Path<String>) -> HttpResponse {
    let consumer_name: String = name.into_inner();
    let result_remove: (bool, String) = res_remove_consumer(&db, &consumer_name).await;

    if result_remove.0 {
        HttpResponse::Ok().body(result_remove.1)
    } else {
        HttpResponse::InternalServerError().body(result_remove.1)
    }

}

#[delete("/remove-consumer-image/{consumer_username}")]
pub async fn remove_consumer_image(db: web::Data<Database>, name: web::Path<String>) -> HttpResponse {
    let consumer_name: String = name.into_inner();
    let result_remove: (bool, String) = res_remove_consumer_image(&db, &consumer_name).await;

    if result_remove.0 {
        HttpResponse::Ok().body(result_remove.1)
    } else {
        HttpResponse::InternalServerError().body(result_remove.1)
    }

}

//library pda
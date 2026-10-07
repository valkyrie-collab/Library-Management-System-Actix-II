use std::fs;
use std::io::Write;
use std::str::FromStr;

use actix_multipart::{Field, Multipart};
use actix_web::{post, get, delete, web, HttpResponse};
use actix_web::http::header::ContentDisposition;
use anchor_client::anchor_lang::Key;
use anchor_client::anchor_lang::solana_program::system_program;
use anchor_client::{Client, ClientError, Cluster, Hash, Instruction, Program, Signer, Transaction};
use futures_util::StreamExt;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature};
use sqlx::{Postgres, QueryBuilder};
use sqlx::postgres::PgQueryResult;
use lms::auth::generate_token;
use lms::models::app_state::{AppState, Database};
use lms::models::consumer::{Consumer, ConsumerDisplay};
use base64::{Engine as _, engine::general_purpose};

// fn res_generate_program<'info>(rpc_url: &String, payer: &'info Keypair) -> Result<Program<&'info Keypair>, ClientError> {
//     let client: Client<&'info Keypair> = Client::new(Cluster::Custom(rpc_url.clone(), rpc_url.clone()), payer);
//     client.program(payer.pubkey())
// }

async fn res_initialize_library(app_state: &web::Data<AppState>, name: String, description: String, admin_id: String) -> (bool, String) {
    // let program: Program<&Keypair> = match res_generate_program(app_state.get_rpc_url(), app_state.get_admin()) {
    //     Ok(xps) => xps,
    //     Err(xer) => {
    //         let xer: String = format!("Error! Cannot generate program exited with error {}", xer);
    //         return (false, xer);
    //     }
    // };

    let mut admin_pubkey_file: fs::File = match fs::OpenOptions::new().create(true).write(true).append(true)
        .open("./admin_pubkey_file.txt") {
            Ok(xapf) => xapf,
            Err(xer) => {
                let msg: String = format!("Error! cannot save admin public key {}", xer);
                return (false, msg)
            }
        };
    
    if let Err(xer) = admin_pubkey_file.write(admin_id.as_bytes()) {
        let msg: String = format!("Error! cannot save the admin pubkey {}", xer);
        return (false, msg)
    }

    let admin_id_pubkey: Pubkey = match Pubkey::from_str(&admin_id) {
        Ok(xaip) => xaip,
        Err(xer) => {
            let msg: String = format!("Error! pubkey did not generate exited with error {}", xer);
            return (false, msg);
        }
    };

    let (lib_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"LibraryManagement".as_ref(), admin_id_pubkey.as_ref()], app_state.get_program_id()
    );

    let (tres_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"Treasury".as_ref(), lib_pda.as_ref()], app_state.get_program_id()
    );

    // let init_lib_transaction: Result<Signature, ClientError> = program.request().accounts(
    //     lms_program::accounts::InitLibrary {
    //         lib: lib_pda,
    //         tres: tres_pda,
    //         admin: app_state.get_admin_id(),
    //         system_program: system_program::id()
    //     }
    // ).args(
    //     lms_program::instruction::InitLibrary {
    //         name: name,
    //         des: description,
    //         time: if let Ok(xtd) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
    //             xtd.as_secs()
    //         } else {
    //             return (false, String::from("Cannot able to set the library initialize time"))
    //         }
    //     }
    // ).send().await;

    let init_lib_acc: lms_program::accounts::InitLibrary = lms_program::accounts::InitLibrary {
        lib: lib_pda,
        tres: tres_pda,
        admin: admin_id_pubkey,
        system_program: system_program::id()
    };

    let args: lms_program::instruction::InitLibrary = lms_program::instruction::InitLibrary {
        name: name,
        des: description,
        time: if let Ok(xtd) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
            xtd.as_secs()
        } else {
            return (false, String::from("Cannot able to set the library initialize time"))
        }
    };

    let ix: Instruction = Instruction {
        program_id: app_state.get_program_id().key(),
        accounts: anchor_client::anchor_lang::ToAccountMetas::to_account_metas(&init_lib_acc, None),
        data: anchor_client::anchor_lang::InstructionData::data(&args)
    };

    let recent_block_hash: Hash = match app_state.get_rpc_url().get_latest_blockhash().await {
        Ok(xrbh) => xrbh,
        Err(xer) => {
            let msg: String = format!("Error! cannot generate block hash, Exited with error {}", xer);
            return (false, msg);
        }
    };
    let msg: Message = Message::new_with_blockhash(&[ix], Some(&admin_id_pubkey), &recent_block_hash);
    let init_lib_tx: Transaction = Transaction::new_unsigned(msg);

    match bincode::serialize(&init_lib_tx) {
        Ok(xst) => (true, general_purpose::STANDARD.encode(xst)),
        Err(xer) => {
            let msg: String = format!("Error! bincode serialize error | {}", xer);
            (false, msg)
        }
    }

}

async fn res_initialize_book(app_state: &web::Data<AppState>, isbn_number: String, copy_number: u64, admin_id: &String) -> (bool, String) {
    let admin_id_pubkey: Pubkey = match Pubkey::from_str(&admin_id) {
        Ok(xaip) => xaip,
        Err(xer) => {
            let msg: String = format!("Error! generating admin public key | {}", xer);
            return (false, msg);
        }
    };

    let (lib_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"LibraryManagement".as_ref(), admin_id_pubkey.as_ref()], app_state.get_program_id()
    );

    let (book_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"Book".as_ref(), admin_id_pubkey.as_ref(), isbn_number.as_ref(), copy_number.to_le_bytes().as_ref()], 
        app_state.get_program_id()
    );

    let book_acc: lms_program::accounts::InitBook = lms_program::accounts::InitBook {
        book: book_pda,
        lib: lib_pda,
        admin: admin_id_pubkey,
        system_program: system_program::id()
    };

    let acc_data: lms_program::instruction::InitBook = lms_program::instruction::InitBook {
        isbn_number: isbn_number,
        copy_number: copy_number
    };

    let ix: Instruction = Instruction {
        program_id: app_state.get_program_id().key(),
        accounts: anchor_client::anchor_lang::ToAccountMetas::to_account_metas(&book_acc, None),
        data: anchor_client::anchor_lang::InstructionData::data(&acc_data)
    };

    let recent_block_hash: Hash = match app_state.get_rpc_url().get_latest_blockhash().await {
        Ok(xrbh) => xrbh,
        Err(xer) => {
            let msg: String = format!("Error! hash did not created {}", xer);
            return (false, msg);
        }
    };

    let msg: Message = Message::new_with_blockhash(&[ix], Some(&admin_id_pubkey), &recent_block_hash);
    let tx: Transaction = Transaction::new_unsigned(msg);

    match bincode::serialize(&tx) {
        Ok(xbs) => (true, general_purpose::STANDARD.encode(xbs)),
        Err(xer) => {
            let msg: String = format!("Error! while serialize | {}", xer);
            (false, msg)
        }
    }

}

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

async fn res_is_admin(db: &web::Data<Database>, cnr_name: &String) -> (bool, String) {
    let res_roles: Result<Vec<String>, sqlx::Error> = sqlx::query_scalar(r#"
        SELECT roles FROM consumer WHERE name = $1
    "#).bind(cnr_name).fetch_all(db.get_database()).await;

    let roles: Vec<String> = match res_roles {
        Ok(xrr) => {

            if xrr.is_empty() {
                return (false, format!("Did not found roles with username {}", cnr_name));
            }

            xrr
        }
        Err(xer) => {
            let er_msg: String = format!("Error! query runtime error {}", xer);
            return (false, er_msg);
        }
    };

    let mut is_admin: bool = false;
    let mut msg: String = format!("The user {} is not admin", cnr_name);

    for xi in &roles {
        
        if xi == "admin" {
            is_admin = true;
            msg = format!("The user {} is admin", cnr_name);
            break;
        }

    }

    (is_admin, msg)
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

#[get("/initialize-library/{consumer_name}/{name}/{description}/{admin_id}")]
pub async fn initialize_library(db: web::Data<Database>, app_state: web::Data<AppState>, params: web::Path<(String, String, String, String)>) -> HttpResponse {
    let res_admin: (bool, String) = res_is_admin(&db, &params.0).await;

    if res_admin.0 {
        let res_lib: (bool, String) = res_initialize_library(&app_state, params.1.clone(), params.2.clone(), params.3.clone()).await;
        if res_lib.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(res_lib.1)
    } else {
        HttpResponse::BadRequest().body(res_admin.1)
    }

}

#[get("/initialize-book/{consumer_name}/{isbn_number}/{copy_number}/{admin_id}")]
pub async fn initialize_book(db: web::Data<Database>, app_state: web::Data<AppState>, params: web::Path<(String, String, u64, String)>) -> HttpResponse {
    let res_admin: (bool, String) = res_is_admin(&db, &params.0).await;

    if res_admin.0 {
        let res_lib: (bool, String) = res_initialize_book(&app_state, params.1.clone(), params.2, &params.3).await;
        if res_lib.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(res_lib.1)
    } else {
        HttpResponse::BadRequest().body(res_admin.1)
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
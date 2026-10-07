use std::fs;
use std::io::Read;
use std::str::FromStr;
use actix_multipart::{Field, Multipart};
use actix_web::{web, HttpResponse, post, get, delete};
use actix_web::http::header::ContentDisposition;
use actix_web::web::Bytes;
use anchor_client::anchor_lang::Key;
use anchor_client::anchor_lang::solana_program::system_program;
use anchor_client::{Client, ClientError, Cluster, Hash, Instruction, Program, Signer, Transaction};
use futures_util::StreamExt;
use solana_client::rpc_response::transaction::Signature;
use solana_sdk::message::Message;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use sqlx::{Postgres, QueryBuilder};
use sqlx::postgres::PgQueryResult;
use lms::models::app_state::{AppState, Database};
use lms::models::book::{Book, BookBorrowDetails, BookDetails, BookShow};
use base64::{Engine as _, engine::general_purpose};

//borrow and return book
// fn res_generate_program<'info>(rpc_url: &String, payer: &'info Keypair) -> Option<Program<&'info Keypair>> {
//     let client: Client<&'info Keypair> = Client::new(Cluster::Custom(rpc_url.clone(), rpc_url.clone()), payer);
//     let program: Option<Program<&'info Keypair>> = match client.program(payer.pubkey()) {
//         Ok(xpr) => Some(xpr),
//         Err(xer) => {
//             println!("program not generated exited with with error {}", xer);
//             None
//         }
//     };

//     program
// }

fn get_admin_public_key() -> Option<Pubkey> {
    let admin_id: String = match fs::File::open("./admin_pubkey_file.txt") {
        Ok(mut xfc) => {
            let mut admin_id_string: String = String::with_capacity(50);

            if let Err(xer) = xfc.read_to_string(&mut admin_id_string) {
                println!("Error! cannot read file content {}", xer);
                return None;
            }

            String::from(admin_id_string.trim())
        }
        Err(xer) => {
            let msg: String = format!("Error! opening file {}", xer);
            println!("{}", msg);
            return None;
        }
    };

    let admin_id_pubkey: Pubkey = match Pubkey::from_str(&admin_id) {
        Ok(xaip) => xaip,
        Err(xer) => {
            println!("Error! forming public key {}", xer);
            return None;
        }
    };

    Some(admin_id_pubkey)
}

async fn res_return_book(db: &web::Data<Database>, app_state: &web::Data<AppState>, bdk: &web::Json<BookBorrowDetails>) -> (bool, String) {
    // let program: Program<&Keypair> = if let Some(xrgp) = res_generate_program(app_state.get_rpc_url(), app_state.get_admin()) {
    //     xrgp
    // } else { return (false, String::from("Did not find program")) };

    let admin_id_pubkey: Pubkey = if let Some(xaip) = get_admin_public_key() { xaip } else {
        let msg: String = format!("Error! cannot form pubkey ");
        return (false, msg);
    };

    let (book_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"Book".as_ref(), admin_id_pubkey.as_ref(), bdk.copy_number.to_le_bytes().as_ref(), bdk.book.get_isbn().as_ref()], app_state.get_program_id()
    );

    let (lib_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"LibraryManagement".as_ref(), admin_id_pubkey.as_ref()], app_state.get_program_id()
    );

    let (tres_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"Treasury".as_ref(), lib_pda.as_ref()], app_state.get_program_id()
    );

    let usr_pda: Pubkey = match Pubkey::from_str(&bdk.borrower_id) {
        Ok(xup) => xup,
        Err(xer) => {
            let xer: String = format!("Error! cannot generate user pda {}", xer);
            return (false, xer);
        }
    };

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT borrow_duration FROM book_copy WHERE isbn = $1 AND copy_number = $2
    "#);
    let query_result: Result<Option<i64>, sqlx::Error> = query_builder.build_query_scalar().bind(bdk.book.get_isbn())
        .bind(bdk.copy_number).fetch_optional(db.get_database()).await;

    let fine: i64 = match query_result {
        Ok(Some(xqr)) => {
            let cur_time: i64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(xcr) => xcr.as_secs() as i64,
                Err(xer) => {
                    let msg: String = format!("Error! cannot generate system time | {}", xer);
                    return (false, msg);
                }
            };

            let val: f64 = (cur_time - xqr) as f64;

            ( if val < 0.0 { 0.0 } else { val } * 0.025) as i64
        }
        Ok(None) => return (false, String::from("Error! did not get any value")),
        Err(xer) => {
            let msg: String = format!("Error! query runtime | {}", xer);
            return (false, msg);
        }
    };

    let book_return_acc: lms_program::accounts::ReturnBook = lms_program::accounts::ReturnBook {
        book: book_pda,
        lib: lib_pda,
        tres: tres_pda,
        user: usr_pda,
        system_program: system_program::id()
    };

    let book_args: lms_program::instruction::ReturnBook = lms_program::instruction::ReturnBook {
        fine: fine as u64
    };
    
    let ix: Instruction = Instruction {
        program_id: app_state.get_program_id().key(),
        accounts: anchor_client::anchor_lang::ToAccountMetas::to_account_metas(&book_return_acc, None),
        data: anchor_client::anchor_lang::InstructionData::data(&book_args)
    };

    let recent_block_hash: Hash = match app_state.get_rpc_url().get_latest_blockhash().await {
        Ok(xrbh) => xrbh,
        Err(xer) => {
            let msg: String = format!("Error! hash formation | {}", xer);
            return (false, msg)
        }
    };

    let msg: Message = Message::new_with_blockhash(&[ix], Some(&admin_id_pubkey), &recent_block_hash);
    let tx: Transaction = Transaction::new_unsigned(msg);

    match bincode::serialize(&tx) {
        Ok(xbs) => (true, general_purpose::STANDARD.encode(xbs)),
        Err(xer) => {
            let msg: String = format!("Error! bincode serialize | {}", xer);
            (false, msg)
        }
    }

}

async fn res_update_after_return_book(db: &web::Data<Database>, app_state: &web::Data<AppState>, bdk: &web::Json<BookBorrowDetails>) -> (Option<bool>, String) {
    let admin_id_pubkey: Pubkey = if let Some(xaip) = get_admin_public_key() { xaip } else {
        let msg: String = format!("Error! cannot form pubkey ");
        return (None, msg);
    };

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        UPDATE book_copy SET is_borrowed = FALSE, pda_address = NULL, borrower_address = NULL, borrow_duration = 0
        WHERE isbn = $1 AND copy_number = $2
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query_builder.build().bind(bdk.book.get_isbn())
        .bind(bdk.copy_number).execute(db.get_database()).await;

    let borrow_time_duration: u64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(xtd) => xtd.as_secs() + (2 * 24 * 3600),
        Err(xer) => {
            let xer: String = format!("Error! while generating borrow time {}", xer);
            return (None, xer)
        }
    };

    match query_result {
        Ok(xqr) => (Some(true), format!("Success! rows affected {}", xqr.rows_affected())),
        Err(xer) => {
            println!("Error! query did not ran {}", xer);

            let (book_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
                &[b"Book".as_ref(), admin_id_pubkey.as_ref(), bdk.book.get_isbn().as_ref(), bdk.copy_number.to_le_bytes().as_ref()],
                app_state.get_program_id()
            );

            let (lib_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
                &[b"LibraryManagement".as_ref(), admin_id_pubkey.as_ref()], app_state.get_program_id()
            );

            let usr_pda: Pubkey = match Pubkey::from_str(&bdk.borrower_id) {
                Ok(xup) => xup,
                Err(xer) => {
                    let msg: String = format!("Error! while parsing string to Pubkey | {}", xer);
                    return (None, msg);
                }
            };

            let borrow_book_account: lms_program::accounts::BorrowBook = lms_program::accounts::BorrowBook {
                book: book_pda,
                lib: lib_pda,
                user: usr_pda
            };

            let borrow_book_instruction: lms_program::instruction::BorrowBook = lms_program::instruction::BorrowBook {
                time_dur: borrow_time_duration
            };

            let ix: Instruction = Instruction {
                program_id: app_state.get_program_id().key(),
                accounts: anchor_client::anchor_lang::ToAccountMetas::to_account_metas(&borrow_book_account, None),
                data: anchor_client::anchor_lang::InstructionData::data(&borrow_book_instruction)
            };

            let recent_block_hash: Hash = match app_state.get_rpc_url().get_latest_blockhash().await {
                Ok(xrbh) => xrbh,
                Err(xer) => {
                    let msg: String = format!("Error! Hash generate | {}", xer);
                    return (None, msg)
                }
            };

            let msg: Message = Message::new_with_blockhash(&[ix], Some(&admin_id_pubkey), &recent_block_hash);
            let tx: Transaction = Transaction::new_unsigned(msg);

            match bincode::serialize(&tx) {
                Ok(xbs) => (Some(false), general_purpose::STANDARD.encode(xbs)),
                Err(xer) => {
                    let msg: String = format!("Error! Bincode Serialize | {}", xer);
                    (None, msg)
                }
            }

        }
    }

}

async fn res_borrow_book(app_state: &web::Data<AppState>, bdk: &web::Json<BookBorrowDetails>) -> (bool, String) {
    let admin_id_pubkey: Pubkey = if let Some(xgd) = get_admin_public_key() { xgd } else {
        return (false, String::from("Error! Getting admin address"))
    };

    let (book_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"Book".as_ref(), admin_id_pubkey.as_ref(), bdk.book.get_isbn().as_ref(), bdk.copy_number.to_le_bytes().as_ref()],
        app_state.get_program_id()
    );

    let (lib_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
        &[b"LibraryManagement".as_ref(), admin_id_pubkey.as_ref()], app_state.get_program_id()
    );

    let usr_pda: Pubkey = match Pubkey::from_str(&bdk.borrower_id) {
        Ok(xup) => xup,
        Err(xer) => {
            let msg: String = format!("Error! while parsing string to Pubkey | {}", xer);
            return (false, msg);
        }
    };

    let borrow_book_account: lms_program::accounts::BorrowBook = lms_program::accounts::BorrowBook {
        book: book_pda,
        lib: lib_pda,
        user: usr_pda
    };

    let borrow_time_duration: u64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(xtd) => xtd.as_secs() + (bdk.days * 24 * 3600),
        Err(xer) => {
            let xer: String = format!("Error! while generating borrow time {}", xer);
            return (false, xer)
        }
    };

    let borrow_book_instruction: lms_program::instruction::BorrowBook = lms_program::instruction::BorrowBook {
        time_dur: borrow_time_duration
    };

    let ix: Instruction = Instruction {
        program_id: app_state.get_program_id().key(),
        accounts: anchor_client::anchor_lang::ToAccountMetas::to_account_metas(&borrow_book_account, None),
        data: anchor_client::anchor_lang::InstructionData::data(&borrow_book_instruction)
    };

    let recent_block_hash: Hash = match app_state.get_rpc_url().get_latest_blockhash().await {
        Ok(xrbh) => xrbh,
        Err(xer) => {
            let msg: String = format!("Error! Hash generate | {}", xer);
            return (false, msg)
        }
    };

    let msg: Message = Message::new_with_blockhash(&[ix], Some(&admin_id_pubkey), &recent_block_hash);
    let tx: Transaction = Transaction::new_unsigned(msg);

    match bincode::serialize(&tx) {
        Ok(xbs) => (true, general_purpose::STANDARD.encode(xbs)),
        Err(xer) => {
            let msg: String = format!("Error! Bincode Serialize | {}", xer);
            (false, msg)
        }
    }

}

async fn res_update_borrow_book(db: &web::Data<Database>, app_state: &web::Data<AppState>, bdk: &web::Json<BookBorrowDetails>) -> (Option<bool>, String) {
    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        UPDATE book_copy SET is_borrowed = TRUE, pda_address = $1, borrower_address = $2, borrow_duration = $3
        WHERE isbn = $4 AND copy_number = $5
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query_builder.build().bind(app_state.get_program_id().to_string())
        .bind(&bdk.borrower_id).bind(bdk.days as i64).bind(bdk.book.get_isbn()).bind(bdk.copy_number).execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => (Some(true), format!("Success! rows affected {}", xqr.rows_affected())),
        Err(xer) => {
            let qr_err: String = format!("Error! query runtime | {}", xer);
            
            let admin_id_pubkey: Pubkey = if let Some(xaip) = get_admin_public_key() { xaip } else {
                let msg: String = format!("Error! cannot form pubkey ");
                return (None, msg);
            };

            let (book_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
                &[b"Book".as_ref(), admin_id_pubkey.as_ref(), bdk.copy_number.to_le_bytes().as_ref(), bdk.book.get_isbn().as_ref()], app_state.get_program_id()
            );

            let (lib_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
                &[b"LibraryManagement".as_ref(), admin_id_pubkey.as_ref()], app_state.get_program_id()
            );

            let (tres_pda, _): (Pubkey, u8) = Pubkey::find_program_address(
                &[b"Treasury".as_ref(), lib_pda.as_ref()], app_state.get_program_id()
            );

            let usr_pda: Pubkey = match Pubkey::from_str(&bdk.borrower_id) {
                Ok(xup) => xup,
                Err(xper) => {
                    let xper: String = format!("Error! cannot generate user pda {} | query error {}", xper, &qr_err);
                    return (None, xper);
                }
            };

            let book_return_acc: lms_program::accounts::ReturnBook = lms_program::accounts::ReturnBook {
                book: book_pda,
                lib: lib_pda,
                tres: tres_pda,
                user: usr_pda,
                system_program: system_program::id()
            };

            let book_args: lms_program::instruction::ReturnBook = lms_program::instruction::ReturnBook {
                fine: 0
            };
            
            let ix: Instruction = Instruction {
                program_id: app_state.get_program_id().key(),
                accounts: anchor_client::anchor_lang::ToAccountMetas::to_account_metas(&book_return_acc, None),
                data: anchor_client::anchor_lang::InstructionData::data(&book_args)
            };

            let recent_block_hash: Hash = match app_state.get_rpc_url().get_latest_blockhash().await {
                Ok(xrbh) => xrbh,
                Err(xper) => {
                    let msg: String = format!("Error! hash formation | {} | query error {}", xper, &qr_err);
                    return (None, msg)
                }
            };

            let msg: Message = Message::new_with_blockhash(&[ix], Some(&admin_id_pubkey), &recent_block_hash);
            let tx: Transaction = Transaction::new_unsigned(msg);

            match bincode::serialize(&tx) {
                Ok(xbs) => (Some(false), general_purpose::STANDARD.encode(xbs)),
                Err(xper) => {
                    let msg: String = format!("Error! bincode serialize | {} | query error {}", xper, &qr_err);
                    (None, msg)
                }
            }

        }
    }

}

async fn res_check_book(db: &web::Data<Database>, srh_wrd: &String) -> bool {
    let srh_wrd_lower: String = format!("%{}%", srh_wrd.to_lowercase());
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT EXISTS (
            SELECT 1 FROM book
            WHERE isbn = $1 OR title ILIKE $2 OR author ILIKE $2
        )
    "#);

    let query_result: Result<Option<bool>, sqlx::Error> = query.build_query_scalar()
        .bind(srh_wrd).bind(srh_wrd_lower).fetch_optional(db.get_database()).await;

    match query_result {
        Ok(Some(xqr)) => xqr,
        Ok(None) => {
            println!("Error! query ran but did not get boolean value");
            false
        }
        Err(xer) => {
            println!("Error! query did not ran, Exited with error {}", xer);
            false
        }
    }
}

async fn res_save_book_copy(db: &web::Data<Database>, isbn_number: &String) -> (bool, String) {
    //initialize book pda
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT COUNT(*)
        FROM book_copy
        WHERE isbn = $1
        AND EXISTS (
            SELECT 1
            FROM book
            WHERE isbn = $1
        )
    "#);

    let query_result: Result<Option<i64>, sqlx::Error> = query.build_query_scalar()
        .bind(isbn_number).fetch_optional(db.get_database()).await;

    match query_result {
        Ok(Some(xqr)) => {
            query = QueryBuilder::new(r#"
                INSERT INTO book_copy ( isbn, copy_number )
                VALUES ( $1, $2 )
            "#);

            let query_result: Result<PgQueryResult, sqlx::Error> = query.build()
                .bind(isbn_number).bind(xqr + 1).execute(db.get_database()).await;

            match query_result {
                Ok(xqr) => if xqr.rows_affected() > 0 {
                    (true, format!("Success! rows affected {}", xqr.rows_affected()))
                } else {
                    (false, String::from("Error! query ran but book copy not added"))
                },
                Err(xer) => {
                    let xer: String = format!("Error! query did not ran, Exited with error {}", xer);
                    (false, xer)
                }
            }

        }
        Ok(None) => (false, String::from("Got empty value from query run")),
        Err(xer) => {
            let xer: String = format!("Error! query did not ran, Exited with error {}", xer);
            (false, xer)
        }
    }
}

async fn res_save_book(db: &web::Data<Database>, bok: &web::Json<Book>) -> (bool, String) {

    if res_check_book(db, bok.get_isbn()).await {
        return res_save_book_copy(db, bok.get_isbn()).await;
    }

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        INSERT INTO book ( isbn, title, author )
        VALUES ( $1, $2, $3 )
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build()
        .bind(bok.get_isbn()).bind(bok.get_title()).bind(bok.get_author()).execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => if xqr.rows_affected() > 0 {
            res_save_book_copy(db, bok.get_isbn()).await
        } else {
            (false, String::from("Error! query ran but rows did not get affected"))
        },
        Err(xer) => {
            let xer: String = format!("Error! query did not ran, Exited with error {}", xer);
            (false, xer)
        }
    }

}

async fn res_add_book_image(db: &web::Data<Database>, isbn_number: &String, img: &mut Multipart) -> (bool, String) {
    let mut file_name: String = String::new();
    let mut file_ext: String = String::from("unknown");
    let mut file_data: Vec<u8> = Vec::new();

    while let Some(xrf) = img.next().await {
        let mut field_value: Field = match xrf {
            Ok(xfv) => xfv,
            Err(xer) => {
                let xer: String = format!("Error! fetching image value {}", xer);
                return (false, xer);
            }
        };

        let content_disposition: &ContentDisposition = if let Some(xrs) = field_value.content_disposition() {
            xrs
        } else { return (false, String::from("Did not get content disposition")) };

        file_name.push_str(content_disposition.get_filename().unwrap_or("unknown"));
        file_ext = field_value.content_type().map(|xmp| xmp.to_string()).unwrap_or(file_ext);

        while let Some(xfd) = field_value.next().await {
            let chunks: Bytes = match xfd {
                Ok(xcf) => xcf,
                Err(xer) => {
                    let xer: String = format!("Error! while retriving image data, Exited with error {}", xer);
                    return (false, xer);
                }
            };

            file_data.extend_from_slice(&chunks)
        }

        break;
    }

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        INSERT INTO book_image ( isbn, name, ext, data )
        VALUES ( $1, $2, $3, $4 )
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build().bind(isbn_number).bind(file_name)
        .bind(file_ext).bind(file_data).execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => {

            if xqr.rows_affected() > 0 {
                (true, format!("Success! rows affected {}", xqr.rows_affected()))
            } else {
                (false, String::from("Error! rows affected 0"))
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! Query did not run, Exited with error {}", xer);
            (false, xer)
        }
    }

}

async fn res_get_book_public(db: &web::Data<Database>, srh_wrd: &String) -> Vec<BookShow> {

    if !res_check_book(db, srh_wrd).await {
        println!("There is no book with search word {}", srh_wrd);
        return Vec::<BookShow>::new();
    }

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT b.isbn, b.title, b.author, c.copy_number, c.is_borrowed, i.name, i.ext, i.data
        FROM book b INNER JOIN book_copy c ON b.isbn = c.isbn
        INNER JOIN book_image i ON b.isbn = i.isbn
        WHERE b.isbn = $1 OR b.title ILIKE $2 OR b.author ILIKE $2
    "#);

    let srh_wrd_lower: String = format!("%{}%", srh_wrd.to_lowercase());
    let query_result: Result<Vec<BookShow>, sqlx::Error> = query.build_query_as()
        .bind(srh_wrd).bind(srh_wrd_lower).fetch_all(db.get_database()).await;

    query_result.unwrap_or_else(|xer| {
        println!("Error! query did not ran, Exited with error {}", xer);
        Vec::<BookShow>::new()
    })
}

async fn res_get_book_private(db: &web::Data<Database>, srh_wrd: &String, brw_id: &String) -> Vec<BookShow> {

    if !res_check_book(db, srh_wrd).await {
        println!("Error! The book did not exist");
        return Vec::<BookShow>::new();
    }

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT b.isbn, b.title, b.author, c.copy_number, c.is_borrowed, i.name, i.ext, i.data
        FROM book b INNER JOIN book_copy c ON b.isbn = c.isbn
        INNER JOIN book_image i ON b.isbn = i.isbn
        WHERE ( b.isbn = $1 OR b.title ILIKE $2 OR b.author ILIKE $2 )
        AND c.is_borrowed = TRUE AND c.borrower_id = $3
    "#);

    let srh_wrd_lower: String = format!("%{}%", srh_wrd.to_lowercase());
    let query_result: Result<Vec<BookShow>, sqlx::Error> = query.build_query_as().bind(srh_wrd)
        .bind(srh_wrd_lower).bind(brw_id).fetch_all(db.get_database()).await;

    query_result.unwrap_or_else(|xer| {
        println!("Error! query did not ran, Exited with error {}", xer);
        Vec::<BookShow>::new()
    })
}

async fn res_remove_book(db: &web::Data<Database>, srh_wrd: &String) -> (bool, String) {
    //app sate to check if the book is borrowed or not
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        SELECT EXISTS (
            SELECT 1 FROM book WHERE ( isbn = $1 OR title ILIKE $1 OR author ILIKE $1 )
        )
    "#);

    let query_result: Result<Option<bool>, sqlx::Error> = query.build_query_scalar()
        .bind(srh_wrd).fetch_optional(db.get_database()).await;

    match query_result {
        Ok(Some(xqr)) => {

            if xqr {
                query = QueryBuilder::new(r#"
                    DELETE FROM book b WHERE b.isbn = $1 OR b.title ILIKE $1 OR b.author ILIKE $1
                    AND NOT EXISTS ( SELECT 1 FROM book_copy c WHERE b.isbn = c.isbn )
                "#);

                let query_result: Result<PgQueryResult, sqlx::Error> = query.build()
                    .bind(srh_wrd).execute(db.get_database()).await;

                match query_result {
                    Ok(xqr) => {

                        if xqr.rows_affected() > 0 {
                            (true, format!("Success! rows affected {}", xqr.rows_affected()))
                        } else {
                            (false, String::from("UnSuccessful! rows affected 0"))
                        }

                    }
                    Err(xer) => {
                        let xer: String = format!("Error! delete query did not ran, Exited with error {}", xer);
                        (false, xer)
                    }
                }

            } else {
                (false, format!("Already Deleted! book {} as been deleted successfully", srh_wrd))
            }
        }
        Ok(None) => (false, String::from("Error! query ran but return null value")),
        Err(xer) => {
            let xer: String = format!("Error! query did not ran, Exited with error {}", xer);
            (false, xer)
        }
    }

}

async fn res_remove_book_copy(db: &web::Data<Database>, isbn_number: &String, copy_number: i64) -> (bool, String) {
    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(

        if copy_number > 0 {
            r#"
                SELECT EXISTS (
                    SELECT 1 FROM book_copy WHERE isbn = $1 AND copy_number = $2
                    AND is_borrowed = FALSE
                )
            "#
        } else {
            r#"
                SELECT EXISTS (
                    SELECT 1 FROM book_copy WHERE isbn = $1
                    AND is_borrowed = FALSE
                )
            "#
        }

    );

    let query_result: Result<Option<bool>, sqlx::Error> = if copy_number > 0 {
        query.build_query_scalar().bind(isbn_number).bind(copy_number)
    } else {
        query.build_query_scalar().bind(isbn_number)
    }.fetch_optional(db.get_database()).await;

    match query_result {
        Ok(Some(xqr)) => {

            if xqr {
                let mut query: QueryBuilder<Postgres> = QueryBuilder::new(

                    if copy_number > 0 {
                        r#"
                            DELETE FROM book_copy WHERE isbn = $1 AND copy_number = $2
                            AND is_borrowed = FALSE
                        "#
                    } else {
                        r#"
                            DELETE FROM book_copy WHERE isbn = $1 AND is_borrowed = FALSE
                        "#
                    }

                );

                let query_result: Result<PgQueryResult, sqlx::Error> = if copy_number > 0 {
                    query.build().bind(isbn_number).bind(copy_number)
                } else {
                    query.build().bind(isbn_number)
                }.execute(db.get_database()).await;

                match query_result {
                    Ok(xqr) => {

                        if xqr.rows_affected() > 0 {
                            (true, format!("Success! rows affected {}", xqr.rows_affected()))
                        } else {
                            (false, String::from("Error! rows affected 0"))
                        }

                    }
                    Err(xer) => {
                        let xer: String = format!("Error! delete query did not ran, Exited with error {}", xer);
                        (false, xer)
                    }
                }

            } else {
                (false, format!("Already Deleted! book {} as been deleted successfully, not borrowed", isbn_number))
            }
        }
        Ok(None) => (false, String::from("Error! query ran but return null value")),
        Err(xer) => {
            let xer: String = format!("Error! query did not ran, Exited with error {}", xer);
            (false, xer)
        }
    }

}

async fn res_remove_book_image(db: &web::Data<Database>, isbn_number: &String) -> (bool, String) {

    if !res_check_book(db, isbn_number).await {
        return (false, format!("Error! There is no book with isbn number {}", isbn_number));
    }

    let mut query: QueryBuilder<Postgres> = QueryBuilder::new(r#"
        DELETE FROM book_image WHERE isbn = $1
    "#);

    let query_result: Result<PgQueryResult, sqlx::Error> = query.build().bind(isbn_number)
        .execute(db.get_database()).await;

    match query_result {
        Ok(xqr) => {

            if xqr.rows_affected() > 0 {
                (true, format!("Success! rows affected {}", xqr.rows_affected()))
            } else {
                (false, String::from("Error! rows affected 0"))
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! query did not run, Exited with error {}", xer);
            (false, xer)
        }
    }

}

#[post("/save-book")]
pub async fn save_book(db: web::Data<Database>, bok: web::Json<Book>) -> HttpResponse {
    let save_book_result: (bool, String) = res_save_book(&db, &bok).await;
    if save_book_result.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(save_book_result.1)
}

#[post("/save-book-image/{isbn_number}")]
pub async fn save_book_image(db: web::Data<Database>, isbn_number: web::Path<String>, img: Multipart) -> HttpResponse {
    let mut img: Multipart = img;
    let isbn_number: String = isbn_number.into_inner();
    let book_image_result: (bool, String) = res_add_book_image(&db, &isbn_number, &mut img).await;
    if book_image_result.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(book_image_result.1)
}

#[get("/get-book-details-public/{search_word}")]
pub async fn get_public_book_details(db: web::Data<Database>, search_word: web::Path<String>, ) -> HttpResponse {
    let srh_wrd: String = search_word.into_inner();
    let book_details_result: Vec<BookShow> = res_get_book_public(&db, &srh_wrd).await;
    if book_details_result.is_empty() { HttpResponse::InternalServerError() } else { HttpResponse::Ok() }.json(book_details_result)
}

#[get("/get-book-details-private/{search_word}/{borrow_id}")]
pub async fn get_private_book_details(db: web::Data<Database>, words: web::Path<(String, String)>) -> HttpResponse {
    let srh_wrd: String = words.0.clone();
    let brw_id: String = words.1.clone();
    let book_details_result: Vec<BookShow> = res_get_book_private(&db, &srh_wrd, &brw_id).await;
    if book_details_result.is_empty() { HttpResponse::InternalServerError() } else { HttpResponse::Ok() }.json(book_details_result)
}

#[get("/check-book-presence/{search_word}")]
pub async fn check_book_presence(db: web::Data<Database>, search_word: web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().body(res_check_book(&db, &search_word.into_inner()).await.to_string())
}

#[post("/borrow-book")]
pub async fn borrow_book(app_state: web::Data<AppState>, bdk: web::Json<BookBorrowDetails>) -> HttpResponse {
    let response: (bool, String) = res_borrow_book(&app_state, &bdk).await;
    if response.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(response.1)
}

#[post("/return-book-on-failed")]
pub async fn return_failed_borrowed_book(db: web::Data<Database>, app_state: web::Data<AppState>, bdk: web::Json<BookBorrowDetails>) -> HttpResponse {
    let response: (Option<bool>, String) = res_update_borrow_book(&db, &app_state, &bdk).await;

    match response.0 {
        Some(xbl) => {
            if xbl { HttpResponse::Accepted() } else { HttpResponse::Ok() }.body(response.1)
        }
        None => {
            HttpResponse::InternalServerError().body(response.1)
        }
    }
}

#[post("/return-book")]
pub async fn return_book(db: web::Data<Database>, app_state: web::Data<AppState>, bdk: web::Json<BookBorrowDetails>) -> HttpResponse {
    let response: (bool, String) = res_return_book(&db, &app_state, &bdk).await;
    if response.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(response.1)
}

#[post("/borrow-book-on-failed")]
pub async fn borrow_failed_return_book(db: web::Data<Database>, app_state: web::Data<AppState>, bdk: web::Json<BookBorrowDetails>) -> HttpResponse {
    let response: (Option<bool>, String) = res_update_after_return_book(&db, &app_state, &bdk).await;

    match response.0 {
        Some(xbl) => {
            if xbl { HttpResponse::Accepted() } else { HttpResponse::Ok() }.body(response.1)
        }
        None => {
            HttpResponse::InternalServerError().body(response.1)
        }
    }
}

#[delete("/remove-book/{search_word}")]
pub async fn remove_book(db: web::Data<Database>, search_word: web::Path<String>) -> HttpResponse {
    let srh_wrd: String = search_word.into_inner();
    let remove_book_details: (bool, String) = res_remove_book(&db, &srh_wrd).await;
    if remove_book_details.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(remove_book_details.1)
}

#[delete("/remove-book-copy/{isbn_number}/{copy_number}")]
pub async fn remove_book_copy(db: web::Data<Database>, words: web::Path<(String, i64)>) -> HttpResponse {
    let srh_wrd: String = words.0.clone();
    let cp_num: i64 = words.1;
    let remove_book_details: (bool, String) = res_remove_book_copy(&db, &srh_wrd, cp_num).await;
    if remove_book_details.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(remove_book_details.1)
}


#[delete("/remove-book-image/{search_word}")]
pub async fn remove_book_image(db: web::Data<Database>, search_word: web::Path<String>) -> HttpResponse {
    let srh_wrd: String = search_word.into_inner();
    let remove_book_details: (bool, String) = res_remove_book_image(&db, &srh_wrd).await;
    if remove_book_details.0 { HttpResponse::Ok() } else { HttpResponse::InternalServerError() }.body(remove_book_details.1)
}
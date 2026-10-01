use actix_multipart::{Field, Multipart};
use actix_web::{web, HttpResponse, post, get, delete};
use actix_web::http::header::ContentDisposition;
use actix_web::web::Bytes;
use futures_util::StreamExt;
use sqlx::{Postgres, QueryBuilder};
use sqlx::postgres::PgQueryResult;
use lms::models::app_state::{AppState, Database};
use lms::models::book::{Book, BookDetails, BookShow};

//borrow and return book

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
pub async fn get_private_book_details(db: web::Data<Database>, words: web::Path<(String,String)>) -> HttpResponse {
    let srh_wrd: String = words.0.clone();
    let brw_id: String = words.1.clone();
    let book_details_result: Vec<BookShow> = res_get_book_private(&db, &srh_wrd, &brw_id).await;
    if book_details_result.is_empty() { HttpResponse::InternalServerError() } else { HttpResponse::Ok() }.json(book_details_result)
}

#[get("/check-book-presence/{search_word}")]
pub async fn check_book_presence(db: web::Data<Database>, search_word: web::Path<String>) -> HttpResponse {
    HttpResponse::Ok().body(res_check_book(&db, &search_word.into_inner()).await.to_string())
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
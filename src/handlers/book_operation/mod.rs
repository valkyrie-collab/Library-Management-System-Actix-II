use std::env;

use actix_web::{post, get, delete, web, HttpResponse};

use crate::models::book::{Book, BookBorrowDetails, BookShow, BookShowConsumer};

#[post("/save-book")]
pub async fn save_book(client: web::Data<reqwest::Client>, bok: web::Json<Book>) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in save book, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let book_url: String = format!("{}/book/save-book", url);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(&book_url).header("content-type", "application/json")
    .json(&bok).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let msg: String = xrs.text().await.unwrap_or(String::from("unknown"));

            if is_success {
                HttpResponse::Accepted().body(msg)
            } else {
                HttpResponse::BadRequest().body(msg)
            }
        }
        Err(xer) => {
            let xer: String = format!("Error! did not get response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[post("/add-book-image/{isbn_number}")]
pub async fn add_image(client: web::Data<reqwest::Client>, isbn_number: web::Path<String>, img: web::Bytes) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in add book image, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let isbn_number: String = isbn_number.into_inner();
    let book_url: String = format!("{}/book/save-book-image/{}", url, &isbn_number);

    let file_name: String = format!("{}_book_image_file", isbn_number);
    let image_part: reqwest::multipart::Part = match reqwest::multipart::Part::bytes(img.to_vec())
    .file_name(file_name.clone()).mime_str("application/octet-stream") {
        Ok(xip) => xip,
        Err(xer) => {
            let xer: String = format!("Error! cannot generate multipart file, Exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let image_form: reqwest::multipart::Form = reqwest::multipart::Form::new().part(file_name, image_part);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(&book_url).multipart(image_form).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let msg: String = xrs.text().await.unwrap_or(String::from("unknown"));

            if is_success {
                HttpResponse::Accepted().body(msg)
            } else {
                HttpResponse::BadRequest().body(msg)
            }
        }
        Err(xer) => {
            let xer: String = format!("Error! did not get response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[get("/get-book-details-public/{search_word}")]
pub async fn get_book_details(client: web::Data<reqwest::Client>, search_word: web::Path<String>) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in get book details public, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let srh_wrd: String = search_word.into_inner();
    let book_url: String = format!("{}/book/get-book-details-public/{}", url, srh_wrd);
    let response: Result<reqwest::Response, reqwest::Error> = client.get(&book_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();

            if is_success {
                let books: Vec<BookShow> = xrs.json().await.unwrap_or(Vec::new());
                HttpResponse::Ok().json(books)
            } else {
                HttpResponse::BadRequest().json(Vec::<BookShow>::new())
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[get("/get-book-details-private/{search_word}/{borrower_id}")]
pub async fn get_book_details_private(client: web::Data<reqwest::Client>, words: web::Path<(String, String)>) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in get book details private, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let search_word: String = words.0.clone();
    let borrower_id: String = words.1.clone();
    let book_url: String = format!("{}/book/get-book-details-private/{}/{}", url, search_word, borrower_id);
    let response: Result<reqwest::Response, reqwest::Error> = client.get(&book_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();

            if is_success {
                let books: Vec<BookShow> = xrs.json().await.unwrap_or(Vec::new());
                HttpResponse::Ok().json(books)
            } else {
                HttpResponse::BadRequest().json(Vec::<BookShow>::new())
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

// #[get("/search-book-public/{search_word}")]
// pub async fn search_book_public(client: web::Data<reqwest::Client>, srh: web::Path<String>) -> HttpResponse {
//     let url: String = match env::var("BOOK_URL") {
//         Ok(xbu) => xbu,
//         Err(xer) => {
//             let xer: String = format!("Error! did not find book url in search book public, exited with error {}", xer);
//             return HttpResponse::InternalServerError().body(xer);
//         }
//     };
// 
//     let srh: String = srh.into_inner();
//     let book_url: String = format!("{}/search-book-public/{}", url, &srh);
//     let response: Result<reqwest::Response, reqwest::Error> = client.get(&book_url).send().await;
// 
//     match response {
//         Ok(xrs) => {
//             let is_success: bool = xrs.status().is_success();
// 
//             if is_success {
//                 let books: Vec<BookShow> = xrs.json().await.unwrap_or(Vec::new());
//                 HttpResponse::Ok().json(books)
//             } else {
//                 HttpResponse::BadRequest().json(Vec::<BookShow>::new())
//             }
// 
//         }
//         Err(xer) => {
//             let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
//             HttpResponse::InternalServerError().body(xer)
//         }
//     }
// 
// }

//#[get("/search-book-private/{search_word}")]
// pub async fn search_book_private(client: web::Data<reqwest::Client>, srh: web::Path<String>) -> HttpResponse {
//     let url: String = match env::var("BOOK_URL") {
//         Ok(xbu) => xbu,
//         Err(xer) => {
//             let xer: String = format!("Error! did not find book url in search book private, exited with error {}", xer);
//             return HttpResponse::InternalServerError().body(xer);
//         }
//     };
// 
//     let srh: String = srh.into_inner();
//     let book_url: String = format!("{}/search-book-private/{}", url, &srh);
//     let response: Result<reqwest::Response, reqwest::Error> = client.get(&book_url).send().await;
// 
//     match response {
//         Ok(xrs) => {
//             let is_success: bool = xrs.status().is_success();
// 
//             if is_success {
//                 let books: Vec<BookShowConsumer> = xrs.json().await.unwrap_or(Vec::new());
//                 HttpResponse::Ok().json(books)
//             } else {
//                 HttpResponse::BadRequest().json(Vec::<BookShowConsumer>::new())
//             }
// 
//         }
//         Err(xer) => {
//             let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
//             HttpResponse::InternalServerError().body(xer)
//         }
//     }
// 
// }

#[post("/return-book")]
pub async fn return_book(client: web::Data<reqwest::Client>, bdk: web::Json<BookBorrowDetails>) -> HttpResponse {
    let url: String = match std::env::var("BOOK_URL") {
        Ok(xrl) => xrl,
        Err(xer) => {
            let msg: String = format!("Error! while getting book url {}", xer);
            return HttpResponse::InternalServerError().body(msg);
        }
    };

    let bk_url: String = format!("{}/book/return-book", url);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(&bk_url).header("content-type", "application/json")
        .json(&bdk).send().await;

    match response {
        Ok(xrp) => {
            let is_succeed: bool = xrp.status().is_success();
            let msg: String = xrp.text().await.unwrap_or_else(
                |xer| format!("Error! did not get any response message | {}", xer)
            );
            if is_succeed { HttpResponse::Ok() } else { HttpResponse::BadRequest() }.body(msg)
        }
        Err(xer) => {
            let msg: String = format!("Error! did not get response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(msg)
        }
    }

}

#[post("/borrow-book")]
pub async fn borrow_book(client: web::Data<reqwest::Client>, bdk: web::Json<BookBorrowDetails>) -> HttpResponse {
    let url: String = match std::env::var("BOOK_URL") {
        Ok(xrl) => xrl,
        Err(xer) => {
            let msg: String = format!("Error! while getting book url {}", xer);
            return HttpResponse::InternalServerError().body(msg);
        }
    };

    let bk_url: String = format!("{}/book/borrow-book", url);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(&bk_url).header("content-type", "application/json")
        .json(&bdk).send().await;

    match response {
        Ok(xrp) => {
            let is_succeed: bool = xrp.status().is_success();
            let msg: String = xrp.text().await.unwrap_or_else(
                |xer| format!("Error! did not get any response message | {}", xer)
            );
            if is_succeed { HttpResponse::Ok() } else { HttpResponse::BadRequest() }.body(msg)
        }
        Err(xer) => {
            let msg: String = format!("Error! did not get response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(msg)
        }
    }
}

#[delete("/remove-book/{isbn_number}")]
pub async fn remove_book(client: web::Data<reqwest::Client>, isbn_number: web::Path<String>) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in remove book, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let isbn_number: String = isbn_number.into_inner();
    let book_url: String = format!("{}/book/remove-book/{}", url, &isbn_number);
    let response: Result<reqwest::Response, reqwest::Error> = client.delete(&book_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let msg: String = xrs.text().await.unwrap_or(String::from("unknown"));

            if is_success {
                HttpResponse::Ok().body(msg)
            } else {
                HttpResponse::BadRequest().body(msg)
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[delete("/remove-book-copy/{isbn_number}/{copy_number}")]
pub async fn remove_book_copy(client: web::Data<reqwest::Client>, isbn_cp: web::Path<(String, i64)>) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in remove book, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let isbn_number: String = isbn_cp.0.clone();
    let copy_number: i64 = isbn_cp.1;
    let book_url: String = format!("{}/book/remove-book-copy/{}/{}", url, &isbn_number, &copy_number);
    let response: Result<reqwest::Response, reqwest::Error> = client.delete(&book_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let msg: String = xrs.text().await.unwrap_or(String::from("unknown"));

            if is_success {
                HttpResponse::Ok().body(msg)
            } else {
                HttpResponse::BadRequest().body(msg)
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }
}

#[delete("/remove-book-image/{isbn_number}")]
pub async fn remove_book_image(client: web::Data<reqwest::Client>, isbn_number: web::Path<String>) -> HttpResponse {
    let url: String = match env::var("BOOK_URL") {
        Ok(xbu) => xbu,
        Err(xer) => {
            let xer: String = format!("Error! did not find book url in remove book image, exited with error {}", xer);
            return HttpResponse::InternalServerError().body(xer);
        }
    };

    let isbn_number: String = isbn_number.into_inner();
    let book_url: String = format!("{}/book/remove-book-image/{}", url, &isbn_number);
    let response: Result<reqwest::Response, reqwest::Error> = client.delete(&book_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let msg: String = xrs.text().await.unwrap_or(String::from("unknown"));

            if is_success {
                HttpResponse::Ok().body(msg)
            } else {
                HttpResponse::BadRequest().body(msg)
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! did not get any server response, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}
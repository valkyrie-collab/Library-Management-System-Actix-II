use actix_web::{HttpResponse, delete, get, post, web};

use crate::models::consumer::{Consumer, ConsumerDisplay};

#[post("/sign-up")]
pub async fn sign_up(client: web::Data<reqwest::Client>, cnr: web::Json<Consumer>) -> HttpResponse {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            let xer: String = format!("Error! while fetching consumer url in sign up {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let consumer_url: String = format!("{}/consumer/sign-up", url);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(consumer_url)
    .header("content-type", "application/json").json(&cnr).send().await;

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
            let xer: String = format!("Error! while getting response from sign up, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[post("/sign-in")]
pub async fn sign_in(client: web::Data<reqwest::Client>, cnr: web::Json<Consumer>) -> HttpResponse {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            let xer: String = format!("Error! while fetching consumer url in sign in {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let consumer_url: String = format!("{}/consumer/sign-in", url);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(consumer_url)
    .header("content-type", "application/json").json(&cnr).send().await;

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
            let xer: String = format!("Error! while getting response from sign in, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[post("/add-consumer-image/{consumer_name}")]
pub async fn add_consumer_image(client: web::Data<reqwest::Client>, img_data: web::Bytes, name: web::Path<String>) -> HttpResponse {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            let xer: String = format!("Error! while fetching consumer url in consumer details {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let consumer_name: String = name.into_inner();

    let part: reqwest::multipart::Part = match reqwest::multipart::Part::bytes(img_data.to_vec())
    .file_name(format!("{}_image_file", consumer_name)).mime_str("application/octet-stream") {
        Ok(xrmp) => xrmp,
        Err(xer) => {
            let xer: String = format!("Error! in add consumer image while performing image operaiton, Exited with {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let form: reqwest::multipart::Form = reqwest::multipart::Form::new().part("image_file", part);

    let consumer_url: String = format!("{}/consumer/add-consumer-image/{}", url, &consumer_name);
    let response: Result<reqwest::Response, reqwest::Error> = client.post(consumer_url).multipart(form).send().await;

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
            let xer: String = format!("Error! while getting response from add consumer image, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[get("/consumer-details/{consumer_name}")]
pub async fn get_consumer_details(client: web::Data<reqwest::Client>, name: web::Path<String>) -> HttpResponse {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            let xer: String = format!("Error! while fetching consumer url in consumer details {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let consumer_name: String = name.into_inner();
    let consumer_url: String = format!("{}/consumer/consumer-details/{}", url, &consumer_name);
    let response: Result<reqwest::Response, reqwest::Error> = client.get(&consumer_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let cnr_dis: Option<ConsumerDisplay> = match xrs.json().await {
                Ok(xcd) => xcd,
                Err(xer) => {
                    let xer: String = format!("Error! in get consumer details did not find consumer display, Exited with error {}", xer);
                    return HttpResponse::BadRequest().body(xer);
                }
            };

            if is_success {
                HttpResponse::Ok().json(cnr_dis)
            } else {
                HttpResponse::BadRequest().json(cnr_dis)
            }

        }
        Err(xer) => {
            let xer: String = format!("Error! in getting response, in get consumer details, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[delete("/remove-consumer/{consumer_username}")]
pub async fn remove_consumer(client: web::Data<reqwest::Client>, name: web::Path<String>) -> HttpResponse {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            let xer: String = format!("Error! while fetching consumer url in remove consumer {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let consumer_name: String = name.into_inner();
    let consumer_url: String = format!("{}/consumer/remove-consumer/{}", url, &consumer_name);
    let response: Result<reqwest::Response, reqwest::Error> = client.delete(&consumer_url).send().await;

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
            let xer: String = format!("Error! while getting response from remove consumer, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

#[delete("/remove-consumer-image/{consumer_username}")]
pub async fn remove_consumer_image(client: web::Data<reqwest::Client>, name: web::Path<String>) -> HttpResponse {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            let xer: String = format!("Error! while fetching consumer url in remove consumer image {}", xer);
            return HttpResponse::BadRequest().body(xer)
        }
    };

    let consumer_name: String = name.into_inner();
    let consumer_url: String = format!("{}/consumer/remove-consumer-image/{}", url, &consumer_name);
    let response: Result<reqwest::Response, reqwest::Error> = client.delete(&consumer_url).send().await;

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
            let xer: String = format!("Error! while getting response from remove consumer image, Exited with error {}", xer);
            HttpResponse::InternalServerError().body(xer)
        }
    }

}

// #[get("/check-consumer-presence/{consumer_username}")]
pub async fn check_consumer(client: &web::Data<reqwest::Client>, consumer_name: &String) -> bool {
    let url: String = match std::env::var("CONSUMER_URL") {
        Ok(xurl) => xurl,
        Err(xer) => {
            println!("Error! while fetching consumer url in check consumer presence {}", xer);
            return false;
        }
    };

    let consumer_url: String = format!("{}/consumer/check-consumer-presence/{}", url, consumer_name);
    let response: Result<reqwest::Response, reqwest::Error> = client.get(&consumer_url).send().await;

    match response {
        Ok(xrs) => {
            let is_success: bool = xrs.status().is_success();
            let msg: bool = xrs.text().await.unwrap_or(String::from("false")).parse().unwrap_or(false);

            if is_success {
                msg
            } else {
                false
            }
        }
        Err(xer) => {
            println!("Error! while getting response from check consumer, Exited with error {}", xer);
            false
        }
    }

}
//Initialization of Library Management Pda

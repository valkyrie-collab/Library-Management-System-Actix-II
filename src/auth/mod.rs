use actix_web::HttpMessage;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use crate::handlers::consumer_operation::check_consumer;
use crate::models::app_state::Claims;

pub async fn generate_token(sub: &String, roles: &Vec<String>) -> Option<String> {
    let secret_key: String = match std::env::var("SECRET_KEY") {
        Ok(xsk) => xsk,
        Err(xer) => {
            println!("Error! cannot get secret key, Exited with error {}", xer);
            return None;
        }
    };

    let exp: u64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(xtd) => {
            xtd.as_secs() + (48 * 3600)
        }
        Err(xer) => {
            println!("Cannot generate current time, Exited with error {}", xer);
            return None;
        }
    };

    let claims: Claims = Claims::new(sub.clone(), exp, roles.clone());
    let encoding_result: Result<String, jsonwebtoken::errors::Error> = jsonwebtoken::encode(
        &jsonwebtoken::Header::default(), &claims, &jsonwebtoken::EncodingKey::from_secret(secret_key.as_bytes())
    );

    match encoding_result  {
        Ok(xje) => Some(xje),
        Err(xer) => {
            println!("Error! cannot generate json web token, Exited with error {}", xer);
            None
        }
    }

}

pub async fn validate_token(req: actix_web::dev::ServiceRequest, brearer: BearerAuth) -> Result<actix_web::dev::ServiceRequest, (actix_web::error::Error, actix_web::dev::ServiceRequest)> {
    let secret_key: String = match std::env::var("SECRET_KEY") {
        Ok(xsk) => xsk,
        Err(xer) => {
            let err: String = format!("Error! cannot get secret key, Exited with error {}", xer);
            let xer: actix_web::error::Error = actix_web::error::ErrorNotFound(err);
            return Err((xer, req));
        }
    };

    let client: &actix_web::web::Data<reqwest::Client> = if let Some(xad) = req.app_data() { xad } else {
        let xer: actix_web::error::Error = actix_web::error::ErrorNotFound("Client not found");
        return Err((xer, req))
    };

    let claims: Result<jsonwebtoken::TokenData<Claims>, jsonwebtoken::errors::Error> = jsonwebtoken::decode(
        brearer.token(), &jsonwebtoken::DecodingKey::from_secret(secret_key.as_bytes()), &jsonwebtoken::Validation::default()
    );

    match claims {
        Ok(xcl) => {
            let claims: Claims = xcl.claims;

            let curr_time: u64 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(xct) => xct.as_secs(),
                Err(xer) => {
                    let xer: actix_web::error::Error = actix_web::error::ErrorInternalServerError(xer);
                    return Err((xer, req));
                }
            };

            if curr_time > claims.get_exp() {
                let xer: actix_web::error::Error = actix_web::error::ErrorRequestTimeout("Current time exceeded token time");
                return Err((xer, req));
            }

            let is_present: bool = check_consumer(&client, claims.get_sub()).await;

            if is_present {
                req.extensions_mut().insert(claims);
                Ok(req)
            } else {
                let xer: String = format!("Consumer with consumer name {} is not found", claims.get_sub());
                let err: actix_web::error::Error = actix_web::error::ErrorNotFound(xer);
                Err((err, req))
            }
        }
        Err(xer) => {
            let xer: actix_web::error::Error = actix_web::error::ErrorInternalServerError(xer);
            Err((xer, req))
        }
    }

}
use actix_web::{web, HttpResponse, Responder};
use chrono::prelude::*;
use rand::Rng;
use sha2::{Sha256, Digest};
use log::{info};
use serde_json::json;
use crate::redis_control;


pub async fn generate_token(path: web::Path<(String, String)>) -> impl Responder {
    let (date, token_type) = path.into_inner();

    // 날짜 검증
    if !is_valid_date(&date) {
        info!("Invalid date provided: {}", date);
        return HttpResponse::BadRequest().body("Invalid request parameters");
    }

    // 타입 검증
    if !is_valid_token_type(&token_type) {
        info!("Invalid token type provided: {}", token_type);
        return HttpResponse::BadRequest().body("Invalid request parameters");
    }

    // 토큰 생성
    match create_token(&date, &token_type) {
        Ok((json_result, token)) => {
            match redis_control::store_token(&token) {
                Ok(_) => HttpResponse::Ok().body(json_result),
                Err(e) => {
                    info!("Failed to store token in Redis: {}", e);
                    HttpResponse::InternalServerError().body("Failed to store generated token")
                }
            }
        },
        Err(e) => {
            info!("Failed to generate token: {}", e);
            HttpResponse::InternalServerError().body("An error occurred while processing your request")
        }
    }
}

fn is_valid_date(date: &str) -> bool {
    let today = Local::now().format("%Y%m%d").to_string();
    let yesterday = (Local::now() - chrono::Duration::days(1)).format("%Y%m%d").to_string();
    
    date == today || date == yesterday
}

fn is_valid_token_type(token_type: &str) -> bool {
    matches!(token_type, "good" | "soso" | "bad")
}


pub fn create_token(date: &str, token_type: &str) -> Result<(String, String), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let random_part: u64 = rng.gen();
    
    let mut hasher = Sha256::new();
    hasher.update(date);
    hasher.update(token_type);
    hasher.update(random_part.to_string());
    let result = hasher.finalize();

    let token = format!("{}{:x}", date, result)[..40].to_string();

    let json_result = json!({
        "token": token
    }).to_string();

    Ok((json_result, token))
}


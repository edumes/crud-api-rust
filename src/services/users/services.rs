use super::models::{RegisterUser, UpdateUser, UserStruct};
use crate::AppState;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use bcrypt::{hash, DEFAULT_COST};

#[get("/users")]
async fn all_users(data: web::Data<AppState>) -> impl Responder {
    let result = sqlx::query!("SELECT * FROM users")
        .fetch_all(&data.postgres_client)
        .await;

    match result {
        Ok(users) => HttpResponse::Ok().json(
            users
                .iter()
                .map(|user| UserStruct {
                    id: user.id,
                    name: user.name.clone(),
                    email: user.email.clone(),
                    password: user.password.clone(),
                })
                .collect::<Vec<UserStruct>>(),
        ),
        Err(e) => {
            println!("Error: {}", e);
            HttpResponse::InternalServerError().body("Error")
        }
    }
}

#[post("/users")]
async fn create_user(data: web::Data<AppState>, user: web::Json<RegisterUser>) -> impl Responder {
    let hashed = hash(&user.password, DEFAULT_COST).expect("Failed to hash password");

    if !(user.email != "") {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"message": "Email is required"}));
    }
    if !(user.name != "") {
        return HttpResponse::BadRequest().json(serde_json::json!({"message": "Name is required"}));
    }
    if !(user.password != "") {
        return HttpResponse::BadRequest()
            .json(serde_json::json!({"message": "Password is required"}));
    }

    if !(hashed != user.password) {
        return HttpResponse::InternalServerError().body("Error hashing password");
    }

    let result = sqlx::query!(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3) RETURNING *",
        user.name,
        user.email,
        hashed
    )
    .fetch_one(&data.postgres_client)
    .await;

    match result {
        Ok(result_query) => {
            let user = UserStruct {
                id: result_query.id,
                name: result_query.name,
                email: result_query.email,
                password: result_query.password,
            };
            HttpResponse::Created().json(serde_json::json!({
                "message": "User created",
                "data": user
            }))
        }
        Err(e) => {
            if e.to_string().contains("users_email_key") {
                return HttpResponse::Conflict().json(serde_json::json!({
                    "message": "Email already exists"
                }));
            }
            println!("Error: {}", e);
            HttpResponse::InternalServerError().json(serde_json::json!({
                "message": format!("Error: {}", e)
            }))
        }
    }
}

#[put("/users/{id}")]
async fn update_user(
    data: web::Data<AppState>,
    user: web::Json<UpdateUser>,
    id: web::Path<i32>,
) -> impl Responder {
    let result = sqlx::query!(
        "UPDATE users SET name = $1, email = $2, password = $3 WHERE id = $4",
        user.name,
        user.email,
        user.password,
        id.into_inner()
    )
    .execute(&data.postgres_client)
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User updated"),
        Err(e) => {
            println!("Error: {}", e);
            HttpResponse::InternalServerError().body("Error")
        }
    }
}

#[delete("/users/{id}")]
async fn delete_user(data: web::Data<AppState>, id: web::Path<i32>) -> impl Responder {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id.into_inner())
        .execute(&data.postgres_client)
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(e) => {
            println!("Error: {}", e);
            HttpResponse::InternalServerError().body("Error")
        }
    }
}

pub fn users_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(all_users)
        .service(create_user)
        .service(update_user)
        .service(delete_user);
}
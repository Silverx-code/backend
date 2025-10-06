use crate::auth::{jwt, models::*, validation};
use bcrypt::{hash, verify, DEFAULT_COST};
use deadpool_postgres::Pool;
use validator::Validate;
use warp::Reply;

pub async fn signup_handler(
    signup_req: SignupRequest,
    db_pool: Pool,
) -> Result<impl Reply, warp::Rejection> {
    // Validate input
    if let Err(validation_errors) = signup_req.validate() {
        let errors: Vec<String> = validation_errors
            .field_errors()
            .iter()
            .flat_map(|(field, errors)| {
                errors.iter().map(move |error| {
                    format!("{}: {}", field, error.message.clone().unwrap_or_default())
                })
            })
            .collect();

        let error_response = ErrorResponse {
            error: "Validation failed".to_string(),
            details: Some(errors),
        };

        return Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get database connection
    let client = match db_pool.get().await {
        Ok(client) => client,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "Database connection failed".to_string(),
                details: None,
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Check if username already exists
    let username_check = client
        .query(
            "SELECT id FROM users WHERE username = $1",
            &[&signup_req.username],
        )
        .await;

    if let Ok(rows) = username_check {
        if !rows.is_empty() {
            let error_response = ErrorResponse {
                error: "Username already taken".to_string(),
                details: None,
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::CONFLICT,
            ));
        }
    }

    // Check if email already exists
    let email_check = client
        .query("SELECT id FROM users WHERE email = $1", &[&signup_req.email])
        .await;

    if let Ok(rows) = email_check {
        if !rows.is_empty() {
            let error_response = ErrorResponse {
                error: "Email already registered".to_string(),
                details: None,
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::CONFLICT,
            ));
        }
    }

    // Hash password
    let password_hash = match hash(&signup_req.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "Failed to hash password".to_string(),
                details: None,
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Insert user into database
    let insert_result = client
        .query_one(
            "INSERT INTO users (username, email, password_hash) VALUES ($1, $2, $3) RETURNING id, username, email, created_at",
            &[&signup_req.username, &signup_req.email, &password_hash],
        )
        .await;

    match insert_result {
        Ok(row) => {
            let user_id: i32 = row.get(0);
            let username: String = row.get(1);
            let email: String = row.get(2);
            let created_at: chrono::NaiveDateTime = row.get(3);
            let created_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(created_at, chrono::Utc);
            // Generate JWT token
            let token = match jwt::create_jwt(user_id, username.clone(), email.clone()) {
                Ok(token) => token,
                Err(_) => {
                    let error_response = ErrorResponse {
                        error: "Failed to generate token".to_string(),
                        details: None,
                    };
                    return Ok(warp::reply::with_status(
                        warp::reply::json(&error_response),
                        warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                    ));
                }
            };

            let response = AuthResponse {
                token,
                user: UserResponse {
                    id: user_id,
                    username,
                    email,
                    created_at,
                },
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&response),
                warp::http::StatusCode::CREATED,
            ))
        }
        Err(_) => {
            let error_response = ErrorResponse {
                error: "Failed to create user".to_string(),
                details: None,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}

pub async fn login_handler(
    login_req: LoginRequest,
    db_pool: Pool,
) -> Result<impl Reply, warp::Rejection> {
    // Validate input
    if let Err(_) = login_req.validate() {
        let error_response = ErrorResponse {
            error: "Invalid input".to_string(),
            details: None,
        };
        return Ok(warp::reply::with_status(
            warp::reply::json(&error_response),
            warp::http::StatusCode::BAD_REQUEST,
        ));
    }

    // Get database connection
    let client = match db_pool.get().await {
        Ok(client) => client,
        Err(_) => {
            let error_response = ErrorResponse {
                error: "Database connection failed".to_string(),
                details: None,
            };
            return Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // Find user by username or email
    let user_result = client
        .query_one(
            "SELECT id, username, email, password_hash, created_at FROM users WHERE username = $1 OR email = $1",
            &[&login_req.username_or_email],
        )
        .await;

    match user_result {
        Ok(row) => {
            let user_id: i32 = row.get(0);
            let username: String = row.get(1);
            let email: String = row.get(2);
            let password_hash: String = row.get(3);
            let created_at: chrono::NaiveDateTime = row.get(4);
            let created_at = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(created_at, chrono::Utc);

            // Verify password
            match verify(&login_req.password, &password_hash) {
                Ok(is_valid) if is_valid => {
                    // Update last login
                    let _ = client
                        .execute(
                            "UPDATE users SET last_login = NOW() WHERE id = $1",
                            &[&user_id],
                        )
                        .await;

                    // Generate JWT token
                    let token = match jwt::create_jwt(user_id, username.clone(), email.clone()) {
                        Ok(token) => token,
                        Err(_) => {
                            let error_response = ErrorResponse {
                                error: "Failed to generate token".to_string(),
                                details: None,
                            };
                            return Ok(warp::reply::with_status(
                                warp::reply::json(&error_response),
                                warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                            ));
                        }
                    };

                    let response = AuthResponse {
                        token,
                        user: UserResponse {
                            id: user_id,
                            username,
                            email,
                            created_at,
                        },
                    };

                    Ok(warp::reply::with_status(
                        warp::reply::json(&response),
                        warp::http::StatusCode::OK,
                    ))
                }
                _ => {
                    let error_response = ErrorResponse {
                        error: "Invalid credentials".to_string(),
                        details: None,
                    };
                    Ok(warp::reply::with_status(
                        warp::reply::json(&error_response),
                        warp::http::StatusCode::UNAUTHORIZED,
                    ))
                }
            }
        }
        Err(_) => {
            let error_response = ErrorResponse {
                error: "Invalid credentials".to_string(),
                details: None,
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&error_response),
                warp::http::StatusCode::UNAUTHORIZED,
            ))
        }
    }
}
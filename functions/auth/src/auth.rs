use aws_sdk_dynamodb;
use lambda_http::{Body, IntoResponse, Response};
use serde_json::json;
use std::env;

use model;

pub async fn login(dynamodb: &aws_sdk_dynamodb::Client, email: &str) -> Response<Body> {
    let user = model::User::from_email(dynamodb, &email).await;
    println!("User record found by email: {:?}", user);
    match user {
        None => format!("login request success").into_response(),
        Some(user) => {
            let auth_session_id = user.create_auth_session(dynamodb).await;
            println!("AUTHSESSION#{:?} created for {}", auth_session_id, user.id);
            // TODO: Send email with code
            format!("login request success").into_response()
        }
    }
}

pub async fn authenticate(
    dynamodb: &aws_sdk_dynamodb::Client,
    email: &str,
    code: &str,
) -> Response<Body> {
    let user_record = model::User::from_email(dynamodb, email).await;
    // TODO: Add check to see if user is active

    match user_record {
        None => format!("Authentication failed").into_response(),
        Some(user) => {
            let auth_sessions_response = user.get_auth_sessions(dynamodb).await;
            match auth_sessions_response {
                Some(auth_sessions) => {
                    let matched_auth_session =
                        auth_sessions.iter().find(|&auth_session| auth_session.code == code);
                    match matched_auth_session {
                        Some(auth_session) => {
                            // TODO: delete auth session and create session in a db transaction
                            auth_session.delete(dynamodb).await;
                            let session = user.create_session(dynamodb, None).await;
                            Response::builder()
                                .status(200)
                                .header(
                                    "Set-Cookie",
                                    format!(
                                        "session_id={}; Domain={}; Secure; HttpOnly; Expires={}",
                                        session.id,
                                        env::var("HOSTED_ZONE").unwrap(),
                                        session.expiry.to_rfc2822()
                                    ),
                                )
                                .header(
                                    "Set-Cookie",
                                    format!(
                                        "csrf-token={}; Domain={}; Secure; Expires={}",
                                        session.csrf_token,
                                        env::var("HOSTED_ZONE").unwrap(),
                                        session.expiry.to_rfc2822()
                                    ),
                                )
                                .body(json!({"csrf_token": session.csrf_token}).to_string().into())
                                .unwrap()
                        }
                        None => format!("Authentication failed").into_response(),
                    }
                }
                None => format!("Authentication failed").into_response(),
            }
        }
    }
}

use crate::core::{Session, User};
use crate::data::Database;
use crate::AppState;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::info;

pub async fn auth<D: Database>(
    State(state): State<AppState<D>>,
    mut request: Request,
    next: Next,
) -> Response {
    // Paths that can be accessed by anonymous users
    let anonymous_paths: Vec<&str> = vec!["/datasets"];
    let path: &str = request.uri().path();

    let allow_anonymous = anonymous_paths
        .iter()
        .any(|prefix| path.starts_with(prefix));
    let allow_anonymous = allow_anonymous && request.method().as_str() == "GET";

    let auth_header_value = request.headers().get("Authorization");

    let token = if let Some(value) = auth_header_value {
        let value = value.to_str().unwrap();
        if value.starts_with("Bearer ") {
            Some(value.trim_start_matches("Bearer ").to_string())
        } else {
            None
        }
    } else {
        None
    };

    if token.is_none() && allow_anonymous {
        info!("New Anonymous Access: Creating anonymous session");
        let session = Session::create(state.db, None).await.unwrap();
        let mut response = next.run(request).await;
        response.headers_mut().insert(
            "Authorization",
            format!("Bearer {}", session.id).parse().unwrap(),
        );

        return (StatusCode::OK, response).into_response();
    }

    if token.is_none() && !allow_anonymous {
        return (StatusCode::UNAUTHORIZED, "UNAUTHORIZED").into_response();
    }

    let token = token.unwrap();
    let session = match Session::from_id(state.db.clone(), &token).await {
        Ok(s) => s,
        Err(_) => return (StatusCode::UNAUTHORIZED, "UNAUTHORIZED").into_response(),
    };

    request.extensions_mut().insert(session.clone());
    match session.user_id {
        Some(user_id) => {
            info!("User ID found in session: {}", user_id);
            let user_response = User::from_id(state.db, &user_id).await;

            match user_response {
                Ok(user) => {
                    request.extensions_mut().insert(user);
                }
                Err(_) => {
                    info!("User not found");
                    return (StatusCode::UNAUTHORIZED, "UNAUTHORIZED").into_response();
                }
            }
        }
        None => {
            info!("Exisiting Anonymous Session");
            let response = next.run(request).await;
            return (StatusCode::OK, response).into_response();
        }
    }
    let response = next.run(request).await;
    response.into_response()
}

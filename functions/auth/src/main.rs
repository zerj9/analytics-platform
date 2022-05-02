mod auth;
use aws_sdk_dynamodb;
use lambda_http::{service_fn, Body, Error, Request, RequestExt, Response};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let func = service_fn(|req| handler(req, &client));
    lambda_http::run(func).await?;
    Ok(())
}

enum QueryType {
    Login,
    Authenticate,
    Unknown,
}

impl FromStr for QueryType {
    type Err = ();

    fn from_str(input: &str) -> Result<QueryType, Self::Err> {
        match input {
            "login" => Ok(QueryType::Login),
            "authenticate" => Ok(QueryType::Authenticate),
            _ => Ok(QueryType::Unknown),
        }
    }
}

struct QueryParameters<'a> {
    query_type: QueryType,
    email: Option<&'a str>,
    auth_session_id: Option<&'a str>,
    code: Option<&'a str>,
}

async fn handler(
    event: Request,
    client: &aws_sdk_dynamodb::Client,
) -> Result<Response<Body>, Error> {
    let request_query_string_parameters = event.query_string_parameters();
    let query_parameters = QueryParameters {
        query_type: event.query_string_parameters().first("type").unwrap().parse().unwrap(),
        email: request_query_string_parameters.first("email"),
        auth_session_id: request_query_string_parameters.first("auth_session_id"),
        code: request_query_string_parameters.first("code"),
    };

    match query_parameters {
        QueryParameters {
            query_type: QueryType::Login,
            email: Some(email),
            auth_session_id: None,
            code: None,
        } => {
            println!("Login request for: {}", email);
            Ok(auth::login(&client, query_parameters.email.unwrap()).await)
        }
        QueryParameters {
            query_type: QueryType::Authenticate,
            email: Some(email),
            auth_session_id: Some(auth_session_id),
            code: Some(code),
        } => Ok(auth::authenticate(&client, email, auth_session_id, code).await),
        _ => Ok(Response::builder()
            .status(400)
            .body("bad request".into())
            .expect("failed to render response")),
    }
}

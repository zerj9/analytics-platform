use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

use model;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let shared_config = aws_config::load_from_env().await;
    let client = aws_sdk_dynamodb::Client::new(&shared_config);
    let func = service_fn(|req| handler(req, &client));
    lambda_runtime::run(func).await?;
    Ok(())
}

async fn handler(
    event: LambdaEvent<Value>,
    client: &aws_sdk_dynamodb::Client,
) -> Result<Value, Error> {
    let (event, _context) = event.into_parts();

    let cookies_json = event["cookies"].as_array();

    if let Some(cookies) = cookies_json {
        let cookies = cookies.to_vec();

        let session_cookie =
            cookies.iter().find(|&cookie| cookie.as_str().unwrap().starts_with("session_id="));

        match session_cookie {
            None => Ok(json!({ "isAuthorized": false })),
            Some(cookie) => {
                let session_id = cookie.as_str().unwrap().strip_prefix("session_id=").unwrap();
                let session = model::Session::from_id(client, session_id).await;
                // let user = model::User::from_id; IMPLEMENT from_id
                println!("session: {:?}", session.unwrap());
                Ok(json!({ "isAuthorized": true }))
            }
        }
    } else {
        Ok(json!({ "isAuthorized": false }))
    }
}

use lambda_runtime::{service_fn, Error, LambdaEvent};
use serde_json::{json, Value};

use model::{Session, User};

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
    dynamodb: &aws_sdk_dynamodb::Client,
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
                let session = Session::from_id(dynamodb, session_id).await;

                if let Some(sess) = session {
                    let user = User::from_id(dynamodb, &sess.user_id).await.unwrap();
                    println!("session: {:?} for user {:?}", sess, user);
                    Ok(json!({
                       "isAuthorized": true,
                       "context": {
                           "user_id": user.id,
                           "user_email": user.email
                       }
                    }))
                } else {
                    Ok(json!({ "isAuthorized": false }))
                }
            }
        }
    } else {
        Ok(json!({ "isAuthorized": false }))
    }
}

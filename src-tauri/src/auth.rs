use std::{error::Error, string::ParseError, sync::{mpsc, Arc, Mutex}, thread, time::Duration};

use google_drive::{Client, AccessToken};
use hyper::{Request, Body, Response, service::{make_service_fn, service_fn}, Server};
use tokio::sync::oneshot::{self, Sender};
#[derive(Debug)]
struct TokenStruct {
    pub state: Option<String>,
    pub code: Option<String>,
}

pub(crate) struct GDStruct {
    pub token: AccessToken,
    pub drive: Client
}

async fn handle_request(req: Request<Body>, tokens: Arc<Mutex<TokenStruct>>) -> Result<Response<Body>, hyper::Error> {
    let url = req.uri();
    let query_params: Vec<_> = url.query().unwrap_or_default().split('&').collect();

    let mut tokens_guard = tokens.lock().unwrap();

    for param in query_params {
        let key_value: Vec<_> = param.split('=').collect();
        if key_value.len() == 2 {
            match key_value[0] {
                "state" => tokens_guard.state = Some(key_value[1].to_string()),
                "code" => tokens_guard.code = Some(key_value[1].to_string()),
                _ => {}
            }
        }
    }

    Ok(Response::new(Body::from("Token received. You can now close the browser.")))
}

pub(crate) async fn auth() -> GDStruct {
    // Create a token structure to store state and code
    let token_struct = Arc::new(Mutex::new(TokenStruct {
        state: None,
        code: None,
    }));

    // Clone the Arc to pass to the server closure
    let tokens = token_struct.clone();

    // Define the server address
    let addr = ([127, 0, 0, 1], 0).into();

    // Create a make_service closure to build the server
    let make_svc = make_service_fn(move |_conn| {
        let tokens = tokens.clone();
        async {
            Ok::<_, hyper::Error>(service_fn(move |req| {
                handle_request(req, tokens.clone())
            }))
        }
    });

    // Create the server
    let server = Server::bind(&addr).serve(make_svc);
    let actual_addr = server.local_addr().to_string();

    let client = reqwest::Client::new();

    let state = uuid::Uuid::new_v4();

    let user_consent_url = client.get("https://entangleauth.eeshwar-krishnan.workers.dev/auth")
        .query(&[("redirect", &format!("http://{}", actual_addr)), ("status", &format!("{}", state))]).send().await.unwrap().text().await.unwrap();

    println!("{}", user_consent_url);

    webbrowser::open(&user_consent_url).unwrap();


    // Run the server in the background
    let server_handle = tokio::spawn(server);

    // Wait for the server to start
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let mut end = false;
    while !end {
        let val = token_struct.lock().unwrap();
        if val.state.is_some() {
            end = true;
        }
        std::mem::drop(val);
        thread::sleep(Duration::from_secs(1));
    }

    // Return the token struct
    let val = token_struct.lock().unwrap();

    let access_token = client.get("https://entangleauth.eeshwar-krishnan.workers.dev/confirm")
        .query(&[("redirect", &format!("http://{}", actual_addr)), ("status", &val.state.clone().unwrap()), ("code", &val.code.clone().unwrap())]).send().await.unwrap().json::<AccessToken>().await.unwrap();

    println!("{:?}", access_token);

    let mut google_drive = Client::new("", 
    "", format!("http://{}", actual_addr), &access_token.access_token, &access_token.refresh_token); 

    return GDStruct {
        token: access_token,
        drive: google_drive,
    }
}
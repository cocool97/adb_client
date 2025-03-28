use crate::error::Result;
use base64::{prelude::BASE64_STANDARD, Engine};
use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize)]
struct InternalSignResponse {
    token: String,
    public_key: String,
}

pub struct SignResponse {
    pub token: Vec<u8>,
    pub public_key: Vec<u8>,
}

pub fn perform_remote_auth(auth_message: Vec<u8>, remote_auth_url: &str) -> Result<SignResponse> {
    println!("Requesting auth from {remote_auth_url}");
    let http_client = Client::new();
    let response = http_client
        .post(remote_auth_url)
        .body(auth_message)
        .send()?
        .json::<InternalSignResponse>()?;

    let token = BASE64_STANDARD.decode(response.token)?;

    Ok(SignResponse {
        token,
        public_key: response.public_key.into_bytes(),
    })
}

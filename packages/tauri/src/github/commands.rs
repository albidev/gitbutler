use std::collections::HashMap;
use tauri::{AppHandle, Manager};

use crate::error::Error;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const GITHUB_CLIENT_ID: &str = "cd51880daa675d9e6452";

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Verification {
    pub user_code: String,
    pub device_code: String,
}

#[tauri::command(async)]
#[instrument]
pub async fn init_device_oauth() -> Result<Verification, Error> {
    let mut req_body = HashMap::new();
    req_body.insert("client_id", GITHUB_CLIENT_ID);
    req_body.insert("scope", "repo");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://github.com/login/device/code")
        .headers(headers)
        .json(&req_body)
        .send()
        .await;

    match res {
        Ok(res) => {
            let rsp_body = res.text().await;
            match rsp_body {
                Ok(rsp_body) => {
                    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
                    struct CodesContainer {
                        user_code: String,
                        device_code: String,
                    }
                    match serde_json::from_str::<CodesContainer>(&rsp_body) {
                        Ok(container) => {
                            return Ok(Verification {
                                user_code: container.user_code,
                                device_code: container.device_code,
                            })
                        }
                        Err(_) => {
                            return Err(Error::Unknown);
                        }
                    }
                }
                Err(_) => {
                    return Err(Error::Unknown);
                }
            }
        }
        Err(_) => Err(Error::Unknown),
    }
}

#[tauri::command(async)]
#[instrument]
pub async fn check_auth_status(device_code: &str) -> Result<String, Error> {
    let mut req_body = HashMap::new();
    req_body.insert("client_id", GITHUB_CLIENT_ID);
    req_body.insert("device_code", device_code);
    req_body.insert("grant_type", "urn:ietf:params:oauth:grant-type:device_code");

    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/json"),
    );

    let client = reqwest::Client::new();
    let res = client
        .post("https://github.com/login/oauth/access_token")
        .headers(headers)
        .json(&req_body)
        .send()
        .await;

    match res {
        Ok(res) => {
            let rsp_body = res.text().await;
            match rsp_body {
                Ok(rsp_body) => {
                    #[derive(Debug, Deserialize, Serialize, Clone, Default)]
                    struct AccessTokenContainer {
                        access_token: String,
                    }
                    match serde_json::from_str::<AccessTokenContainer>(&rsp_body) {
                        Ok(rsp_body) => {
                            return Ok(rsp_body.access_token);
                        }
                        Err(_) => {
                            return Err(Error::Unknown);
                        }
                    }
                }
                Err(_) => {
                    return Err(Error::Unknown);
                }
            }
        }
        Err(_) => Err(Error::Unknown),
    }
}

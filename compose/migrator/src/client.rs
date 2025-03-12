use std::{error::Error, fmt::Display};

use log::{debug, error};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    IntoUrl, Method,
};
use serde_json::Value;

pub struct Client {
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-type",
            HeaderValue::from_static("application/fhir+json"),
        );

        Self {
            client: reqwest::blocking::Client::builder()
                .default_headers(headers)
                .build()
                .unwrap(),
        }
    }
    pub fn get<T>(&self, url: T) -> Result<Value, Box<dyn Error>>
    where
        T: IntoUrl + Display,
    {
        let request = self.client.request(Method::GET, url).build().unwrap();

        self.execute(request)
    }

    #[allow(dead_code)]
    pub fn post<T>(&self, url: T, data: &Value) -> Result<Value, Box<dyn Error>>
    where
        T: IntoUrl + Display,
    {
        let json = serde_json::to_string_pretty(&data).unwrap();

        let request = self
            .client
            .request(Method::POST, url)
            .body(json)
            .build()
            .unwrap();

        self.execute(request)
    }

    pub fn put<T>(&self, url: T, value: &Value) -> Result<Value, Box<dyn Error>>
    where
        T: IntoUrl + Display,
    {
        let json = serde_json::to_string(value).unwrap();

        let request = self
            .client
            .request(Method::PUT, url)
            .body(json)
            .build()
            .unwrap();

        self.execute(request)
    }

    fn execute(&self, request: reqwest::blocking::Request) -> Result<Value, Box<dyn Error>> {
        debug!(
            "Request:\nmethod: {}\nurl: {}\nbody: {}",
            request.method().to_string(),
            request.url().to_string(),
            request
                .body()
                .and_then(|b| { b.as_bytes() })
                .and_then(|b| { String::from_utf8(b.to_vec()).ok() })
                .unwrap_or(String::from(""))
        );

        match self.client.execute(request) {
            Ok(response) => {
                let status = response.status();
                let url = response.url().clone();
                let body: Value = response.json().unwrap();

                debug!(
                    "Response:\nstatus: {}\nurl: {}\nbody: {}",
                    status, url, body
                );

                if !status.is_success() {
                    error!("");
                    return Err(body.to_string().into());
                }

                Ok(body)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn delete<T>(&self, url: T) -> Result<Value, Box<dyn Error>>
    where
        T: IntoUrl + Display,
    {
        let request = self.client.request(Method::DELETE, url).build().unwrap();
        self.execute(request)
    }
}

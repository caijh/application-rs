use std::error::Error;
use std::fmt::Display;

use askama::Template;
use axum::body::Body;
use axum::http::{StatusCode, Uri};
use axum::response::{Html, IntoResponse, Response};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

pub const CODE_SUCCESS: i8 = 0;
pub const CODE_FAILURE: i8 = -1;

#[derive(Debug, Serialize, Deserialize)]
pub struct RespBody<T> {
    pub code: Option<i8>,
    pub msg: Option<String>,
    pub data: Option<T>,
}

impl<T> RespBody<T>
where
    T: Serialize + DeserializeOwned + Clone,
{
    pub fn result(arg: &Result<T, Box<dyn Error>>) -> Self {
        match arg {
            Ok(r) => Self {
                code: Some(CODE_SUCCESS),
                msg: None,
                data: Some(r.to_owned()),
            },
            Err(e) => Self {
                code: Some(CODE_FAILURE),
                msg: Some(e.to_string()),
                data: None,
            },
        }
    }

    pub fn error(arg: &dyn Error) -> Self {
        Self {
            code: Some(CODE_FAILURE),
            msg: Some(arg.to_string()),
            data: None,
        }
    }

    pub fn success(arg: &T) -> Self {
        Self {
            code: Some(CODE_SUCCESS),
            msg: None,
            data: Some(arg.clone()),
        }
    }

    pub fn code_info(code: i8, info: &str) -> Self {
        Self {
            code: Some(code),
            msg: Some(info.to_string()),
            data: None,
        }
    }

    pub fn fail_info(info: &str) -> Self {
        Self::code_info(CODE_FAILURE, info)
    }

    pub fn success_info(info: &str) -> Self {
        Self::code_info(CODE_SUCCESS, info)
    }

    pub fn response(&self) -> Response {
        Response::builder()
            .extension(|| {})
            .header("Access-Control-Allow-Origin", "*")
            .header("Cache-Control", "no-cache")
            .header("Content-Type", "application/json;charset=UTF-8")
            .body(Body::from(self.to_string()))
            .unwrap()
    }
}

impl<T> IntoResponse for RespBody<T>
where
    T: Serialize + DeserializeOwned,
{
    fn into_response(self) -> Response {
        self.response()
    }
}

impl<T> Display for RespBody<T>
where
    T: Serialize + DeserializeOwned,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(self).unwrap())
    }
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {err}"),
            )
                .into_response(),
        }
    }
}

pub async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}

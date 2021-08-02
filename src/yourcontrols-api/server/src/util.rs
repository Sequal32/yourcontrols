use std::convert::Infallible;

use anyhow::Result;
use serde::Serialize;
use warp::{
    filters::BoxedFilter,
    hyper::StatusCode,
    reject::{MethodNotAllowed, Reject},
    Filter, Rejection, Reply,
};

use crate::models::InternalError;

#[derive(Debug)]
pub struct AnyhowError {
    inner: anyhow::Error,
}

impl Reject for AnyhowError {}

impl AnyhowError {
    pub fn into_reject(e: anyhow::Error) -> Rejection {
        warp::reject::custom(Self { inner: e })
    }

    pub fn inner(&self) -> &anyhow::Error {
        &self.inner
    }
}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let reason: String;
    let mut code = StatusCode::INTERNAL_SERVER_ERROR;

    if let Some(e) = err.find::<AnyhowError>() {
        reason = e.inner().to_string();
    } else if err.is_not_found() {
        reason = "NOT_FOUND".to_string();
        code = StatusCode::NOT_FOUND;
    } else if let Some(_) = err.find::<MethodNotAllowed>() {
        reason = "METHOD NOT ALLOWED".to_string();
        code = StatusCode::METHOD_NOT_ALLOWED;
    } else if let Some(e) = err.find::<InternalError>() {
        return Ok(warp::reply::with_status(
            warp::reply::json(e),
            StatusCode::from_u16(e.code).unwrap(),
        ));
    } else {
        reason = "NO REASON".to_string();
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&InternalError {
            reason,
            code: code.as_u16(),
        }),
        code,
    ))
}

pub fn with_auth() -> BoxedFilter<()> {
    warp::any()
        .and(warp::header::optional::<String>("authorization"))
        .and_then(handle_auth)
        .untuple_one()
        .boxed()
}

async fn handle_auth(auth: Option<String>) -> Result<(), warp::reject::Rejection> {
    if let Some(auth) = auth {
        if auth == dotenv::var("API_KEY").expect("should be set") {
            return Ok(());
        }
    }
    Err(unauthorized())
}

fn unauthorized() -> warp::reject::Rejection {
    AnyhowError::into_reject(anyhow::anyhow!("Unauthorized"))
}

pub fn resolve_or_reject_payload<T: Serialize>(
    data: Result<T>,
) -> Result<impl warp::Reply, warp::Rejection> {
    match data {
        Ok(d) => Ok(warp::reply::json(&d)),
        Err(e) => Err(AnyhowError::into_reject(e)),
    }
}

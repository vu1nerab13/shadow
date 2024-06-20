use super::Parameter;
use crate::network::ServerObj;
use anyhow::Result as AppResult;
use serde::{Deserialize, Serialize};
use shadow_common::{client as sc, error::ShadowError};
use std::{str::FromStr, sync::Arc};
use strum_macros::EnumString;
use tokio::sync::RwLock;
use warp::{
    http::StatusCode,
    reply::{self, Reply},
};

#[derive(EnumString, Deserialize, Serialize)]
pub enum QueryOperation {
    #[strum(ascii_case_insensitive)]
    Summary,
}

#[derive(Deserialize, Serialize)]
pub struct QueryParameter {
    op: String,
}

impl Parameter for QueryParameter {
    type Operation = QueryOperation;

    fn operation(&self) -> AppResult<Self::Operation> {
        Ok(Self::Operation::from_str(&self.op)?)
    }

    fn summarize() -> String {
        "query operation".into()
    }

    async fn dispatch(
        &self,
        op: Self::Operation,
        server_obj: Arc<RwLock<ServerObj>>,
    ) -> Result<Box<dyn Reply>, ShadowError> {
        match op {
            QueryOperation::Summary => summarize_client(server_obj).await,
        }
    }
}

async fn summarize_client(
    server_obj: Arc<RwLock<ServerObj>>,
) -> Result<Box<dyn Reply>, ShadowError> {
    #[derive(Serialize, Deserialize)]
    struct GetIpReply {
        status: String,
        message: Option<String>,
        country: Option<String>,
        #[serde(rename(serialize = "country_code", deserialize = "countryCode"))]
        country_code: Option<String>,
        region: Option<String>,
        #[serde(rename(serialize = "region_name", deserialize = "regionName"))]
        region_name: Option<String>,
        city: Option<String>,
        zip: Option<String>,
        lat: Option<String>,
        lon: Option<String>,
        timezone: Option<String>,
        isp: Option<String>,
        org: Option<String>,
        r#as: Option<String>,
        query: String,
    }

    #[derive(Serialize, Deserialize)]
    struct Summary {
        ip: GetIpReply,
        info: sc::SystemInfo,
    }

    let server_obj = server_obj.read().await;
    let ip = reqwest::get(format!("http://ip-api.com/json/{}", server_obj.get_ip()))
        .await?
        .json::<GetIpReply>()
        .await?;

    Ok(Box::new(reply::with_status(
        reply::json(&Summary {
            ip,
            info: server_obj.summary(),
        }),
        StatusCode::OK,
    )))
}

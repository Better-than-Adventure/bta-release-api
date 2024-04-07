use core::num;
use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Query, State}, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use crate::{db::ReleaseDatabase, release::{Artifact, Release, ReleaseChannel, Repository}};

const DB_PATH: &'static str = "./releases.db3";

pub struct Api {
    repository: Repository
}

impl Api {
    pub fn new() -> Api {
        Self {
            repository: Repository::dummy()
        }
    }

    pub async fn run(self) {
        let shared_state = Arc::new(self);
        let app = Router::new()
            .route("/api", get(Self::api))
            .with_state(shared_state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    async fn api(
        State(state): State<Arc<Api>>,
        Query(query): Query<HashMap<String, String>>,
    ) -> (StatusCode, Json<Response>) {
        // Open database
        let mut db = match ReleaseDatabase::new(DB_PATH) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        let mut channel: Option<u32> = None;
        let mut release: Option<u32> = None;
        let mut artifact: Option<u32> = None;

        for param in query {
            match param.0.as_str() {
                "channel" => if let Ok(result) = param.1.parse::<u32>() {
                    channel = Some(result);
                } else {
                    return (StatusCode::BAD_REQUEST, Json(Response { response_code: 2, data: ResponseData::None }))
                },
                "release" => if let Ok(result) = param.1.parse::<u32>() {
                    release = Some(result);
                } else {
                    return (StatusCode::BAD_REQUEST, Json(Response { response_code: 2, data: ResponseData::None }))
                },
                "artifact" => if let Ok(result) = param.1.parse::<u32>() {
                    artifact = Some(result);
                } else {
                    return (StatusCode::BAD_REQUEST, Json(Response { response_code: 2, data: ResponseData::None }))
                },
                _ => return (StatusCode::BAD_REQUEST, Json(Response { response_code: 1, data: ResponseData::None }))
            }
        }

        let mut num_params: u32 = 0;
        if channel.is_some() {
            num_params += 1;
        }
        if release.is_some() {
            num_params += 1;
        }
        if artifact.is_some() {
            num_params += 1;
        }

        if num_params > 1 {
            return (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::None }))
        }

        if let Some(channel) = channel {
            if let Ok(channel) = db.read_channel(channel) {
                return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::ReleaseChannel(channel) }))
            } else {
                return (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::None }))
            }
        } else if let Some(release) = release {
            if let Ok(release) = db.read_release(release) {
                return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Release(release) }))
            } else {
                return (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::None }))
            }
        } else if let Some(artifact) = artifact {
            if let Ok(artifact) = db.read_artifact(artifact) {
                return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Artifact(artifact) }))
            } else {
                return (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::None }))
            }
        } else {
            if let Ok(repository) = db.read_repository() {
                return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Repository(repository) }))
            } else {
                return (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::None }))
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Response {
    response_code: u8,
    data: ResponseData
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum ResponseData {
    None,
    Error(String),
    Repository(Repository),
    ReleaseChannel(ReleaseChannel),
    Release(Release),
    Artifact(Artifact)
}
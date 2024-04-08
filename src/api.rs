
use core::num;
use std::{collections::HashMap, sync::Arc};

use axum::{extract::{Path, Query, State}, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use crate::{db::ReleaseDatabase, release::{Artifact, Release, ReleaseChannel, Repository}};

const DB_PATH: &'static str = "./releases.db3";

pub struct Api {
}

impl Api {
    pub fn new() -> Api {
        Self {
        }
    }

    pub async fn run(self) {
        let shared_state = Arc::new(self);
        let app = Router::new()
            .route("/repositories/:repository", get(Self::repositories))
            .route("/repositories/:repository/channels/:channel", get(Self::repositories_channels))
            .route("/repositories/:repository/channels/:channel/releases/:release", get(Self::repositories_channels_releases))
            .route("/repositories/:repository/channels/:channel/releases/:release/artifacts/:artifact", get(Self::repositories_channels_releases_artifacts))
            .with_state(shared_state);

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    async fn repositories(
        State(state): State<Arc<Api>>,
        Path(repository): Path<String>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(DB_PATH) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_repository(repository) {
            Ok(repository) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Repository(repository) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn repositories_channels(
        State(state): State<Arc<Api>>,
        Path((repository, channel)): Path<(String, u32)>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(DB_PATH) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_channel(repository, channel) {
            Ok(channel) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::ReleaseChannel(channel) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn repositories_channels_releases(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release)): Path<(String, u32, u32)>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(DB_PATH) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_release(repository, channel, release) {
            Ok(release) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Release(release) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }    }

    async fn repositories_channels_releases_artifacts(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release, artifact)): Path<(String, u32, u32, u32)>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(DB_PATH) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_artifact(repository, channel, release, artifact) {
            Ok(artifact) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Artifact(artifact) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
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
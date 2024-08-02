

use std::sync::Arc;

use axum::{extract::{Path, State}, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use crate::{config::Config, db::ReleaseDatabase, release::{Artifact, Channel, Release, Repository}};

pub struct Api {
    config: Arc<Config>
}

impl Api {
    pub fn new(config: Config) -> Api {
        Self {
            config: Arc::new(config)
        }
    }

    pub async fn run(self) {
        let shared_state = Arc::new(self);
        let app = Router::new()
            .route("/:repository", get(Self::get_repositories))
            .route("/:repository/:channel", get(Self::get_repositories_channels))
            .route("/:repository/:channel/:release", get(Self::get_repositories_channels_releases))
            .route("/:repository/:channel/:release/:artifact", get(Self::get_repositories_channels_releases_artifacts))
            .with_state(shared_state.clone());

        let listener = tokio::net::TcpListener::bind(shared_state.config.bind_addr()).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    async fn get_repositories(
        State(state): State<Arc<Api>>,
        Path(repository): Path<String>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_repository(repository) {
            Ok(repository) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Repository(repository) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn get_repositories_channels(
        State(state): State<Arc<Api>>,
        Path((repository, channel)): Path<(String, String)>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_channel(repository, channel) {
            Ok(channel) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Channel(channel) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn get_repositories_channels_releases(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release)): Path<(String, String, String)>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, Json(Response { response_code: 100, data: ResponseData::Error(e.to_string()) }))
        };

        match db.read_release(repository, channel, release) {
            Ok(release) => (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Release(release) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }    }

    async fn get_repositories_channels_releases_artifacts(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release, artifact)): Path<(String, String, String, u32)>
    ) -> (StatusCode, Json<Response>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
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
    Channel(Channel),
    Release(Release),
    Artifact(Artifact)
}
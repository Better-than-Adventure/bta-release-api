

use std::{any::Any, fs::File, io::Read, sync::Arc};

use axum::{body::{self, Bytes}, extract::{Path, State}, http::{Response, StatusCode}, response::IntoResponse, routing::get, Json, Router};
use log::warn;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncReadExt;
use crate::{config::Config, db::{DbError, ReleaseDatabase}, release::{Artifact, Channel, Release, Repository}};

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
            .route("/:repository", get(Self::get_repository))
            .route("/:repository/", get(Self::get_repository))
            .route("/:repository/:channel", get(Self::get_repository_channel))
            .route("/:repository/:channel/", get(Self::get_repository_channel))
            .route("/:repository/:channel/:release", get(Self::get_repository_channel_release))
            .route("/:repository/:channel/:release/", get(Self::get_repository_channel_release))
            .route("/:repository/:channel/:release/:artifact", get(Self::get_repository_channel_release_artifact))
            .route("/:repository/:channel/:release/:artifact/", get(Self::get_repository_channel_release_artifact))
            .route("/:repository/:channel/:release/:artifact/download", get(Self::get_repository_channel_release_artifact_download))
            .with_state(shared_state.clone());

        let listener = tokio::net::TcpListener::bind(shared_state.config.bind_addr()).await.unwrap();
        axum::serve(listener, app).await.unwrap();
    }

    async fn get_repository(
        State(state): State<Arc<Api>>,
        Path(repository): Path<String>
    ) -> (StatusCode, Json<ResponseJson>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => {
                warn!("Failed to open database: {}", e.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) }))
            }
        };

        match db.read_repository(repository) {
            Ok(repository) => (StatusCode::OK, Json(ResponseJson { response_code: 0, data: ResponseData::Repository(repository) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(ResponseJson { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn get_repository_channel(
        State(state): State<Arc<Api>>,
        Path((repository, channel)): Path<(String, String)>
    ) -> (StatusCode, Json<ResponseJson>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => {
                warn!("Failed to open database: {}", e.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) }))
            }
        };

        match db.read_channel(repository, channel) {
            Ok(channel) => (StatusCode::OK, Json(ResponseJson { response_code: 0, data: ResponseData::Channel(channel) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(ResponseJson { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn get_repository_channel_release(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release)): Path<(String, String, String)>
    ) -> (StatusCode, Json<ResponseJson>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => {
                warn!("Failed to open database: {}", e.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) }))
            }
        };

        match db.read_release(repository, channel, release) {
            Ok(release) => (StatusCode::OK, Json(ResponseJson { response_code: 0, data: ResponseData::Release(release) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(ResponseJson { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }    
    }

    async fn get_repository_channel_release_artifact(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release, artifact)): Path<(String, String, String, u32)>
    ) -> (StatusCode, Json<ResponseJson>) {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => {
                warn!("Failed to open database: {}", e.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) }))
            }
        };

        match db.read_artifact(repository, channel, release, artifact) {
            Ok(artifact) => (StatusCode::OK, Json(ResponseJson { response_code: 0, data: ResponseData::Artifact(artifact) })),
            Err(e) => (StatusCode::BAD_REQUEST, Json(ResponseJson { response_code: 4, data: ResponseData::Error(e.to_string()) }))
        }
    }

    async fn get_repository_channel_release_artifact_download(
        State(state): State<Arc<Api>>,
        Path((repository, channel, release, artifact)): Path<(String, String, String, u32)>
    ) -> Response<body::Body> {
        let db = match ReleaseDatabase::new(state.config.db_path()) {
            Ok(db) => db,
            Err(e) => {
                warn!("Failed to open database: {}", e.to_string());
                return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) })).into_response()
            }
        };

        match db.read_artifact(&repository, &channel, &release, artifact) {
            Ok(a) => {
                let file_path = state.config.data_dir().join("./".to_string() + a.path());
                let mut file = match File::open(&file_path) {
                    Ok(file) => file,
                    Err(e) => {
                        warn!("Failed to open artifact file in API path /{}/{}/{}/{}/download; real path {}: {}", repository, channel, release, artifact, file_path.to_str().unwrap(), e.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) })).into_response()
                    }
                };
                let mut data = Vec::new();
                match file.read_to_end(&mut data) {
                    Ok(_) => {
                        return Response::new(data.into());
                    },
                    Err(e) => {
                        warn!("Failed to read artifact file in API path /{}/{}/{}/{}/download; real path {}: {}", repository, channel, release, artifact, file_path.to_str().unwrap(), e.to_string());
                        return (StatusCode::INTERNAL_SERVER_ERROR, Json(ResponseJson { response_code: 100, data: ResponseData::Error(e.to_string()) })).into_response()
                    }
                }
            },
            Err(e) => {
                return (StatusCode::BAD_REQUEST, Json(ResponseJson { response_code: 4, data: ResponseData::Error(e.to_string()) })).into_response();
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct ResponseJson {
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
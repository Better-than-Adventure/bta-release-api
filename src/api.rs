use std::{any::Any, collections::HashMap, sync::Arc};

use axum::{extract::{Query, State}, http::StatusCode, routing::get, Json, Router};
use serde::{Deserialize, Serialize};
use crate::release::{Artifact, Release, ReleaseChannel, Repository};

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

        // Check if we have artifact, we also have release and channel
        if artifact.is_some() {
            if release.is_some() {
                if channel.is_some() {
                    // All good
                } else {
                    return (StatusCode::BAD_REQUEST, Json(Response { response_code: 3, data: ResponseData::None }))
                }
            } else {
                return (StatusCode::BAD_REQUEST, Json(Response { response_code: 3, data: ResponseData::None }))
            }
        } else {
            // Check if we have release, we also have channel
            if release.is_some() {
                if channel.is_some() {
                    // All good
                } else {
                    return (StatusCode::BAD_REQUEST, Json(Response { response_code: 3, data: ResponseData::None }))
                }
            }
        }

        if let Some(channel) = channel {
            for c in state.repository.channels() {
                if c.id() == channel {
                    if let Some(release) = release {
                        for r in c.releases() {
                            if r.id() == release {
                                if let Some(artifact) = artifact {
                                    for a in r.artifacts() {
                                        if a.id() == artifact {
                                            let artifact = a.clone();
                                            return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Artifact(artifact) }))
                                        }
                                    }
                                } else {
                                    let release = r.clone();
                                    return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Release(release) }))
                                }
                            }
                        }
                    } else {
                        let release_channel = c.clone();
                        return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::ReleaseChannel(release_channel)}))
                    }
                }
            }
        } else {
            let repository = state.repository.clone();
            return (StatusCode::OK, Json(Response { response_code: 0, data: ResponseData::Repository(repository) }))
        }

        return (StatusCode::BAD_REQUEST, Json(Response { response_code: 4, data: ResponseData::None }))
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
    Repository(Repository),
    ReleaseChannel(ReleaseChannel),
    Release(Release),
    Artifact(Artifact)
}
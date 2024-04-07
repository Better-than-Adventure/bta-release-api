use serde::{Deserialize, Serialize};

type Id = u32;

#[derive(Serialize, Deserialize, Clone)]
pub struct Repository {
    channels: Vec<ReleaseChannel>
}

impl Repository {
    pub fn new(channels: Vec<ReleaseChannel>) -> Self {
        Self {
            channels
        }
    }

    pub fn channels(&self) -> &Vec<ReleaseChannel> {
        &self.channels
    }

    pub fn dummy() -> Repository {
        let vec = vec![
            ReleaseChannel {
                id: 0,
                name: "stable".into(),
                releases: vec![
                    Release {
                        id: 0,
                        name: "7.1".into(),
                        created_at: 0u64,
                        artifacts: vec![
                            Artifact {
                                id: 0,
                                name: "Client JAR".into(),
                                path: "/downloads/stable/7.1/client.jar".into(),
                                artifact_type: ArtifactType::ClientJar
                            },
                            Artifact {
                                id: 1,
                                name: "Server JAR".into(),
                                path: "/downloads/stable/7.1/server.jar".into(),
                                artifact_type: ArtifactType::ServerJar
                            },
                            Artifact {
                                id: 2,
                                name: "MultiMC Instance".into(),
                                path: "/downloads/stable/7.1/mmc.zip".into(),
                                artifact_type: ArtifactType::MmcInstance
                            },
                            Artifact {
                                id: 3,
                                name: "Manifest".into(),
                                path: "/downloads/stable/7.1/manifest.json".into(),
                                artifact_type: ArtifactType::Manifest
                            }
                        ]
                    }
                ]
            },
            ReleaseChannel {
                id: 1,
                name: "snapshot".into(),
                releases: vec![
                    Release {
                        id: 0,
                        name: "7.1 Prerelease 2a".into(),
                        created_at: 0u64,
                        artifacts: vec![
                            Artifact {
                                id: 0,
                                name: "Client JAR".into(),
                                path: "/downloads/snapshot/7.1pre2a/client.jar".into(),
                                artifact_type: ArtifactType::ClientJar
                            },
                            Artifact {
                                id: 1,
                                name: "Server JAR".into(),
                                path: "/downloads/snapshot/7.1pre2a/server.jar".into(),
                                artifact_type: ArtifactType::ServerJar
                            },
                            Artifact {
                                id: 2,
                                name: "MultiMC Instance".into(),
                                path: "/downloads/snapshot/7.1pre2a/mmc.zip".into(),
                                artifact_type: ArtifactType::MmcInstance
                            },
                            Artifact {
                                id: 3,
                                name: "Manifest".into(),
                                path: "/downloads/snapshot/7.1pre2a/manifest.json".into(),
                                artifact_type: ArtifactType::Manifest
                            }
                        ]
                    }
                ]
            },
            ReleaseChannel {
                id: 2,
                name: "nightly".into(),
                releases: vec![
                    Release {
                        id: 0,
                        name: "Nightly 2024-04-06".into(),
                        created_at: 0u64,
                        artifacts: vec![
                            Artifact {
                                id: 0,
                                name: "Client JAR".into(),
                                path: "/downloads/nightly/20240406/client.jar".into(),
                                artifact_type: ArtifactType::ClientJar
                            },
                            Artifact {
                                id: 1,
                                name: "Server JAR".into(),
                                path: "/downloads/nightly/20240406/server.jar".into(),
                                artifact_type: ArtifactType::ServerJar
                            },
                            Artifact {
                                id: 2,
                                name: "Manifest".into(),
                                path: "/downloads/nightly/20240406/manifest.json".into(),
                                artifact_type: ArtifactType::Manifest
                            }
                        ]
                    }
                ]
            }
        ];
        Self { channels: vec }
    }

}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReleaseChannel {
    id: Id,
    name: String,
    releases: Vec<Release>
}

impl ReleaseChannel {
    pub fn new<S: Into<String>>(id: Id, name: S, releases: Vec<Release>) -> Self {
        Self {
            id,
            name: name.into(),
            releases
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn releases(&self) -> &Vec<Release> {
        &self.releases
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Release {
    id: Id,
    name: String,
    created_at: u64,
    artifacts: Vec<Artifact>
}

impl Release {
    pub fn new<S: Into<String>>(id: Id, name: S, created_at: u64, artifacts: Vec<Artifact>) -> Self {
        Self {
            id,
            name: name.into(),
            created_at,
            artifacts
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_at(&self) -> &u64 {
        &self.created_at
    }

    pub fn artifacts(&self) -> &Vec<Artifact> {
        &self.artifacts
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Artifact {
    id: Id,
    name: String,
    path: String,
    artifact_type: ArtifactType
}

impl Artifact {
    pub fn new<S: Into<String>>(id: Id, name: S, path: S, artifact_type: ArtifactType) -> Self {
        Self {
            id,
            name: name.into(),
            path: path.into(),
            artifact_type
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn artifact_type(&self) -> ArtifactType {
        self.artifact_type
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ArtifactType {
    ClientJar,
    ServerJar,
    Manifest,
    MmcInstance
}

impl Into<u32> for ArtifactType {
    fn into(self) -> u32 {
        match self {
            Self::ClientJar => 0,
            Self::ServerJar => 1,
            Self::Manifest => 2,
            Self::MmcInstance => 3
        }
    }
}

impl TryFrom<u32> for ArtifactType {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::ClientJar),
            1 => Ok(Self::ServerJar),
            2 => Ok(Self::Manifest),
            3 => Ok(Self::MmcInstance),
            _ => Err(())
        }
    }
}
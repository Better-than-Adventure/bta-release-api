use serde::{Deserialize, Serialize};

type Id = u32;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub enum Type {
    #[serde(rename = "repository")]
    Repository,
    #[serde(rename = "channel")]
    Channel,
    #[serde(rename = "release")]
    Release,
    #[serde(rename = "artifact")]
    Artifact
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Repository {
    #[serde(rename = "type")]
    _type: Type,
    id: String,
    channels: Vec<ReleaseChannel>
}

impl Repository {
    pub fn new(id: String, channels: Vec<ReleaseChannel>) -> Self {
        Self {
            _type: Type::Repository,
            id,
            channels
        }
    }

    pub fn channels(&self) -> &Vec<ReleaseChannel> {
        &self.channels
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ReleaseChannel {
    #[serde(rename = "type")]
    _type: Type,
    id: Id,
    name: String,
    releases: Vec<Release>
}

impl ReleaseChannel {
    pub fn new<S: Into<String>>(id: Id, name: S, releases: Vec<Release>) -> Self {
        Self {
            _type: Type::Channel,
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
    #[serde(rename = "type")]
    _type: Type,
    id: Id,
    name: String,
    created_at: u64,
    artifacts: Vec<Artifact>
}

impl Release {
    pub fn new<S: Into<String>>(id: Id, name: S, created_at: u64, artifacts: Vec<Artifact>) -> Self {
        Self {
            _type: Type::Release,
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
    #[serde(rename = "type")]
    _type: Type,
    id: Id,
    name: String,
    path: String,
    artifact_type: ArtifactType
}

impl Artifact {
    pub fn new<S: Into<String>>(id: Id, name: S, path: S, artifact_type: ArtifactType) -> Self {
        Self {
            _type: Type::Artifact,
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
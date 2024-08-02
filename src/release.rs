use serde::{Deserialize, Serialize};

type Id = u32;

#[derive(Serialize, Deserialize, Clone)]
pub struct Repository {
    id: String,
    channels: Vec<String>
}

impl Repository {
    pub fn new(id: String, channels: Vec<Channel>) -> Self {
        Self {
            id,
            channels: channels.iter().map(|c| c.id().to_string()).collect()
        }
    }

    pub fn channels(&self) -> &Vec<String> {
        &self.channels
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Channel {
    id: String,
    releases: Vec<String>
}

impl Channel {
    pub fn new<S: Into<String>>(id: S, releases: Vec<Release>) -> Self {
        Self {
            id: id.into(),
            releases: releases.iter().map(|r| r.id().to_string()).collect()
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn releases(&self) -> &Vec<String> {
        &self.releases
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Release {
    id: String,
    name: String,
    created_at: u64,
    artifacts: Vec<String>
}

impl Release {
    pub fn new<S: Into<String>>(id: S, name: S, created_at: u64, artifacts: Vec<Artifact>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            created_at,
            artifacts: artifacts.iter().map(|a| a.id().to_string()).collect()
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn created_at(&self) -> &u64 {
        &self.created_at
    }

    pub fn artifacts(&self) -> &Vec<String> {
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
    MmcInstance,
    Other
}

impl Into<u32> for ArtifactType {
    fn into(self) -> u32 {
        match self {
            Self::ClientJar => 0,
            Self::ServerJar => 1,
            Self::Manifest => 2,
            Self::MmcInstance => 3,
            Self::Other => 4
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
            4 => Ok(Self::Other),
            _ => Err(())
        }
    }
}
use std::{error::Error, fmt::Display, path::Path};

use log::warn;
use rusqlite::Connection;

use crate::release::{Artifact, ArtifactType, Release, Channel, Repository};

type Result<T> = core::result::Result<T, Box<dyn Error>>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DbError {
    NoSuchKey,
    ParseErr
}

impl Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::NoSuchKey => write!(f, "No such key"),
            DbError::ParseErr => write!(f, "Parse error")
        }
    }
}

impl Error for DbError { }

pub struct ReleaseDatabase {
    connection: Connection
}

impl ReleaseDatabase {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let con = Connection::open(path)?;
        Self::init_db(&con)?;
        Ok(Self {
            connection: con
        })
    }

    fn init_db(connection: &Connection) -> Result<()> {
        connection.execute(
            "CREATE TABLE IF NOT EXISTS repository (
                id      TEXT PRIMARY KEY
            )",
            ()
        )?;
        connection.execute(
            "INSERT OR IGNORE INTO repository(id)
            VALUES
                (\"mod\"),
                (\"updater\")",
            ()
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS channel (
                id          TEXT NOT NULL,
                repository  TEXT NOT NULL,
                CONSTRAINT key PRIMARY KEY (id, repository),
                FOREIGN KEY(repository) REFERENCES repository(id)
            )",
            ()
        )?;
        connection.execute(
            "INSERT OR IGNORE INTO channel(id, repository)
            VALUES
                (\"stable\", \"mod\"),
                (\"snapshot\", \"mod\"),
                (\"nightly\", \"mod\"),
                (\"release\", \"updater\")",
                ()
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS release (
                id          INTEGER NOT NULL,
                repository  TEXT NOT NULL,
                channel     TEXT NOT NULL,
                name        TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                CONSTRAINT key PRIMARY KEY (id, channel),
                FOREIGN KEY(channel) REFERENCES channel(id)
            )",
            ()
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS artifact (
                id          INTEGER NOT NULL,
                repository  TEXT NOT NULL,
                channel     TEXT NOT NULL,
                release     INTEGER NOT NULL,
                name        TEXT NOT NULL,
                path        TEXT NOT NULL,
                type        INTEGER NOT NULL,
                CONSTRAINT key PRIMARY KEY (id, release),
                FOREIGN KEY(release) REFERENCES release(id)
            )",
            ()
        )?;

        Ok(())
    }

    pub fn read_artifact(&self, repository_id: String, channel_id: String, release_id: u32, artifact_id: u32) -> Result<Artifact> {
        let mut statement = self.connection.prepare(
            format!(
                "SELECT art.id, art.name, art.path, art.type
                FROM artifact AS art
                INNER JOIN release AS rel ON rel.id=art.release
                INNER JOIN channel AS cha ON cha.id=art.channel
                INNER JOIN repository AS rep ON rep.id=art.repository
                WHERE art.id={artifact_id}").as_str()
        )?;

        let db_artifact = statement.query_row([], |row| {
            Ok(DbArtifact {
                id: row.get(0)?,
                name: row.get(1)?,
                path: row.get(2)?,
                artifact_type: row.get(3)?
            })
        })?;

        match db_artifact.try_into_artifact() {
            Ok(artifact) => Ok(artifact),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }

    pub fn read_release(&self, repository_id: String, channel_id: String, release_id: u32) -> Result<Release> {
        let mut statement = self.connection.prepare(
            format!(
                "SELECT rel.id, rel.name, rel.created_at
                FROM release AS rel
                INNER JOIN channel AS cha ON cha.id=rel.channel
                INNER JOIN repository AS rep ON rep.id=rel.repository
                WHERE rel.id={release_id}").as_str()
        )?;

        let db_release = statement.query_row([], |row| {
            Ok(DbRelease {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?
            })
        })?;

        let mut artifact_statement = self.connection.prepare(
            format!("SELECT id FROM artifact WHERE release={release_id}").as_str()
        )?;

        let artifact_ids: std::result::Result<Vec<_>, _> = artifact_statement
            .query_map([], |row| row.get::<usize, u32>(0))?
            .collect();
        let artifact_ids = match artifact_ids {
            Ok(artifact_ids) => artifact_ids,
            Err(_) => {
                warn!(target: "read_db", "release with id {release_id} has artifact with invalid id");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        let artifacts: std::result::Result<Vec<_>, _> = artifact_ids
            .into_iter()
            .map(|id| self.read_artifact(repository_id.to_string(), channel_id.to_string(), release_id, id))
            .collect();
        let artifacts = match artifacts {
            Ok(artifacts) => artifacts,
            Err(_) => {
                warn!(target: "read_db", "release with id {release_id} has broken artifact");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        match db_release.try_into_release(artifacts) {
            Ok(release) => Ok(release),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }

    pub fn read_channel(&self, repository_id: String, channel_id: String) -> Result<Channel> {
        let mut statement = self.connection.prepare(
            format!(
                "SELECT cha.id 
                FROM channel AS cha
                INNER JOIN repository AS rep ON rep.id=cha.repository
                WHERE cha.id=\"{channel_id}\"").as_str()
        )?;

        let db_channel = statement.query_row([], |row| {
            Ok(DbChannel {
                id: row.get(0)?
            })
        })?;

        let mut release_statement = self.connection.prepare(
            format!("SELECT id FROM release WHERE channel=\"{channel_id}\"").as_str()
        )?;

        let release_ids: std::result::Result<Vec<_>, _> = release_statement
            .query_map([], |row| row.get::<usize, u32>(0))?
            .collect();
        let release_ids = match release_ids {
            Ok(release_ids) => release_ids,
            Err(_) => {
                warn!(target: "read_db", "channel with id \"{channel_id}\" has release with invalid id");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        let releases: std::result::Result<Vec<_>, _> = release_ids
            .into_iter()
            .map(|id| self.read_release(repository_id.to_string(), channel_id.to_string(), id))
            .collect();
        let releases = match releases {
            Ok(releases) => releases,
            Err(_) => {
                warn!(target: "read_db", "channel with id \"{channel_id}\" has broken release");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        match db_channel.try_into_channel(releases) {
            Ok(channel) => Ok(channel),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }

    pub fn read_repository(&self, repository_id: String) -> Result<Repository> {
        let mut statement = self.connection.prepare(
            format!("SELECT id FROM repository WHERE id=\"{repository_id}\"").as_str()
        )?;

        let db_repository = statement.query_row([], |row| {
            Ok(DbRepository {
                id: row.get(0)?
            })
        })?;

        let mut channel_statement = self.connection.prepare(
            format!("select id FROM channel WHERE repository=\"{repository_id}\"").as_str()
        )?;

        let channel_ids: std::result::Result<Vec<_>, _> = channel_statement
            .query_map([], |row| row.get::<usize, String>(0))?
            .collect();
        let channel_ids = match channel_ids {
            Ok(channel_ids) => channel_ids,
            Err(_) => {
                warn!(target: "read_db", "repository with id \"{repository_id}\" has channel with invalid id");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        let channels: std::result::Result<Vec<_>, _> = channel_ids
            .into_iter()
            .map(|id| self.read_channel(repository_id.to_string(), id))
            .collect();
        let channels = match channels {
            Ok(channels) => channels,
            Err(_) => {
                warn!(target: "read_db", "repository with id \"{repository_id}\" has broken channel");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        match db_repository.try_into_repository(channels) {
            Ok(repository) => Ok(repository),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }
}

struct DbRepository {
    id: String
}

impl DbRepository {
    fn try_into_repository(self, channels: Vec<Channel>) -> std::result::Result<Repository, ()> {
        Ok(Repository::new(self.id, channels))
    }
}

struct DbChannel {
    id: String
}

impl DbChannel {
    fn try_into_channel(self, releases: Vec<Release>) -> std::result::Result<Channel, ()> {
        Ok(Channel::new(self.id, releases))
    }
}

struct DbRelease {
    id: u32,
    name: String,
    created_at: u64
}

impl DbRelease {
    fn try_into_release(self, artifacts: Vec<Artifact>) -> std::result::Result<Release, ()> {
        Ok(Release::new(self.id, self.name, self.created_at, artifacts))
    }
}

struct DbArtifact {
    id: u32,
    name: String,
    path: String,
    artifact_type: u32
}

impl  DbArtifact {
    fn try_into_artifact(self) -> std::result::Result<Artifact, ()> {
        if let Ok(artifact_type) = ArtifactType::try_from(self.artifact_type) {
            Ok(Artifact::new(self.id, self.name, self.path, artifact_type))
        } else {
            Err(())
        }
    }
}
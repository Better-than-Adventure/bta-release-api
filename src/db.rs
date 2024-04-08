use std::{error::Error, fmt::Display, path::Path};

use log::warn;
use rusqlite::Connection;

use crate::release::{Artifact, ArtifactType, Release, ReleaseChannel, Repository};

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
                id          INTEGER PRIMARY KEY,
                repository  TEXT NOT NULL,
                name        TEXT NOT NULL,
                FOREIGN KEY(repository) REFERENCES repository(id)
            )",
            ()
        )?;
        connection.execute(
            "INSERT OR IGNORE INTO channel(id, repository, name)
            VALUES
                (0, \"mod\", \"stable\"),
                (1, \"mod\", \"snapshot\"),
                (2, \"mod\", \"nightly\"),
                (3, \"updater\", \"release\")",
                ()
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS release (
                id          INTEGER PRIMARY KEY,
                channel     INTEGER NOT NULL,
                name        TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                FOREIGN KEY(channel) REFERENCES channel(id)
            )",
            ()
        )?;
        connection.execute(
            "CREATE TABLE IF NOT EXISTS artifact (
                id      INTEGER PRIMARY KEY,
                release INTEGER NOT NULL,
                name    TEXT NOT NULL,
                path    TEXT NOT NULL,
                type    INTEGER NOT NULL,
                FOREIGN KEY(release) REFERENCES release(id)
            )",
            ()
        )?;

        Ok(())
    }

    pub fn read_artifact(&self, id: u32) -> Result<Artifact> {
        let mut statement = self.connection.prepare(
            format!("SELECT id, release, name, path, type FROM artifact WHERE id={id}").as_str()
        )?;

        let db_artifact = statement.query_row([], |row| {
            Ok(DbArtifact {
                id: row.get(0)?,
                release: row.get(1)?,
                name: row.get(2)?,
                path: row.get(3)?,
                artifact_type: row.get(4)?
            })
        })?;

        match db_artifact.try_into_artifact() {
            Ok(artifact) => Ok(artifact),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }

    pub fn read_release(&self, id: u32) -> Result<Release> {
        let mut statement = self.connection.prepare(
            format!("SELECT id, channel, name, created_at FROM release WHERE id={id}").as_str()
        )?;

        let db_release = statement.query_row([], |row| {
            Ok(DbRelease {
                id: row.get(0)?,
                channel: row.get(1)?,
                name: row.get(2)?,
                created_at: row.get(3)?
            })
        })?;

        let mut artifact_statement = self.connection.prepare(
            format!("SELECT id FROM artifact WHERE release={id}").as_str()
        )?;

        let artifact_ids: std::result::Result<Vec<_>, _> = artifact_statement
            .query_map([], |row| row.get::<usize, u32>(0))?
            .collect();
        let artifact_ids = match artifact_ids {
            Ok(artifact_ids) => artifact_ids,
            Err(_) => {
                warn!(target: "read_db", "release with id {id} has artifact with invalid id");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        let artifacts: std::result::Result<Vec<_>, _> = artifact_ids
            .into_iter()
            .map(|id| self.read_artifact(id))
            .collect();
        let artifacts = match artifacts {
            Ok(artifacts) => artifacts,
            Err(_) => {
                warn!(target: "read_db", "release with id {id} has broken artifact");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        match db_release.try_into_release(artifacts) {
            Ok(release) => Ok(release),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }

    pub fn read_channel(&self, id: u32) -> Result<ReleaseChannel> {
        let mut statement = self.connection.prepare(
            format!("SELECT id, repository, name FROM channel WHERE id={id}").as_str()
        )?;

        let db_channel = statement.query_row([], |row| {
            Ok(DbChannel {
                id: row.get(0)?,
                repository: row.get(1)?,
                name: row.get(2)?
            })
        })?;

        let mut release_statement = self.connection.prepare(
            format!("SELECT id FROM release WHERE channel={id}").as_str()
        )?;

        let release_ids: std::result::Result<Vec<_>, _> = release_statement
            .query_map([], |row| row.get::<usize, u32>(0))?
            .collect();
        let release_ids = match release_ids {
            Ok(release_ids) => release_ids,
            Err(_) => {
                warn!(target: "read_db", "channel with id {id} has release with invalid id");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        let releases: std::result::Result<Vec<_>, _> = release_ids
            .into_iter()
            .map(|id| self.read_release(id))
            .collect();
        let releases = match releases {
            Ok(releases) => releases,
            Err(_) => {
                warn!(target: "read_db", "channel with id {id} has broken release");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        match db_channel.try_into_channel(releases) {
            Ok(channel) => Ok(channel),
            Err(_) => Err(Box::new(DbError::ParseErr))
        }
    }

    pub fn read_repository(&self, id: String) -> Result<Repository> {
        let mut statement = self.connection.prepare(
            format!("SELECT id FROM repository WHERE id=\"{id}\"").as_str()
        )?;

        let db_repository = statement.query_row([], |row| {
            Ok(DbRepository {
                id: row.get(0)?
            })
        })?;

        let mut channel_statement = self.connection.prepare(
            format!("select id FROM channel WHERE repository=\"{id}\"").as_str()
        )?;

        let channel_ids: std::result::Result<Vec<_>, _> = channel_statement
            .query_map([], |row| row.get::<usize, u32>(0))?
            .collect();
        let channel_ids = match channel_ids {
            Ok(channel_ids) => channel_ids,
            Err(_) => {
                warn!(target: "read_db", "repository with id \"{id}\" has channel with invalid id");
                return Err(Box::new(DbError::ParseErr));
            }
        };

        let channels: std::result::Result<Vec<_>, _> = channel_ids
            .into_iter()
            .map(|id| self.read_channel(id))
            .collect();
        let channels = match channels {
            Ok(channels) => channels,
            Err(_) => {
                warn!(target: "read_db", "repository with id \"{id}\" has broken channel");
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
    fn try_into_repository(self, channels: Vec<ReleaseChannel>) -> std::result::Result<Repository, ()> {
        Ok(Repository::new(self.id, channels))
    }
}

struct DbChannel {
    id: u32,
    repository: String,
    name: String
}

impl DbChannel {
    fn try_into_channel(self, releases: Vec<Release>) -> std::result::Result<ReleaseChannel, ()> {
        Ok(ReleaseChannel::new(self.id, self.name, releases))
    }
}

struct DbRelease {
    id: u32,
    channel: u32,
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
    release: u32,
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
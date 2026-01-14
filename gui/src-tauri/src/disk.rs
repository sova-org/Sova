use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sova_server::Snapshot;
use std::path::PathBuf;
use std::{error::Error, fmt, io, path::Path};
use tokio::{
    fs::{self, ReadDir},
    io::ErrorKind,
};

#[derive(Debug)]
pub enum DiskError {
    DirectoryResolutionFailed,
    DirectoryCreationFailed {
        path: PathBuf,
        source: io::Error,
    },
    DirectoryReadFailed {
        path: PathBuf,
        source: io::Error,
    },
    FileWriteFailed {
        path: PathBuf,
        source: io::Error,
    },
    FileReadFailed {
        path: PathBuf,
        source: io::Error,
    },
    FileDeleteFailed {
        path: PathBuf,
        source: io::Error,
    },
    FileRenameFailed {
        from: PathBuf,
        to: PathBuf,
        source: io::Error,
    },
    SerializationFailed {
        source: serde_json::Error,
    },
    DeserializationFailed {
        path: PathBuf,
        source: serde_json::Error,
    },
    ProjectNotFound {
        name: String,
    },
}

impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiskError::DirectoryResolutionFailed => {
                write!(f, "Could not determine config directory")
            }
            DiskError::DirectoryCreationFailed { path, .. } => {
                write!(f, "Failed to create directory '{}'", path.display())
            }
            DiskError::DirectoryReadFailed { path, .. } => {
                write!(f, "Failed to read directory '{}'", path.display())
            }
            DiskError::FileWriteFailed { path, .. } => {
                write!(f, "Failed to write '{}'", path.display())
            }
            DiskError::FileReadFailed { path, .. } => {
                write!(f, "Failed to read '{}'", path.display())
            }
            DiskError::FileDeleteFailed { path, .. } => {
                write!(f, "Failed to delete '{}'", path.display())
            }
            DiskError::FileRenameFailed { from, to, .. } => {
                write!(
                    f,
                    "Failed to rename '{}' to '{}'",
                    from.display(),
                    to.display()
                )
            }
            DiskError::SerializationFailed { .. } => write!(f, "Failed to serialize data"),
            DiskError::DeserializationFailed { path, .. } => {
                write!(f, "Failed to parse '{}'", path.display())
            }
            DiskError::ProjectNotFound { name } => {
                write!(f, "Project '{}' not found", name)
            }
        }
    }
}

impl Error for DiskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DiskError::DirectoryCreationFailed { source, .. }
            | DiskError::DirectoryReadFailed { source, .. }
            | DiskError::FileWriteFailed { source, .. }
            | DiskError::FileReadFailed { source, .. }
            | DiskError::FileDeleteFailed { source, .. }
            | DiskError::FileRenameFailed { source, .. } => Some(source),
            DiskError::SerializationFailed { source }
            | DiskError::DeserializationFailed { source, .. } => Some(source),
            DiskError::DirectoryResolutionFailed | DiskError::ProjectNotFound { .. } => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectFile {
    pub snapshot: Snapshot,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub tempo: Option<f32>,
    pub line_count: Option<usize>,
}

type Result<T> = std::result::Result<T, DiskError>;

async fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn get_projects_dir() -> Result<PathBuf> {
    let config_dir = dirs::config_dir().ok_or(DiskError::DirectoryResolutionFailed)?;
    let projects_dir = config_dir.join("sova").join("projects");
    ensure_dir(&projects_dir).await?;
    Ok(projects_dir)
}

fn project_path(projects_dir: &Path, name: &str) -> PathBuf {
    projects_dir.join(format!("{}.sova", name))
}

pub async fn save_project(snapshot: &Snapshot, name: &str) -> Result<()> {
    let projects_dir = get_projects_dir().await?;
    let path = project_path(&projects_dir, name);

    let now = Utc::now();

    // Preserve created_at if file exists
    let created_at = match fs::read_to_string(&path).await {
        Ok(content) => serde_json::from_str::<ProjectFile>(&content)
            .map(|f| f.created_at)
            .unwrap_or(now),
        Err(_) => now,
    };

    let file = ProjectFile {
        snapshot: snapshot.clone(),
        created_at,
        updated_at: now,
    };

    let json =
        serde_json::to_string_pretty(&file).map_err(|e| DiskError::SerializationFailed { source: e })?;

    fs::write(&path, json)
        .await
        .map_err(|e| DiskError::FileWriteFailed { path, source: e })
}

pub async fn load_project(name: &str) -> Result<Snapshot> {
    let projects_dir = get_projects_dir().await?;
    let path = project_path(&projects_dir, name);

    let content = fs::read_to_string(&path).await.map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            DiskError::ProjectNotFound {
                name: name.to_string(),
            }
        } else {
            DiskError::FileReadFailed {
                path: path.clone(),
                source: e,
            }
        }
    })?;

    let file: ProjectFile =
        serde_json::from_str(&content).map_err(|e| DiskError::DeserializationFailed {
            path: path.clone(),
            source: e,
        })?;

    Ok(file.snapshot)
}

pub async fn list_projects() -> Result<Vec<ProjectInfo>> {
    let projects_dir = get_projects_dir().await?;
    let mut read_dir: ReadDir =
        fs::read_dir(&projects_dir)
            .await
            .map_err(|e| DiskError::DirectoryReadFailed {
                path: projects_dir.clone(),
                source: e,
            })?;

    let mut projects = Vec::new();

    while let Ok(Some(entry)) = read_dir.next_entry().await {
        let path = entry.path();

        if path.extension().map(|e| e == "sova").unwrap_or(false) {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            if name.is_empty() {
                continue;
            }

            // Read file to extract metadata
            let info = match fs::read_to_string(&path).await {
                Ok(content) => match serde_json::from_str::<ProjectFile>(&content) {
                    Ok(file) => ProjectInfo {
                        name,
                        created_at: Some(file.created_at),
                        updated_at: Some(file.updated_at),
                        tempo: Some(file.snapshot.tempo as f32),
                        line_count: Some(file.snapshot.scene.lines.len()),
                    },
                    Err(_) => ProjectInfo {
                        name,
                        created_at: None,
                        updated_at: None,
                        tempo: None,
                        line_count: None,
                    },
                },
                Err(_) => continue,
            };

            projects.push(info);
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(projects)
}

pub async fn delete_project(name: &str) -> Result<()> {
    let projects_dir = get_projects_dir().await?;
    let path = project_path(&projects_dir, name);

    match fs::remove_file(&path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(()),
        Err(e) => Err(DiskError::FileDeleteFailed { path, source: e }),
    }
}

pub async fn rename_project(old_name: &str, new_name: &str) -> Result<()> {
    let projects_dir = get_projects_dir().await?;
    let old_path = project_path(&projects_dir, old_name);
    let new_path = project_path(&projects_dir, new_name);

    if !old_path.exists() {
        return Err(DiskError::ProjectNotFound {
            name: old_name.to_string(),
        });
    }

    fs::rename(&old_path, &new_path)
        .await
        .map_err(|e| DiskError::FileRenameFailed {
            from: old_path,
            to: new_path,
            source: e,
        })
}

pub async fn get_projects_directory() -> Result<String> {
    let projects_dir = get_projects_dir().await?;
    Ok(projects_dir.to_string_lossy().to_string())
}

pub async fn load_project_from_path(path: &Path) -> Result<Snapshot> {
    let content = fs::read_to_string(path).await.map_err(|e| {
        if e.kind() == ErrorKind::NotFound {
            DiskError::ProjectNotFound {
                name: path.to_string_lossy().to_string(),
            }
        } else {
            DiskError::FileReadFailed {
                path: path.to_path_buf(),
                source: e,
            }
        }
    })?;

    let file: ProjectFile =
        serde_json::from_str(&content).map_err(|e| DiskError::DeserializationFailed {
            path: path.to_path_buf(),
            source: e,
        })?;

    Ok(file.snapshot)
}

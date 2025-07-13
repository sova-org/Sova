use chrono::{DateTime, Utc};
use crate::messages::Snapshot;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{error::Error, fmt, io, path::Path, str::FromStr};
use tokio::{
    fs::{self, DirEntry, ReadDir},
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
    DirectoryEntryReadFailed {
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
    SerializationFailed {
        source: serde_json::Error,
    },
    DeserializationFailed {
        path: PathBuf,
        source: serde_json::Error,
    },
    ProjectNotFound {
        project_name: String,
        path: PathBuf,
    },
    ProjectDeletionFailed {
        path: PathBuf,
        source: io::Error,
    },
    PathMetadataCheckFailed {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiskError::DirectoryResolutionFailed => {
                write!(f, "Could not determine project directories")
            }
            DiskError::DirectoryCreationFailed { path, .. } => {
                write!(f, "Failed to create directory '{}'", path.display())
            }
            DiskError::DirectoryReadFailed { path, .. } => {
                write!(f, "Failed to read directory '{}'", path.display())
            }
            DiskError::DirectoryEntryReadFailed { path, .. } => {
                write!(f, "Failed to read directory entry in '{}'", path.display())
            }
            DiskError::FileWriteFailed { path, .. } => {
                write!(f, "Failed to write file '{}'", path.display())
            }
            DiskError::FileReadFailed { path, .. } => {
                write!(f, "Failed to read file '{}'", path.display())
            }
            DiskError::SerializationFailed { .. } => write!(f, "Failed to serialize data"),
            DiskError::DeserializationFailed { path, .. } => {
                write!(f, "Failed to deserialize data from '{}'", path.display())
            }
            DiskError::ProjectNotFound { project_name, path } => write!(
                f,
                "Project '{}' not found at '{}'",
                project_name,
                path.display()
            ),
            DiskError::ProjectDeletionFailed { path, .. } => {
                write!(f, "Failed to delete project directory '{}'", path.display())
            }
            DiskError::PathMetadataCheckFailed { path, .. } => {
                write!(f, "Failed to check metadata for path '{}'", path.display())
            }
        }
    }
}

impl Error for DiskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DiskError::DirectoryCreationFailed { source, .. }
            | DiskError::DirectoryReadFailed { source, .. }
            | DiskError::DirectoryEntryReadFailed { source, .. }
            | DiskError::FileWriteFailed { source, .. }
            | DiskError::ProjectDeletionFailed { source, .. }
            | DiskError::PathMetadataCheckFailed { source, .. }
            | DiskError::FileReadFailed { source, .. } => Some(source),
            DiskError::SerializationFailed { source, .. }
            | DiskError::DeserializationFailed { source, .. } => Some(source),
            DiskError::DirectoryResolutionFailed | DiskError::ProjectNotFound { .. } => None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tempo: Option<f32>,
    pub line_count: Option<usize>,
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

async fn create_dir_all_map_err(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn write_file_map_err<C: AsRef<[u8]>>(path: &Path, contents: C) -> Result<()> {
    fs::write(path, contents)
        .await
        .map_err(|e| DiskError::FileWriteFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn read_to_string_map_err(path: &Path) -> Result<String> {
    fs::read_to_string(path)
        .await
        .map_err(|e| DiskError::FileReadFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn read_dir_map_err(path: &Path) -> Result<ReadDir> {
    fs::read_dir(path)
        .await
        .map_err(|e| DiskError::DirectoryReadFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn next_entry_map_err(read_dir: &mut ReadDir, dir_path: &Path) -> Result<Option<DirEntry>> {
    read_dir
        .next_entry()
        .await
        .map_err(|e| DiskError::DirectoryEntryReadFailed {
            path: dir_path.to_path_buf(),
            source: e,
        })
}

async fn remove_dir_all_map_err(path: &Path) -> Result<()> {
    fs::remove_dir_all(path)
        .await
        .map_err(|e| DiskError::ProjectDeletionFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn check_path_metadata_map_err(path: &Path) -> Result<std::fs::Metadata> {
    fs::metadata(path)
        .await
        .map_err(|e| DiskError::PathMetadataCheckFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

async fn read_project_metadata(project_name: &str) -> Result<Option<ProjectMetadata>> {
    let metadata_path = get_metadata_path(project_name).await?;
    match read_to_string_map_err(&metadata_path).await {
        Ok(content) => match serde_json::from_str::<ProjectMetadata>(&content) {
            Ok(meta) => Ok(Some(meta)),
            Err(_) => Ok(None),
        },
        Err(DiskError::FileReadFailed { source, .. }) if source.kind() == ErrorKind::NotFound => {
            Ok(None)
        }
        Err(e) => Err(e),
    }
}

async fn get_base_config_dir() -> Result<PathBuf> {
    let path = UserDirs::new()
        .map(|ud| ud.home_dir().join(".config").join("bubocore"))
        .ok_or(DiskError::DirectoryResolutionFailed)?;

    create_dir_all_map_err(&path).await?;
    Ok(path)
}

async fn get_projects_dir() -> Result<PathBuf> {
    let base_dir = get_base_config_dir().await?;
    let projects_dir = base_dir.join("projects");
    create_dir_all_map_err(&projects_dir).await?;
    Ok(projects_dir)
}

async fn get_project_path(project_name: &str) -> Result<PathBuf> {
    let projects_dir = get_projects_dir().await?;
    Ok(projects_dir.join(project_name))
}

async fn get_project_scripts_dir(project_name: &str) -> Result<PathBuf> {
    let project_path = get_project_path(project_name).await?;
    Ok(project_path.join("scripts"))
}

async fn get_snapshot_file_path(project_name: &str) -> Result<PathBuf> {
    let project_path = get_project_path(project_name).await?;
    Ok(project_path.join(format!("{}.bubo", project_name)))
}

async fn get_metadata_path(project_name: &str) -> Result<PathBuf> {
    let project_path = get_project_path(project_name).await?;
    Ok(project_path.join("metadata.json"))
}

pub async fn save_project(snapshot: &Snapshot, project_name: &str) -> Result<()> {
    // 1. Ensure project directory exists
    let project_path = get_project_path(project_name).await?;
    create_dir_all_map_err(&project_path).await?;

    // 2. Save the main snapshot blob (.bubo file)
    let snapshot_file_path = get_snapshot_file_path(project_name).await?;
    let snapshot_json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| DiskError::SerializationFailed { source: e })?;
    write_file_map_err(&snapshot_file_path, snapshot_json).await?;

    // 3. Save individual scripts
    let scripts_dir = get_project_scripts_dir(project_name).await?;
    create_dir_all_map_err(&scripts_dir).await?;

    for (line_idx, line) in snapshot.scene.lines.iter().enumerate() {
        for script in &line.scripts {
            if !script.content.is_empty() {
                let script_filename = format!(
                    "line{}_frame{}.{}",
                    line_idx,
                    script.index,
                    if script.lang.is_empty() {
                        "txt"
                    } else {
                        &script.lang
                    }
                );
                let script_path = scripts_dir.join(script_filename);
                write_file_map_err(&script_path, &script.content).await?;
            }
        }
    }

    // 4. Save/Update Metadata
    let metadata_path = get_metadata_path(project_name).await?;
    let now = Utc::now();
    let tempo = Some(snapshot.tempo as f32);
    let line_count = Some(snapshot.scene.lines.len());

    let metadata = match read_project_metadata(project_name).await? {
        Some(mut existing_meta) => {
            existing_meta.updated_at = now;
            existing_meta.tempo = tempo;
            existing_meta.line_count = line_count;
            existing_meta
        }
        None => ProjectMetadata {
            created_at: now,
            updated_at: now,
            tempo,
            line_count,
        },
    };

    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| DiskError::SerializationFailed { source: e })?;
    write_file_map_err(&metadata_path, metadata_json).await?;

    Ok(())
}

pub async fn load_project(project_name: &str) -> Result<Snapshot> {
    let snapshot_file_path = get_snapshot_file_path(project_name).await?;

    if !snapshot_file_path.exists() {
        return Err(DiskError::ProjectNotFound {
            project_name: project_name.to_string(),
            path: snapshot_file_path,
        });
    }

    let snapshot_json = read_to_string_map_err(&snapshot_file_path).await?;

    let snapshot: Snapshot =
        serde_json::from_str(&snapshot_json).map_err(|e| DiskError::DeserializationFailed {
            path: snapshot_file_path.clone(),
            source: e,
        })?;

    Ok(snapshot)
}

pub async fn list_projects() -> Result<Vec<ProjectInfo>> {
    let projects_dir = get_projects_dir().await?;
    let mut projects = Vec::new();
    let mut read_dir = read_dir_map_err(&projects_dir).await?;

    while let Some(entry) = next_entry_map_err(&mut read_dir, &projects_dir).await? {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    let snapshot_path = get_snapshot_file_path(name_str).await?;
                    if snapshot_path.exists() {
                        let metadata = read_project_metadata(name_str).await?;
                        let project_info = if let Some(m) = metadata {
                            ProjectInfo {
                                name: name_str.to_string(),
                                created_at: Some(m.created_at),
                                updated_at: Some(m.updated_at),
                                tempo: m.tempo,
                                line_count: m.line_count,
                            }
                        } else {
                            ProjectInfo {
                                name: name_str.to_string(),
                                created_at: None,
                                updated_at: None,
                                tempo: None,
                                line_count: None,
                            }
                        };
                        projects.push(project_info);
                    }
                }
            }
        }
    }

    projects.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(projects)
}

pub async fn delete_project(project_name: &str) -> Result<()> {
    let project_path = get_project_path(project_name).await?;

    match check_path_metadata_map_err(&project_path).await {
        Ok(_) => {
            remove_dir_all_map_err(&project_path).await?;
            Ok(())
        }
        Err(DiskError::PathMetadataCheckFailed { source, .. })
            if source.kind() == ErrorKind::NotFound =>
        {
            Ok(())
        }
        Err(e) => Err(e),
    }
}
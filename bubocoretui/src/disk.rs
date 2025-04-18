use directories::UserDirs;
use std::path::PathBuf;
use bubocorelib::server::Snapshot;
use std::{fmt, io, error::Error};
use tokio::fs;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};
use serde_json;

/// Custom error types for disk operations using only std library.
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
    SerializationFailed { source: serde_json::Error },
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
}

impl fmt::Display for DiskError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiskError::DirectoryResolutionFailed => write!(f, "Could not determine project directories"),
            DiskError::DirectoryCreationFailed { path, .. } => write!(f, "Failed to create directory '{}'", path.display()),
            DiskError::DirectoryReadFailed { path, .. } => write!(f, "Failed to read directory '{}'", path.display()),
            DiskError::DirectoryEntryReadFailed { path, .. } => write!(f, "Failed to read directory entry in '{}'", path.display()),
            DiskError::FileWriteFailed { path, .. } => write!(f, "Failed to write file '{}'", path.display()),
            DiskError::FileReadFailed { path, .. } => write!(f, "Failed to read file '{}'", path.display()),
            DiskError::SerializationFailed { .. } => write!(f, "Failed to serialize data"),
            DiskError::DeserializationFailed { path, .. } => write!(f, "Failed to deserialize data from '{}'", path.display()),
            DiskError::ProjectNotFound { project_name, path } => write!(f, "Project '{}' not found at '{}'", project_name, path.display()),
            DiskError::ProjectDeletionFailed { path, .. } => write!(f, "Failed to delete project directory '{}'", path.display()),
        }
    }
}

impl Error for DiskError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DiskError::DirectoryCreationFailed { source, .. } |
            DiskError::DirectoryReadFailed { source, .. } |
            DiskError::DirectoryEntryReadFailed { source, .. } |
            DiskError::FileWriteFailed { source, .. } |
            DiskError::ProjectDeletionFailed { source, .. } |
            DiskError::FileReadFailed { source, .. } => Some(source),
            DiskError::SerializationFailed { source, .. } |
            DiskError::DeserializationFailed { source, .. } => Some(source),
            DiskError::DirectoryResolutionFailed |
            DiskError::ProjectNotFound { .. } => None,
        }
    }
}

/// Metadata associated with a saved project.
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ProjectMetadata {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tempo: Option<f32>,
    line_count: Option<usize>,
}

/// Alias for Result using our custom DiskError.
type Result<T> = std::result::Result<T, DiskError>;

/// Returns the path to the base configuration/data directory for BuboCore.
/// Creates the directory if it doesn't exist.
///
/// Uses $HOME/.config/bubocore on Linux/macOS and Windows.
/// (Note: Using .config on Windows is non-standard, but follows the explicit request).
async fn get_base_config_dir() -> Result<PathBuf> {
    let path = UserDirs::new()
        .map(|ud| ud.home_dir().join(".config").join("bubocore"))
        .ok_or(DiskError::DirectoryResolutionFailed)?;

    fs::create_dir_all(&path)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed { path: path.clone(), source: e })?;
    Ok(path)
}

/// Returns the path to the 'projects' subdirectory within the base config directory.
/// Creates the directory if it doesn't exist.
async fn get_projects_dir() -> Result<PathBuf> {
    let base_dir = get_base_config_dir().await?;
    let projects_dir = base_dir.join("projects");
    fs::create_dir_all(&projects_dir)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed { path: projects_dir.clone(), source: e })?;
    Ok(projects_dir)
}

/// Returns the path for a specific project directory.
/// Does NOT create the directory.
async fn get_project_path(project_name: &str) -> Result<PathBuf> {
    let projects_dir = get_projects_dir().await?;
    Ok(projects_dir.join(project_name))
}

/// Returns the path to the 'scripts' subdirectory within a specific project directory.
/// Does NOT create the directory.
async fn get_project_scripts_dir(project_name: &str) -> Result<PathBuf> {
    let project_path = get_project_path(project_name).await?;
    Ok(project_path.join("scripts"))
}

/// Returns the path to the snapshot file within a specific project directory.
/// Example: ~/.config/bubocore/projects/my_project/snapshot.bubo
async fn get_snapshot_file_path(project_name: &str) -> Result<PathBuf> {
    let project_path = get_project_path(project_name).await?;
    Ok(project_path.join(format!("{}.bubo", project_name)))
}

/// Returns the path to the metadata file within a specific project directory.
async fn get_metadata_path(project_name: &str) -> Result<PathBuf> {
    let project_path = get_project_path(project_name).await?;
    Ok(project_path.join("metadata.json"))
}

/// Saves the complete session snapshot to disk for a given project name.
///
/// This creates:
/// - A main snapshot file `~/.config/bubocore/projects/<project_name>/<project_name>.bubo` (JSON blob)
/// - Individual script files in `~/.config/bubocore/projects/<project_name>/scripts/line{}_frame{}.{lang}`
/// - A metadata.json file with timestamps
///
/// # Arguments
/// * `snapshot` - The `Snapshot` data received from the server.
/// * `project_name` - The name for the project directory and snapshot file.
pub async fn save_project(snapshot: &Snapshot, project_name: &str) -> Result<()> {
    // 1. Ensure project directory exists
    let project_path = get_project_path(project_name).await?;
    fs::create_dir_all(&project_path)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed { path: project_path.clone(), source: e })?;

    // 2. Save the main snapshot blob (.bubo file)
    let snapshot_file_path = get_snapshot_file_path(project_name).await?;
    let snapshot_json = serde_json::to_string_pretty(snapshot)
        .map_err(|e| DiskError::SerializationFailed { source: e })?;
    fs::write(&snapshot_file_path, snapshot_json)
        .await
        .map_err(|e| DiskError::FileWriteFailed { path: snapshot_file_path.clone(), source: e })?;

    // 3. Save individual scripts
    let scripts_dir = get_project_scripts_dir(project_name).await?;
    fs::create_dir_all(&scripts_dir)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed { path: scripts_dir.clone(), source: e })?;

    for (line_idx, line) in snapshot.scene.lines.iter().enumerate() {
        for script_arc in &line.scripts {
            let script = &**script_arc;
            if !script.content.is_empty() {
                let script_filename = format!(
                    "line{}_frame{}.{}",
                    line_idx,
                    script.index,
                    if script.lang.is_empty() { "txt" } else { &script.lang }
                );
                let script_path = scripts_dir.join(script_filename);
                fs::write(&script_path, &script.content)
                    .await
                    .map_err(|e| DiskError::FileWriteFailed { path: script_path.clone(), source: e })?;
            }
        }
    }

    // 4. Save/Update Metadata
    let metadata_path = get_metadata_path(project_name).await?;
    let now = Utc::now();
    // Extract extra info from snapshot
    let tempo = Some(snapshot.tempo as f32);
    let line_count = Some(snapshot.scene.lines.len());

    let metadata: ProjectMetadata = match fs::read_to_string(&metadata_path).await {
        Ok(content) => {
            // Try to parse existing metadata using serde_json
            match serde_json::from_str::<ProjectMetadata>(&content) {
                Ok(mut existing_meta) => {
                    // Successfully parsed, update 'updated_at' and other fields
                    existing_meta.updated_at = now;
                    existing_meta.tempo = tempo;
                    existing_meta.line_count = line_count;
                    existing_meta
                }
                Err(_) => {
                    // Failed to parse, create new metadata (overwrite corrupt file)
                    ProjectMetadata { created_at: now, updated_at: now, tempo, line_count }
                }
            }
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            // Metadata file doesn't exist, create new
            ProjectMetadata { created_at: now, updated_at: now, tempo, line_count }
        }
        Err(e) => {
            // Other file read error
            return Err(DiskError::FileReadFailed { path: metadata_path, source: e });
        }
    };

    // Write the metadata back to the file using serde_json
    let metadata_json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| DiskError::SerializationFailed { source: e })?;
    fs::write(&metadata_path, metadata_json)
        .await
        .map_err(|e| DiskError::FileWriteFailed { path: metadata_path, source: e })?;

    Ok(())
}

/// Loads a session snapshot from disk for a given project name.
///
/// Reads the `~/.config/bubocore/projects/<project_name>/<project_name>.bubo` file.
/// Note: This function only loads the data. Applying it to the server 
/// (sending ClientMessages) must be handled separately by the caller.
///
/// # Arguments
/// * `project_name` - The name of the project to load.
///
/// # Returns
/// A `Result` containing the loaded `Snapshot` if successful.
pub async fn load_project(project_name: &str) -> Result<Snapshot> {
    let snapshot_file_path = get_snapshot_file_path(project_name).await?;

    if !snapshot_file_path.exists() {
        return Err(DiskError::ProjectNotFound {
            project_name: project_name.to_string(),
            path: snapshot_file_path,
        });
    }

    let snapshot_json = fs::read_to_string(&snapshot_file_path)
        .await
        .map_err(|e| DiskError::FileReadFailed { path: snapshot_file_path.clone(), source: e })?;

    let snapshot: Snapshot = serde_json::from_str(&snapshot_json)
        .map_err(|e| DiskError::DeserializationFailed { path: snapshot_file_path.clone(), source: e })?;

    Ok(snapshot)
}

/// Lists the names and metadata of all saved projects found in the projects directory.
pub async fn list_projects() -> Result<Vec<(String, Option<DateTime<Utc>>, Option<DateTime<Utc>>, Option<f32>, Option<usize>)>> {
    let projects_dir = get_projects_dir().await?;
    let mut projects = Vec::new();
    let mut read_dir = fs::read_dir(&projects_dir).await.map_err(|e| DiskError::DirectoryReadFailed { path: projects_dir.clone(), source: e })?;

    while let Some(entry) = read_dir.next_entry().await
        .map_err(|e| DiskError::DirectoryEntryReadFailed { path: projects_dir.clone(), source: e })? {
        let path = entry.path();
        if path.is_dir() {
            if let Some(name) = path.file_name() {
                if let Some(name_str) = name.to_str() {
                    let snapshot_path = get_snapshot_file_path(name_str).await?;
                    if snapshot_path.exists() {
                        // Try to load metadata
                        let metadata_path = get_metadata_path(name_str).await?;
                        let metadata_result = fs::read_to_string(&metadata_path).await;

                        let (created_at, updated_at, tempo, line_count) = match metadata_result {
                            Ok(content) => {
                                // Use serde_json
                                match serde_json::from_str::<ProjectMetadata>(&content) {
                                    Ok(meta) => (Some(meta.created_at), Some(meta.updated_at), meta.tempo, meta.line_count),
                                    Err(_) => (None, None, None, None), // Metadata file corrupt or invalid format
                                }
                            }
                            Err(_) => (None, None, None, None), // Metadata file not found or other read error
                        };
                        projects.push((name_str.to_string(), created_at, updated_at, tempo, line_count));
                    }
                }
            }
        }
    }

    // Sort projects alphabetically by name for consistency
    projects.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(projects)
}

/// Deletes a project and all its associated files (snapshot and scripts).
///
/// Removes the entire directory `~/.config/bubocore/projects/<project_name>`.
/// This operation is idempotent: if the project directory doesn't exist, it returns `Ok(())`.
///
/// # Arguments
/// * `project_name` - The name of the project to delete.
///
/// # Returns
/// A `Result<()>` indicating success or failure.
pub async fn delete_project(project_name: &str) -> Result<()> {
    let project_path = get_project_path(project_name).await?;

    // Check if the directory exists. Use metadata check which works for dirs/files.
    match fs::metadata(&project_path).await {
        Ok(_) => {
            // Directory exists, proceed with recursive deletion
            fs::remove_dir_all(&project_path)
                .await
                .map_err(|e| DiskError::ProjectDeletionFailed { path: project_path.clone(), source: e })?;
            Ok(())
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Ok(())
        }
        Err(e) => {
            // Some other error occurred trying to access the path (e.g., permissions)
            // We can map this to DirectoryReadFailed or similar existing error.
             Err(DiskError::DirectoryReadFailed { path: project_path.clone(), source: e })
        }
    }
}

use chrono::{DateTime, Utc};
use corelib::server::Snapshot;
use directories::UserDirs;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{error::Error, fmt, io, path::Path, str::FromStr};
use tokio::{
    fs::{self, DirEntry, ReadDir},
    io::ErrorKind,
};

#[derive(Debug)]
/// Error type for disk operations in BuboCoreTUI
///
/// This enum represents various errors that can occur during disk operations,
/// including file system operations, serialization, and project management.
/// Each variant contains relevant context about the error, such as file paths
/// and underlying error sources.
///
/// # Variants
///
/// * `DirectoryResolutionFailed` - Failed to determine project directories
/// * `DirectoryCreationFailed` - Failed to create a directory
///     * `path` - Path where creation was attempted
///     * `source` - Underlying IO error
/// * `DirectoryReadFailed` - Failed to read a directory
///     * `path` - Path that couldn't be read
///     * `source` - Underlying IO error
/// * `DirectoryEntryReadFailed` - Failed to read a directory entry
///     * `path` - Path where read failed
///     * `source` - Underlying IO error
/// * `FileWriteFailed` - Failed to write to a file
///     * `path` - Path where write failed
///     * `source` - Underlying IO error
/// * `FileReadFailed` - Failed to read from a file
///     * `path` - Path where read failed
///     * `source` - Underlying IO error
/// * `SerializationFailed` - Failed to serialize data
///     * `source` - Underlying serde_json error
/// * `DeserializationFailed` - Failed to deserialize data
///     * `path` - Path where deserialization failed
///     * `source` - Underlying serde_json error
/// * `ProjectNotFound` - Requested project not found
///     * `project_name` - Name of the missing project
///     * `path` - Path where project was searched
/// * `ProjectDeletionFailed` - Failed to delete a project
///     * `path` - Path where deletion failed
///     * `source` - Underlying IO error
/// * `PathMetadataCheckFailed` - Failed to check path metadata
///     * `path` - Path where check failed
///     * `source` - Underlying IO error
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum EditingMode {
    #[default]
    Normal,
    Vim,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum Theme {
    #[default]
    Classic,
    Ocean,
    Forest,
}

impl fmt::Display for EditingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EditingMode::Normal => write!(f, "normal"),
            EditingMode::Vim => write!(f, "vim"),
        }
    }
}

impl fmt::Display for Theme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Theme::Classic => write!(f, "classic"),
            Theme::Ocean => write!(f, "ocean"),
            Theme::Forest => write!(f, "forest"),
        }
    }
}

#[derive(Debug)]
pub struct ParseEditingModeError;

impl fmt::Display for ParseEditingModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid editing mode")
    }
}

impl Error for ParseEditingModeError {}

impl FromStr for EditingMode {
    type Err = ParseEditingModeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(EditingMode::Normal),
            "vim" => Ok(EditingMode::Vim),
            _ => Err(ParseEditingModeError),
        }
    }
}

#[derive(Debug)]
pub struct ParseThemeError;

impl fmt::Display for ParseThemeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid theme")
    }
}

impl Error for ParseThemeError {}

impl FromStr for Theme {
    type Err = ParseThemeError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "classic" => Ok(Theme::Classic),
            "ocean" => Ok(Theme::Ocean),
            "forest" => Ok(Theme::Forest),
            _ => Err(ParseThemeError),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Configuration settings for the BuboCoreTUI client.
///
/// Stores user preferences like editing mode, connection details, and durations.
/// Persisted in `client_config.json` within the base config directory.
pub struct ClientConfig {
    #[serde(default)]
    pub editing_mode: EditingMode,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub last_ip_address: Option<String>,
    #[serde(default)]
    pub last_port: Option<u16>,
    #[serde(default)]
    pub last_username: Option<String>,
    /// Duration of the sketch in seconds. Defaults to 300 (5 minutes).
    #[serde(default = "default_sketch_duration")]
    pub sketch_duration_secs: u64,
    /// Time before the screensaver activates in seconds. Defaults to 60.
    #[serde(default = "default_screensaver_timeout")]
    pub screensaver_timeout_secs: u64,
    /// Whether the screensaver is enabled. Defaults to true.
    #[serde(default = "default_screensaver_enabled")]
    pub screensaver_enabled: bool,
}

// Fonctions pour fournir les valeurs par défaut pour serde
fn default_sketch_duration() -> u64 {
    300 // 5 minutes
}

fn default_screensaver_timeout() -> u64 {
    60 // 1 minute
}

fn default_screensaver_enabled() -> bool {
    true
}

impl Default for ClientConfig {
    fn default() -> Self {
        ClientConfig {
            editing_mode: EditingMode::default(),
            theme: Theme::default(),
            last_ip_address: None,
            last_port: None,
            last_username: None,
            sketch_duration_secs: default_sketch_duration(), // Utiliser la fonction par défaut
            screensaver_timeout_secs: default_screensaver_timeout(), // Utiliser la fonction par défaut
            screensaver_enabled: default_screensaver_enabled(), // Utiliser la fonction par défaut
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
/// Metadata for a BuboCore project.
///
/// This struct stores essential information about a project, including:
/// - Creation and modification timestamps
/// - Project tempo (if specified)
/// - Number of lines in the project (if known)
///
/// # Fields
///
/// * `created_at` - The UTC timestamp when the project was created
/// * `updated_at` - The UTC timestamp when the project was last modified
/// * `tempo` - Optional tempo value for the project (in BPM)
/// * `line_count` - Optional count of lines in the project
///
/// # Examples
///
/// ```rust
/// let metadata = ProjectMetadata {
///     created_at: Utc::now(),
///     updated_at: Utc::now(),
///     tempo: Some(120.0),
///     line_count: Some(16),
/// };
/// ```
struct ProjectMetadata {
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    tempo: Option<f32>,
    line_count: Option<usize>,
}

/// Alias for Result using our custom DiskError.
type Result<T> = std::result::Result<T, DiskError>;

/// Creates all directories in the specified path, including any necessary parent directories.
///
/// This is a wrapper around `fs::create_dir_all` that maps the standard IO error
/// to our custom `DiskError::DirectoryCreationFailed` type.
///
/// # Arguments
///
/// * `path` - The path where directories should be created
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if successful, or a `DiskError` if creation fails
///
/// # Errors
///
/// Returns `DiskError::DirectoryCreationFailed` if directory creation fails,
/// containing both the path that failed and the underlying IO error.
async fn create_dir_all_map_err(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .await
        .map_err(|e| DiskError::DirectoryCreationFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

/// Writes contents to a file at the specified path, mapping any IO errors to our custom DiskError type.
///
/// This is a wrapper around `fs::write` that maps the standard IO error
/// to our custom `DiskError::FileWriteFailed` type.
///
/// # Arguments
///
/// * `path` - The path where the file should be written
/// * `contents` - The contents to write to the file, which can be any type that implements `AsRef<[u8]>`
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if successful, or a `DiskError` if writing fails
///
/// # Errors
///
/// Returns `DiskError::FileWriteFailed` if file writing fails,
/// containing both the path that failed and the underlying IO error.
///
/// # Examples
///
/// ```rust
/// let path = Path::new("example.txt");
/// let contents = "Hello, world!";
/// write_file_map_err(path, contents).await?;
/// ```
async fn write_file_map_err<C: AsRef<[u8]>>(path: &Path, contents: C) -> Result<()> {
    fs::write(path, contents)
        .await
        .map_err(|e| DiskError::FileWriteFailed {
            path: path.to_path_buf(),
            source: e,
        })
}

/// Reads the contents of a file as a string, mapping any IO errors to our custom DiskError type.
///
/// This is a wrapper around `fs::read_to_string` that maps the standard IO error
/// to our custom `DiskError::FileReadFailed` type.
///
/// # Arguments
///
/// * `path` - The path of the file to read
///
/// # Returns
///
/// * `Result<String>` - The file contents as a string if successful, or a `DiskError` if reading fails
///
/// # Errors
///
/// Returns `DiskError::FileReadFailed` if file reading fails,
/// containing both the path that failed and the underlying IO error.
///
/// # Examples
///
/// ```rust
/// let path = Path::new("example.txt");
/// let contents = read_to_string_map_err(path).await?;
/// println!("File contents: {}", contents);
/// ```
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

/// Returns the path to the base configuration/data directory for BuboCore.
/// Creates the directory if it doesn't exist.
///
/// Uses $HOME/.config/bubocore on Linux/macOS and Windows.
/// (Note: Using .config on Windows is non-standard, but follows the explicit request).
async fn get_base_config_dir() -> Result<PathBuf> {
    let path = UserDirs::new()
        .map(|ud| ud.home_dir().join(".config").join("bubocore"))
        .ok_or(DiskError::DirectoryResolutionFailed)?;

    create_dir_all_map_err(&path).await?;
    Ok(path)
}

/// Returns the path to the client configuration file.
/// Example: ~/.config/bubocore/client_config.json
async fn get_client_config_path() -> Result<PathBuf> {
    let base_dir = get_base_config_dir().await?;
    Ok(base_dir.join("client_config.json"))
}

/// Returns the path to the 'projects' subdirectory within the base config directory.
/// Creates the directory if it doesn't exist.
async fn get_projects_dir() -> Result<PathBuf> {
    let base_dir = get_base_config_dir().await?;
    let projects_dir = base_dir.join("projects");
    create_dir_all_map_err(&projects_dir).await?;
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
        for script_arc in &line.scripts {
            let script = &**script_arc;
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

    let snapshot_json = read_to_string_map_err(&snapshot_file_path).await?;

    let snapshot: Snapshot =
        serde_json::from_str(&snapshot_json).map_err(|e| DiskError::DeserializationFailed {
            path: snapshot_file_path.clone(),
            source: e,
        })?;

    Ok(snapshot)
}

/// Lists the names and metadata of all saved projects found in the projects directory.
pub async fn list_projects() -> Result<
    Vec<(
        String,
        Option<DateTime<Utc>>,
        Option<DateTime<Utc>>,
        Option<f32>,
        Option<usize>,
    )>,
> {
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
                        let (created_at, updated_at, tempo, line_count) = metadata
                            .map(|m| {
                                (
                                    Some(m.created_at),
                                    Some(m.updated_at),
                                    m.tempo,
                                    m.line_count,
                                )
                            })
                            .unwrap_or((None, None, None, None));

                        projects.push((
                            name_str.to_string(),
                            created_at,
                            updated_at,
                            tempo,
                            line_count,
                        ));
                    }
                }
            }
        }
    }

    projects.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(projects)
}

/// Deletes a project directory and all its contents.
///
/// This function attempts to delete a project directory and all its contents.
/// If the project directory doesn't exist, it returns Ok(()). If the directory
/// exists but deletion fails, it returns an error.
///
/// # Arguments
///
/// * `project_name` - The name of the project to delete
///
/// # Returns
///
/// * `Result<()>` - Ok(()) if the project was successfully deleted or didn't exist,
///   or a `DiskError` if deletion fails for any other reason
///
/// # Errors
///
/// Returns `DiskError::ProjectDeletionFailed` if the project directory exists
/// but cannot be deleted. Returns other `DiskError` variants if path resolution
/// or metadata checks fail.
///
/// # Examples
///
/// ```rust
/// // Delete an existing project
/// delete_project("my_project").await?;
///
/// // Attempting to delete a non-existent project is not an error
/// delete_project("nonexistent").await?;
/// ```
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

/// Reads the client configuration from disk.
///
/// If the configuration file doesn't exist or is invalid, returns the default configuration.
pub async fn read_client_config() -> Result<ClientConfig> {
    let config_path = get_client_config_path().await?;

    match read_to_string_map_err(&config_path).await {
        Ok(content) => {
            serde_json::from_str(&content).map_err(|e| DiskError::DeserializationFailed {
                path: config_path.clone(),
                source: e,
            })
        }
        Err(DiskError::FileReadFailed { source, .. }) if source.kind() == ErrorKind::NotFound => {
            Ok(ClientConfig::default()) // Return default config if file not found
        }
        Err(e) => Err(e), // Propagate other read errors
    }
}

/// Writes the client configuration to disk.
///
/// # Arguments
/// * `config` - The `ClientConfig` struct to save.
pub async fn write_client_config(config: &ClientConfig) -> Result<()> {
    let config_path = get_client_config_path().await?;
    let config_json = serde_json::to_string_pretty(config)
        .map_err(|e| DiskError::SerializationFailed { source: e })?;
    write_file_map_err(&config_path, config_json).await
}

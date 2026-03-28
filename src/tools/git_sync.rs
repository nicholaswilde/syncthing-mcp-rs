//! Git-Sync tools for version control of SyncThing configurations.

use crate::api::models::Config;
use crate::error::Result;

/// Exporter for SyncThing configurations to diffable formats.
pub struct ConfigExporter {
    config: Config,
}

impl ConfigExporter {
    /// Creates a new configuration exporter.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Masks sensitive information in the configuration.
    ///
    /// This currently masks:
    /// - GUI user, password, and apiKey
    /// - LDAP password
    pub fn mask_sensitive(&mut self) {
        // Mask GUI sensitive fields
        if let Some(gui) = self.config.gui.as_object_mut() {
            if gui.contains_key("user") {
                gui.insert(
                    "user".to_string(),
                    serde_json::Value::String("********".to_string()),
                );
            }
            if gui.contains_key("password") {
                gui.insert(
                    "password".to_string(),
                    serde_json::Value::String("********".to_string()),
                );
            }
            if gui.contains_key("apiKey") {
                gui.insert(
                    "apiKey".to_string(),
                    serde_json::Value::String("********".to_string()),
                );
            }
        }

        // Mask LDAP sensitive fields
        if let Some(ldap) = self.config.ldap.as_object_mut()
            && ldap.contains_key("password")
        {
            ldap.insert(
                "password".to_string(),
                serde_json::Value::String("********".to_string()),
            );
        }
    }

    /// Exports the configuration to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.config)?)
    }

    /// Exports the configuration to a YAML string.
    pub fn to_yaml(&self) -> Result<String> {
        Ok(serde_yaml_ng::to_string(&self.config)?)
    }
}

/// A Git client for managing configuration backups.
pub struct GitClient {
    repo_path: std::path::PathBuf,
}

impl GitClient {
    /// Creates a new Git client for the given repository path.
    pub fn new(repo_path: std::path::PathBuf) -> Self {
        Self { repo_path }
    }

    /// Initializes a new Git repository at the target path.
    pub async fn init(&self) -> Result<()> {
        // Create directory if it doesn't exist
        if !self.repo_path.exists() {
            std::fs::create_dir_all(&self.repo_path).map_err(|e| crate::error::Error::Internal(format!("Failed to create repo directory: {}", e)))?;
        }
        self.run_command(&["init"]).await?;
        Ok(())
    }

    /// Adds a file to the Git index.
    pub async fn add(&self, file_path: &str) -> Result<()> {
        self.run_command(&["add", file_path]).await?;
        Ok(())
    }

    /// Commits changes to the repository.
    /// Returns the commit hash.
    pub async fn commit(&self, message: &str) -> Result<String> {
        self.run_command(&["commit", "-m", message]).await?;
        let output = self.run_command(&["rev-parse", "HEAD"]).await?;
        Ok(output.trim().to_string())
    }

    /// Runs an arbitrary Git command and returns its output.
    pub async fn run_command(&self, args: &[&str]) -> Result<String> {
        let output = std::process::Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| crate::error::Error::Internal(format!("Failed to execute git command: {}", e)))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(crate::error::Error::Internal(format!(
                "Git command failed: git {:?} - {}",
                args, stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Orchestrates configuration backups to Git.
pub struct GitSyncManager {
    git: GitClient,
    repo_path: std::path::PathBuf,
}

impl GitSyncManager {
    /// Creates a new Git sync manager for the given repository path.
    pub fn new(repo_path: std::path::PathBuf) -> Self {
        Self {
            git: GitClient::new(repo_path.clone()),
            repo_path,
        }
    }

    /// Initializes the Git repository for backups.
    pub async fn init(&self) -> Result<()> {
        self.git.init().await
    }

    /// Backs up a configuration to the Git repository.
    /// Returns the commit hash.
    pub async fn backup_config(&self, config: Config) -> Result<String> {
        let mut exporter = ConfigExporter::new(config);
        exporter.mask_sensitive();

        let json_content = exporter.to_json()?;
        let yaml_content = exporter.to_yaml()?;

        let json_path = self.repo_path.join("config.json");
        let yaml_path = self.repo_path.join("config.yaml");

        std::fs::write(&json_path, json_content)
            .map_err(|e| crate::error::Error::Internal(format!("Failed to write config.json: {}", e)))?;
        std::fs::write(&yaml_path, yaml_content)
            .map_err(|e| crate::error::Error::Internal(format!("Failed to write config.yaml: {}", e)))?;

        self.git.add("config.json").await?;
        self.git.add("config.yaml").await?;
        
        // Use a generic commit message for now
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.git.commit(&format!("Backup configuration: {}", timestamp)).await
    }
}

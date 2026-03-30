use crate::credentials::{AwsBackend, VaultBackend, register_backend, resolve_api_key};
use tracing::warn;
use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Application configuration.
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    /// The host to connect to (if only one instance).
    pub host: String,
    /// The port to connect to.
    pub port: u16,
    /// The API key to use.
    pub api_key: Option<String>,
    /// The MCP transport mode (e.g., "stdio").
    #[serde(default = "default_transport")]
    pub mcp_transport: String,
    /// The log level.
    #[serde(default = "default_log_level")]
    pub log_level: String,
    /// Whether to skip SSL certificate verification.
    #[serde(default = "default_no_verify_ssl")]
    pub no_verify_ssl: bool,
    /// Maximum number of retry attempts for API requests.
    #[serde(default = "default_retry_max_attempts")]
    pub retry_max_attempts: u32,
    /// Initial backoff for retries in milliseconds.
    #[serde(default = "default_retry_initial_backoff_ms")]
    pub retry_initial_backoff_ms: u64,
    /// Timeout for API requests in seconds.
    #[serde(default = "default_timeout_s")]
    pub timeout_s: u64,
    /// A list of SyncThing instances to manage.
    #[serde(default, deserialize_with = "deserialize_instances")]
    pub instances: Vec<InstanceConfig>,
    /// The HTTP server configuration for MCP.
    #[serde(default)]
    pub http_server: HttpServerConfig,
    /// The list of SyncThing event types to notify about.
    #[serde(default = "default_mcp_events")]
    pub mcp_events: Vec<String>,
    /// Bandwidth orchestration configuration.
    #[serde(default)]
    pub bandwidth: BandwidthConfig,
    /// Vault configuration.
    #[serde(default)]
    pub vault: VaultConfig,
    /// AWS Secrets Manager configuration.
    #[serde(default)]
    pub aws: AwsConfig,
}

/// Configuration for HashiCorp Vault.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct VaultConfig {
    /// Whether Vault is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// The Vault server address.
    #[serde(default = "default_vault_address")]
    pub address: String,
    /// The Vault token.
    pub token: Option<String>,
    /// The Vault KV mount point.
    #[serde(default = "default_vault_mount")]
    pub mount: String,
}

fn default_vault_address() -> String {
    "http://127.0.0.1:8200".to_string()
}

fn default_vault_mount() -> String {
    "secret".to_string()
}

/// Configuration for AWS Secrets Manager.
#[derive(Debug, Deserialize, Clone, Default)]
pub struct AwsConfig {
    /// Whether AWS is enabled.
    #[serde(default)]
    pub enabled: bool,
    /// The AWS region.
    #[serde(default = "default_aws_region")]
    pub region: String,
    /// The AWS profile name.
    pub profile: Option<String>,
    /// The AWS endpoint URL (useful for LocalStack).
    pub endpoint_url: Option<String>,
}

fn default_aws_region() -> String {
    "us-east-1".to_string()
}

/// Bandwidth limits to apply.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct BandwidthLimits {
    /// Maximum receive rate in Kbps.
    pub max_recv_kbps: Option<i64>,
    /// Maximum send rate in Kbps.
    pub max_send_kbps: Option<i64>,
}

/// A performance profile that defines bandwidth limits.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceProfile {
    /// Name of the profile (e.g., "working_hours").
    pub name: String,
    /// Bandwidth limits for this profile.
    pub limits: BandwidthLimits,
}

/// A schedule for when to apply a performance profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProfileSchedule {
    /// Profile to apply.
    pub profile_name: String,
    /// Days of the week this schedule applies to.
    pub days: Vec<String>,
    /// Start time in 24h format (e.g., "09:00").
    pub start_time: String,
    /// End time in 24h format (e.g., "17:00").
    pub end_time: String,
}

/// Configuration for bandwidth orchestration.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct BandwidthConfig {
    /// Available performance profiles.
    pub profiles: Vec<PerformanceProfile>,
    /// Schedules for applying profiles.
    pub schedules: Vec<ProfileSchedule>,
    /// The name of the currently active profile, if any.
    pub active_profile: Option<String>,
}

/// Configuration for the HTTP/SSE server.
#[derive(Debug, Deserialize, Clone)]
pub struct HttpServerConfig {
    /// Whether the HTTP server is enabled.
    #[serde(default = "default_http_server_enabled")]
    pub enabled: bool,
    /// The host to bind the HTTP server to.
    #[serde(default = "default_http_server_host")]
    pub host: String,
    /// The port to bind the HTTP server to.
    #[serde(default = "default_http_server_port")]
    pub port: u16,
    /// Optional API key for basic authentication.
    pub api_key: Option<String>,
}

/// Configuration for a specific SyncThing instance.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct InstanceConfig {
    /// The name of the instance.
    pub name: Option<String>,
    /// The base URL of the instance.
    pub url: String,
    /// The API key for this instance.
    pub api_key: Option<String>,
    /// Whether to skip SSL certificate verification for this instance.
    pub no_verify_ssl: Option<bool>,
    /// Maximum number of retry attempts for this instance.
    pub retry_max_attempts: Option<u32>,
    /// Initial backoff for retries for this instance in milliseconds.
    pub retry_initial_backoff_ms: Option<u64>,
    /// Timeout for this instance in seconds.
    pub timeout_s: Option<u64>,
}

fn default_transport() -> String {
    "stdio".to_string()
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_no_verify_ssl() -> bool {
    true
}

fn default_retry_max_attempts() -> u32 {
    3
}

fn default_retry_initial_backoff_ms() -> u64 {
    100
}

fn default_timeout_s() -> u64 {
    30
}

fn default_http_server_enabled() -> bool {
    false
}

fn default_http_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_http_server_port() -> u16 {
    3000
}

fn default_mcp_events() -> Vec<String> {
    vec![
        "FolderStateChanged".to_string(),
        "DeviceConnected".to_string(),
        "DeviceDisconnected".to_string(),
        "LocalIndexUpdated".to_string(),
    ]
}

impl Default for HttpServerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            host: "0.0.0.0".to_string(),
            port: 3000,
            api_key: None,
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8384,
            api_key: None,
            mcp_transport: "stdio".to_string(),
            log_level: "info".to_string(),
            no_verify_ssl: true,
            retry_max_attempts: 3,
            retry_initial_backoff_ms: 100,
            timeout_s: 30,
            instances: Vec::new(),
            http_server: HttpServerConfig::default(),
            mcp_events: default_mcp_events(),
            bandwidth: BandwidthConfig::default(),
            vault: VaultConfig::default(),
            aws: AwsConfig::default(),
        }
    }
}

fn deserialize_instances<'de, D>(deserializer: D) -> Result<Vec<InstanceConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{MapAccess, SeqAccess, Visitor};

    struct InstancesVisitor;

    impl<'de> Visitor<'de> for InstancesVisitor {
        type Value = Vec<InstanceConfig>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence or a map")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut vec = Vec::new();
            while let Some(value) = seq.next_element()? {
                vec.push(value);
            }
            Ok(vec)
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut btree_map = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, InstanceConfig>()? {
                btree_map.insert(key, value);
            }
            Ok(btree_map.into_values().collect())
        }
    }

    deserializer.deserialize_any(InstancesVisitor)
}

/// The result of a configuration load operation.
pub enum ConfigResult {
    /// A successfully loaded configuration.
    Config(Box<AppConfig>),
    /// The application should exit (e.g., after successful encryption).
    Exit,
}

impl AppConfig {
    /// Loads the configuration from file, environment variables, and CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or is invalid.
    pub async fn load(
        file_path: Option<String>,
        cli_args: Vec<String>,
    ) -> Result<ConfigResult, ConfigError> {
        let mut builder = Config::builder();
        let matches = parse_args(cli_args);

        if let Some(matches) = matches.subcommand_matches("encrypt") {
            let value = matches.get_one::<String>("value").unwrap();
            match crate::credentials::encrypt_value(value) {
                Some(encrypted) => {
                    println!("{}", encrypted);
                }
                None => {
                    eprintln!("Failed to encrypt value");
                }
            }
            return Ok(ConfigResult::Exit);
        }

        // 1. Determine Config File Path
        let path_to_load = if let Some(p) = file_path {
            Some(p)
        } else {
            matches.get_one::<String>("config").cloned()
        };

        // 2. Set Defaults
        builder = builder
            .set_default("mcp_transport", "stdio")?
            .set_default("log_level", "info")?
            .set_default("no_verify_ssl", true)?
            .set_default("host", "localhost")?
            .set_default("port", 8384)?
            .set_default("retry_max_attempts", 3)?
            .set_default("retry_initial_backoff_ms", 100)?
            .set_default("timeout_s", 30)?
            .set_default("http_server.enabled", false)?
            .set_default("http_server.host", "0.0.0.0")?
            .set_default("http_server.port", 3000)?
            .set_default("mcp_events", default_mcp_events())?
            .set_default("vault.enabled", false)?
            .set_default("vault.address", "http://127.0.0.1:8200")?
            .set_default("vault.mount", "secret")?
            .set_default("aws.enabled", false)?
            .set_default("aws.region", "us-east-1")?;

        // 3. Load from File
        if let Some(path) = path_to_load {
            builder = builder.add_source(File::with_name(&path));
        } else {
            builder = builder.add_source(File::with_name("config").required(false));
            if let Ok(home) = std::env::var("HOME") {
                let path = format!("{}/.config/syncthing-mcp-rs/config", home);
                builder = builder.add_source(File::with_name(&path).required(false));
            }
        }

        // 4. Load from Environment Variables
        builder = builder.add_source(
            Environment::with_prefix("SYNCTHING")
                .prefix_separator("_")
                .separator("__")
                .try_parsing(true),
        );

        // 5. Apply CLI overrides
        if let Some(host) = matches.get_one::<String>("host") {
            builder = builder.set_override("host", host.as_str())?;
        }
        if let Some(port) = matches.get_one::<u16>("port") {
            builder = builder.set_override("port", *port)?;
        }
        if let Some(api_key) = matches.get_one::<String>("api_key") {
            builder = builder.set_override("api_key", api_key.as_str())?;
        }
        if let Some(transport) = matches.get_one::<String>("mcp_transport") {
            builder = builder.set_override("mcp_transport", transport.as_str())?;
        }
        if matches.get_flag("no_verify_ssl") {
            builder = builder.set_override("no_verify_ssl", true)?;
        }
        if let Some(level) = matches.get_one::<String>("log_level") {
            builder = builder.set_override("log_level", level.as_str())?;
        }
        if matches.get_flag("http_server_enabled") {
            builder = builder.set_override("http_server.enabled", true)?;
        }
        if let Some(host) = matches.get_one::<String>("http_server_host") {
            builder = builder.set_override("http_server.host", host.as_str())?;
        }
        if let Some(port) = matches.get_one::<u16>("http_server_port") {
            builder = builder.set_override("http_server.port", *port)?;
        }
        if let Some(key) = matches.get_one::<String>("http_server_api_key") {
            builder = builder.set_override("http_server.api_key", key.as_str())?;
        }
        if let Some(events) = matches.get_one::<String>("mcp_events") {
            let event_list: Vec<String> = events.split(',').map(|s| s.trim().to_string()).collect();
            builder = builder.set_override("mcp_events", event_list)?;
        }

        let mut config: AppConfig = builder.build()?.try_deserialize()?;
        config.validate().await.map_err(ConfigError::Message)?;
        Ok(ConfigResult::Config(Box::new(config)))
    }

    /// Validates the configuration and ensures at least one instance is configured.
    pub async fn validate(&mut self) -> Result<(), String> {
        // Register Vault backend if enabled
        if self.vault.enabled {
            if let Some(token) = &self.vault.token {
                let backend = VaultBackend::new(
                    self.vault.address.clone(),
                    token.clone(),
                    self.vault.mount.clone(),
                );
                register_backend("vault", Box::new(backend));
            } else {
                warn!("Vault is enabled but no token provided. Vault backend not registered.");
            }
        }

        // Register AWS backend if enabled
        if self.aws.enabled {
            let backend = AwsBackend::new(
                self.aws.region.clone(),
                self.aws.profile.clone(),
                self.aws.endpoint_url.clone(),
            ).await;
            register_backend("aws", Box::new(backend));
        }

        if self.instances.is_empty() && !self.host.is_empty() {
            let url = if self.host.starts_with("http") {
                self.host.clone()
            } else {
                format!("http://{}:{}", self.host, self.port)
            };

            self.instances.push(InstanceConfig {
                name: Some("default".to_string()),
                url,
                api_key: resolve_api_key(self.api_key.clone()).await,
                no_verify_ssl: Some(self.no_verify_ssl),
                retry_max_attempts: Some(self.retry_max_attempts),
                retry_initial_backoff_ms: Some(self.retry_initial_backoff_ms),
                timeout_s: Some(self.timeout_s),
            });
        }

        if self.instances.is_empty() {
            return Err("At least one SyncThing instance must be configured".to_string());
        }

        for (i, inst) in self.instances.iter_mut().enumerate() {
            if inst.url.is_empty() {
                return Err(format!("Instance {} is missing URL", i));
            }
            // Resolve API key if it's a keyring link
            inst.api_key = resolve_api_key(inst.api_key.take()).await;

            // Propagate global retry settings if not set on instance
            if inst.retry_max_attempts.is_none() {
                inst.retry_max_attempts = Some(self.retry_max_attempts);
            }
            if inst.retry_initial_backoff_ms.is_none() {
                inst.retry_initial_backoff_ms = Some(self.retry_initial_backoff_ms);
            }
            if inst.timeout_s.is_none() {
                inst.timeout_s = Some(self.timeout_s);
            }
        }

        Ok(())
    }

    /// Returns the instance configuration by name or index.
    pub fn get_instance(
        &self,
        name_or_index: Option<&str>,
    ) -> std::result::Result<&InstanceConfig, String> {
        match name_or_index {
            None => self
                .instances
                .first()
                .ok_or_else(|| "No instances configured".to_string()),
            Some(s) => {
                if let Some(inst) = s
                    .parse::<usize>()
                    .ok()
                    .and_then(|idx| self.instances.get(idx))
                {
                    return Ok(inst);
                }
                self.instances
                    .iter()
                    .find(|i| i.name.as_deref() == Some(s))
                    .ok_or_else(|| format!("Instance not found: {}", s))
            }
        }
    }
}

fn parse_args(args: Vec<String>) -> ArgMatches {
    use clap::{Arg, ArgAction, Command};

    let cmd = Command::new("syncthing-mcp-rs")
        .version(env!("CARGO_PKG_VERSION"))
        .subcommand(
            Command::new("encrypt")
                .about("Encrypt a sensitive value")
                .arg(Arg::new("value").required(true).help("Value to encrypt")),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file"),
        )
        .arg(Arg::new("host").long("host").help("SyncThing host"))
        .arg(
            Arg::new("port")
                .long("port")
                .help("SyncThing port")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("api_key")
                .long("api-key")
                .help("SyncThing API Key"),
        )
        .arg(
            Arg::new("mcp_transport")
                .long("transport")
                .help("Transport mode: stdio or http"),
        )
        .arg(
            Arg::new("http_server_enabled")
                .long("http-enabled")
                .action(ArgAction::SetTrue)
                .help("Enable the HTTP/SSE server"),
        )
        .arg(
            Arg::new("http_server_host")
                .long("http-host")
                .help("HTTP server host"),
        )
        .arg(
            Arg::new("http_server_port")
                .long("http-port")
                .help("HTTP server port")
                .value_parser(clap::value_parser!(u16)),
        )
        .arg(
            Arg::new("http_server_api_key")
                .long("http-api-key")
                .help("API key for the HTTP server"),
        )
        .arg(
            Arg::new("mcp_events")
                .long("events")
                .help("Comma-separated list of SyncThing events to notify about"),
        )
        .arg(
            Arg::new("no_verify_ssl")
                .long("no-verify-ssl")
                .action(ArgAction::SetTrue)
                .help("Disable SSL certificate verification"),
        )
        .arg(Arg::new("log_level").long("log-level").help("Log level"));

    if args.is_empty() {
        cmd.get_matches_from(vec!["syncthing-mcp-rs"])
    } else {
        cmd.get_matches_from(args)
    }
}

#[cfg(test)]
mod http_tests;
#[cfg(test)]
mod tests;

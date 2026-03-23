use crate::credentials::resolve_api_key;
use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
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
    /// A list of SyncThing instances to manage.
    #[serde(default, deserialize_with = "deserialize_instances")]
    pub instances: Vec<InstanceConfig>,
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
            instances: Vec::new(),
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
    Config(AppConfig),
    /// The application should exit (e.g., after successful encryption).
    Exit,
}

impl AppConfig {
    /// Loads the configuration from file, environment variables, and CLI arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration cannot be loaded or is invalid.
    pub fn load(
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
            .set_default("retry_initial_backoff_ms", 100)?;

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

        let mut config: AppConfig = builder.build()?.try_deserialize()?;
        config.validate().map_err(ConfigError::Message)?;
        Ok(ConfigResult::Config(config))
    }

    /// Validates the configuration and ensures at least one instance is configured.
    pub fn validate(&mut self) -> Result<(), String> {
        if self.instances.is_empty() && !self.host.is_empty() {
            let url = if self.host.starts_with("http") {
                self.host.clone()
            } else {
                format!("http://{}:{}", self.host, self.port)
            };

            self.instances.push(InstanceConfig {
                name: Some("default".to_string()),
                url,
                api_key: resolve_api_key(self.api_key.clone()),
                no_verify_ssl: Some(self.no_verify_ssl),
                retry_max_attempts: Some(self.retry_max_attempts),
                retry_initial_backoff_ms: Some(self.retry_initial_backoff_ms),
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
            inst.api_key = resolve_api_key(inst.api_key.take());

            // Propagate global retry settings if not set on instance
            if inst.retry_max_attempts.is_none() {
                inst.retry_max_attempts = Some(self.retry_max_attempts);
            }
            if inst.retry_initial_backoff_ms.is_none() {
                inst.retry_initial_backoff_ms = Some(self.retry_initial_backoff_ms);
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
mod tests {
    use super::*;
    use crate::test_utils::ENV_LOCK;

    #[test]
    fn test_load_defaults() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::remove_var("SYNCTHING_HOST");
            std::env::remove_var("SYNCTHING_PORT");
        }

        let config = match AppConfig::load(None, vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8384);
        assert_eq!(config.retry_max_attempts, 3);
        assert_eq!(config.retry_initial_backoff_ms, 100);
    }

    #[test]
    fn test_cli_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let args = vec![
            "app".to_string(),
            "--host".to_string(),
            "cli.com".to_string(),
            "--port".to_string(),
            "4000".to_string(),
        ];
        let config = match AppConfig::load(None, args).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };
        assert_eq!(config.host, "cli.com");
        assert_eq!(config.port, 4000);
    }

    #[test]
    fn test_env_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            std::env::set_var("SYNCTHING_HOST", "env.com");
            std::env::set_var("SYNCTHING_PORT", "5050");
        }

        let config = match AppConfig::load(None, vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };

        unsafe {
            std::env::remove_var("SYNCTHING_HOST");
            std::env::remove_var("SYNCTHING_PORT");
        }

        assert_eq!(config.host, "env.com");
        assert_eq!(config.port, 5050);
    }

    #[test]
    fn test_file_override() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use std::io::Write;
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            "host = \"file.com\"\nport = 6060\nretry_max_attempts = 5"
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };
        assert_eq!(config.host, "file.com");
        assert_eq!(config.port, 6060);
        assert_eq!(config.retry_max_attempts, 5);
    }

    #[test]
    fn test_multi_instance_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use std::io::Write;
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
host = "primary.com"
retry_max_attempts = 10

[[instances]]
name = "primary"
url = "http://192.168.1.1"
api_key = "key1"

[[instances]]
name = "secondary"
url = "http://192.168.1.2"
api_key = "key2"
no_verify_ssl = false
retry_max_attempts = 2
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };
        assert_eq!(config.instances.len(), 2);
        assert_eq!(config.instances[0].name, Some("primary".to_string()));
        assert_eq!(config.instances[0].url, "http://192.168.1.1");
        assert_eq!(config.instances[0].retry_max_attempts, Some(10));
        assert_eq!(config.instances[1].name, Some("secondary".to_string()));
        assert_eq!(config.instances[1].no_verify_ssl, Some(false));
        assert_eq!(config.instances[1].retry_max_attempts, Some(2));
    }

    #[test]
    fn test_multi_instance_map_loading() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        use std::io::Write;
        let mut file = tempfile::Builder::new().suffix(".toml").tempfile().unwrap();
        writeln!(
            file,
            r#"
[instances.inst1]
url = "http://inst1"
api_key = "key1"

[instances.inst2]
url = "http://inst2"
api_key = "key2"
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = match AppConfig::load(Some(path), vec![]).unwrap() {
            ConfigResult::Config(c) => c,
            ConfigResult::Exit => panic!("Expected Config, got Exit"),
        };
        assert_eq!(config.instances.len(), 2);
        // BTreeMap sorts by key, so inst1 should be first if visit_map uses it.
        // Actually, into_values() on BTreeMap will be in key order.
        assert!(config.instances.iter().any(|i| i.url == "http://inst1"));
        assert!(config.instances.iter().any(|i| i.url == "http://inst2"));
    }

    #[test]
    fn test_config_validation_errors() {
        let mut config = AppConfig {
            instances: vec![InstanceConfig {
                url: "".to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };
        assert!(config.validate().is_err());
        assert!(config.validate().unwrap_err().contains("missing URL"));

        config.instances = vec![];
        config.host = "".to_string();
        assert!(config.validate().is_err());
        assert!(
            config
                .validate()
                .unwrap_err()
                .contains("At least one SyncThing instance")
        );
    }

    #[test]
    fn test_get_instance() {
        let config = AppConfig {
            instances: vec![
                InstanceConfig {
                    name: Some("first".to_string()),
                    url: "http://first".to_string(),
                    ..Default::default()
                },
                InstanceConfig {
                    name: Some("second".to_string()),
                    url: "http://second".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // Get default (first)
        assert_eq!(
            config.get_instance(None).unwrap().name,
            Some("first".to_string())
        );

        // Get by index
        assert_eq!(
            config.get_instance(Some("1")).unwrap().name,
            Some("second".to_string())
        );

        // Get by name
        assert_eq!(
            config.get_instance(Some("second")).unwrap().name,
            Some("second".to_string())
        );

        // Not found
        assert!(config.get_instance(Some("third")).is_err());
    }
}

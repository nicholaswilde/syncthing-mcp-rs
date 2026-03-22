use clap::ArgMatches;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub api_key: Option<String>,
    #[serde(default = "default_transport")]
    pub mcp_transport: String,
    #[serde(default = "default_log_level")]
    pub log_level: String,
    #[serde(default = "default_no_verify_ssl")]
    pub no_verify_ssl: bool,
    #[serde(default, deserialize_with = "deserialize_instances")]
    pub instances: Vec<InstanceConfig>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Default)]
pub struct InstanceConfig {
    pub name: Option<String>,
    pub url: String,
    pub api_key: Option<String>,
    pub no_verify_ssl: Option<bool>,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8384,
            api_key: None,
            mcp_transport: "stdio".to_string(),
            log_level: "info".to_string(),
            no_verify_ssl: true,
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

impl AppConfig {
    pub fn load(file_path: Option<String>, cli_args: Vec<String>) -> Result<Self, ConfigError> {
        let mut builder = Config::builder();
        let matches = parse_args(cli_args);

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
            .set_default("port", 8384)?;

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
        Ok(config)
    }

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
                api_key: self.api_key.clone(),
                no_verify_ssl: Some(self.no_verify_ssl),
            });
        }

        if self.instances.is_empty() {
            return Err("At least one SyncThing instance must be configured".to_string());
        }

        for (i, inst) in self.instances.iter().enumerate() {
            if inst.url.is_empty() {
                return Err(format!("Instance {} is missing URL", i));
            }
        }

        Ok(())
    }

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

        let config = AppConfig::load(None, vec![]).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 8384);
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
        let config = AppConfig::load(None, args).unwrap();
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

        let config = AppConfig::load(None, vec![]).unwrap();

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
        writeln!(file, "host = \"file.com\"\nport = 6060").unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = AppConfig::load(Some(path), vec![]).unwrap();
        assert_eq!(config.host, "file.com");
        assert_eq!(config.port, 6060);
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

[[instances]]
name = "primary"
url = "http://192.168.1.1"
api_key = "key1"

[[instances]]
name = "secondary"
url = "http://192.168.1.2"
api_key = "key2"
no_verify_ssl = false
"#
        )
        .unwrap();
        let path = file.path().to_str().unwrap().to_string();

        let config = AppConfig::load(Some(path), vec![]).unwrap();
        assert_eq!(config.instances.len(), 2);
        assert_eq!(config.instances[0].name, Some("primary".to_string()));
        assert_eq!(config.instances[0].url, "http://192.168.1.1");
        assert_eq!(config.instances[1].name, Some("secondary".to_string()));
        assert_eq!(config.instances[1].no_verify_ssl, Some(false));
    }

    #[test]
    fn test_config_validation() {
        let mut config = AppConfig {
            instances: vec![],
            ..Default::default()
        };
        assert!(config.validate().is_ok());
        assert_eq!(config.instances.len(), 1);
        assert_eq!(config.instances[0].name, Some("default".to_string()));

        config.instances = vec![InstanceConfig {
            url: "".to_string(),
            ..Default::default()
        }];
        assert!(config.validate().is_err());
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

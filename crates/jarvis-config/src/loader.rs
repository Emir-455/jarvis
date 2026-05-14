use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use serde_yaml::Value;
use tracing::{debug, warn};

use jarvis_common::config::JarvisConfig;

/// Load and merge all modular YAML config files from the given directory.
///
/// Mirrors the Python `config/loader.py` behaviour:
/// 1. Load `config.yaml` (bootstrap file).
/// 2. Pop the `includes` list and load each referenced YAML in order.
/// 3. Deep-merge into a single tree.
/// 4. Resolve `${env:VAR}` references from the environment.
/// 5. Deserialize into `JarvisConfig`.
pub fn load_config_from_dir(config_dir: &Path) -> Result<JarvisConfig> {
    let base_path = config_dir.join("config.yaml");
    let mut merged = load_yaml_file(&base_path)?;

    // Pop includes list
    let includes: Vec<String> = if let Some(Value::Sequence(seq)) = merged.get("includes") {
        seq.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect()
    } else {
        Vec::new()
    };

    if let Value::Mapping(ref mut map) = merged {
        map.remove(Value::String("includes".into()));
    }

    for include_file in &includes {
        let include_path = config_dir.join(include_file);
        match load_yaml_file(&include_path) {
            Ok(include_data) => {
                merged = deep_merge(merged, include_data);
                debug!("Config loaded: {include_file}");
            }
            Err(e) => {
                warn!("Config file not found or invalid: {include_file}: {e}");
            }
        }
    }

    // Resolve environment variables
    merged = resolve_env_vars(merged);

    // Validate required env vars
    validate_required_env(&merged);

    let config: JarvisConfig =
        serde_yaml::from_value(merged).context("Failed to deserialize merged config")?;

    Ok(config)
}

/// Load config from the default config directory (./config).
pub fn load_config() -> Result<JarvisConfig> {
    let config_dir = find_config_dir()?;
    load_config_from_dir(&config_dir)
}

fn find_config_dir() -> Result<PathBuf> {
    // Try ./config first, then exe-relative
    let candidates = [
        PathBuf::from("config"),
        std::env::current_exe()
            .unwrap_or_default()
            .parent()
            .unwrap_or(Path::new("."))
            .join("config"),
    ];

    for candidate in &candidates {
        if candidate.join("config.yaml").exists() {
            return Ok(candidate.clone());
        }
    }

    // Fallback: return default and let caller handle missing files
    Ok(PathBuf::from("config"))
}

fn load_yaml_file(path: &Path) -> Result<Value> {
    if !path.exists() {
        warn!("Config file not found: {}", path.display());
        return Ok(Value::Mapping(Default::default()));
    }

    let content = std::fs::read_to_string(path)
        .with_context(|| format!("reading {}", path.display()))?;
    let data: Value =
        serde_yaml::from_str(&content).with_context(|| format!("parsing {}", path.display()))?;

    match data {
        Value::Mapping(_) => Ok(data),
        _ => Ok(Value::Mapping(Default::default())),
    }
}

fn deep_merge(base: Value, overrides: Value) -> Value {
    match (base, overrides) {
        (Value::Mapping(mut base_map), Value::Mapping(override_map)) => {
            for (key, override_val) in override_map {
                let merged_val = if let Some(base_val) = base_map.remove(&key) {
                    deep_merge(base_val, override_val)
                } else {
                    override_val
                };
                base_map.insert(key, merged_val);
            }
            Value::Mapping(base_map)
        }
        (_, override_val) => override_val,
    }
}

fn resolve_env_vars(value: Value) -> Value {
    let env_re = Regex::new(r"\$\{env:([^}]+)\}").expect("valid regex");

    match value {
        Value::String(s) => {
            let resolved = env_re
                .replace_all(&s, |caps: &regex::Captures| {
                    let var_name = &caps[1];
                    match std::env::var(var_name) {
                        Ok(val) => val,
                        Err(_) => {
                            warn!("Environment variable not found: {var_name}");
                            caps[0].to_string()
                        }
                    }
                })
                .to_string();
            Value::String(resolved)
        }
        Value::Mapping(map) => {
            let mut resolved = serde_yaml::Mapping::new();
            for (k, v) in map {
                resolved.insert(k, resolve_env_vars(v));
            }
            Value::Mapping(resolved)
        }
        Value::Sequence(seq) => {
            Value::Sequence(seq.into_iter().map(resolve_env_vars).collect())
        }
        other => other,
    }
}

fn validate_required_env(config: &Value) {
    let required = config
        .get("environment")
        .and_then(|e| e.get("required"))
        .and_then(|r| r.as_sequence());

    if let Some(vars) = required {
        let missing: Vec<&str> = vars
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|name| std::env::var(name).is_err())
            .collect();

        if !missing.is_empty() {
            warn!(
                "Missing required environment variables: {}. \
                 Define them or add to .env file.",
                missing.join(", ")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deep_merge_basic() {
        let base: Value = serde_yaml::from_str("a: 1\nb:\n  c: 2").unwrap();
        let over: Value = serde_yaml::from_str("b:\n  d: 3\ne: 4").unwrap();
        let merged = deep_merge(base, over);
        assert_eq!(merged.get("a").unwrap().as_u64(), Some(1));
        assert_eq!(
            merged.get("b").unwrap().get("c").unwrap().as_u64(),
            Some(2)
        );
        assert_eq!(
            merged.get("b").unwrap().get("d").unwrap().as_u64(),
            Some(3)
        );
        assert_eq!(merged.get("e").unwrap().as_u64(), Some(4));
    }

    #[test]
    fn env_var_resolution() {
        std::env::set_var("TEST_JARVIS_VAR", "hello");
        let val = Value::String("${env:TEST_JARVIS_VAR}_world".into());
        let resolved = resolve_env_vars(val);
        assert_eq!(resolved.as_str().unwrap(), "hello_world");
        std::env::remove_var("TEST_JARVIS_VAR");
    }
}

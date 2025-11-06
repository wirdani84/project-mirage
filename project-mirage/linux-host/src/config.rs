use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub host: HostConfig,
    
    #[serde(default)]
    pub network: NetworkConfig,
    
    #[serde(default)]
    pub streaming: StreamingConfig,
    
    #[serde(default)]
    pub security: SecurityConfig,
    
    #[serde(default)]
    pub input: InputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostConfig {
    pub name: Option<String>,
    #[serde(default = "default_edge_threshold")]
    pub display_edge_threshold: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default = "default_discovery_port")]
    pub discovery_port: u16,
    
    #[serde(default = "default_control_port")]
    pub control_port: u16,
    
    #[serde(default)]
    pub allowed_subnets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConfig {
    #[serde(default = "default_max_fps")]
    pub max_fps: u32,
    
    #[serde(default = "default_codec")]
    pub codec: String,
    
    #[serde(default = "default_bitrate")]
    pub bitrate_mbps: u32,
    
    #[serde(default = "default_true")]
    pub hardware_encode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_true")]
    pub require_pairing: bool,
    
    #[serde(default = "default_session_timeout")]
    pub session_timeout_minutes: u64,
    
    #[serde(default)]
    pub cert_path: Option<String>,
    
    #[serde(default)]
    pub key_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputConfig {
    #[serde(default = "default_mouse_acceleration")]
    pub mouse_acceleration: f32,
    
    #[serde(default = "default_true")]
    pub enable_smooth_scroll: bool,
    
    #[serde(default = "default_edge_activation_delay")]
    pub edge_activation_delay_ms: u32,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            name: None,
            display_edge_threshold: default_edge_threshold(),
        }
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            discovery_port: default_discovery_port(),
            control_port: default_control_port(),
            allowed_subnets: vec!["192.168.0.0/16".to_string(), "10.0.0.0/8".to_string()],
        }
    }
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            max_fps: default_max_fps(),
            codec: default_codec(),
            bitrate_mbps: default_bitrate(),
            hardware_encode: true,
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            require_pairing: true,
            session_timeout_minutes: default_session_timeout(),
            cert_path: None,
            key_path: None,
        }
    }
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_acceleration: default_mouse_acceleration(),
            enable_smooth_scroll: true,
            edge_activation_delay_ms: default_edge_activation_delay(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: HostConfig::default(),
            network: NetworkConfig::default(),
            streaming: StreamingConfig::default(),
            security: SecurityConfig::default(),
            input: InputConfig::default(),
        }
    }
}

impl Config {
    pub async fn load(path: &str) -> Result<Self> {
        let expanded_path = shellexpand::tilde(path);
        let path = Path::new(expanded_path.as_ref());

        if path.exists() {
            let contents = fs::read_to_string(path)
                .await
                .context("Failed to read config file")?;
            
            toml::from_str(&contents).context("Failed to parse config file")
        } else {
            // Create default config
            let config = Config::default();
            
            // Try to create parent directory
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent).await;
            }
            
            // Try to write default config
            let toml_string = toml::to_string_pretty(&config)?;
            let _ = fs::write(path, toml_string).await;
            
            Ok(config)
        }
    }
}

// Default value functions
fn default_edge_threshold() -> u32 { 10 }
fn default_discovery_port() -> u16 { 5353 }
fn default_control_port() -> u16 { 8443 }
fn default_max_fps() -> u32 { 60 }
fn default_codec() -> String { "h264".to_string() }
fn default_bitrate() -> u32 { 10 }
fn default_session_timeout() -> u64 { 60 }
fn default_mouse_acceleration() -> f32 { 1.0 }
fn default_edge_activation_delay() -> u32 { 100 }
fn default_true() -> bool { true }

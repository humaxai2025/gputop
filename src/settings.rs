use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    pub temperature_warning: f32,
    pub temperature_critical: f32,
    pub power_warning: f32,
    pub power_critical: f32,
    pub memory_usage_warning: f32,
    pub memory_usage_critical: f32,
    pub utilization_low: f32,
    pub utilization_high: f32,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            temperature_warning: 75.0,
            temperature_critical: 85.0,
            power_warning: 80.0,  // Percentage of TDP
            power_critical: 95.0,
            memory_usage_warning: 85.0,
            memory_usage_critical: 95.0,
            utilization_low: 10.0,   // For idle warnings
            utilization_high: 95.0,  // For overwork warnings
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub min_interval_seconds: u64,
    pub show_export_notifications: bool,
    pub show_process_notifications: bool,
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            min_interval_seconds: 10,
            show_export_notifications: true,
            show_process_notifications: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub health_thresholds: HealthThresholds,
    pub notification_settings: NotificationSettings,
    pub update_interval_ms: u64,
    pub max_history_points: usize,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            health_thresholds: HealthThresholds::default(),
            notification_settings: NotificationSettings::default(),
            update_interval_ms: 1000,
            max_history_points: 300,
        }
    }
}

pub struct SettingsManager {
    settings: AppSettings,
    config_path: std::path::PathBuf,
}

impl SettingsManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not find config directory"))?
            .join("gputop");
        
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        
        let config_path = config_dir.join("settings.json");
        let settings = if config_path.exists() {
            Self::load_settings(&config_path)?
        } else {
            AppSettings::default()
        };
        
        Ok(Self {
            settings,
            config_path,
        })
    }
    
    pub fn get_settings(&self) -> &AppSettings {
        &self.settings
    }
    
    pub fn get_settings_mut(&mut self) -> &mut AppSettings {
        &mut self.settings
    }
    
    pub fn save_settings(&self) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.settings)?;
        fs::write(&self.config_path, json)?;
        Ok(())
    }
    
    fn load_settings(path: &Path) -> Result<AppSettings> {
        let content = fs::read_to_string(path)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }
    
    pub fn reset_to_defaults(&mut self) -> Result<()> {
        self.settings = AppSettings::default();
        self.save_settings()
    }
}
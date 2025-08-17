use std::process::Command;
use crate::health::{HealthAlert, HealthStatus};

pub struct NotificationManager;

impl NotificationManager {
    pub fn new() -> Self {
        Self
    }
    
    pub fn send_health_alert(&self, alert: &HealthAlert) {
        // Only send notifications for Warning and Critical alerts to avoid spam
        match alert.severity {
            HealthStatus::Warning | HealthStatus::Critical => {
                self.send_toast_notification(
                    &format!("GPUTop Health Alert - {}", alert.severity.text()),
                    &alert.message,
                    &self.get_alert_icon(&alert.severity),
                );
            }
            _ => {} // Don't send notifications for Good/Excellent status
        }
    }
    
    pub fn send_export_success(&self, filename: &str) {
        self.send_toast_notification(
            "GPUTop Export Complete",
            &format!("Data exported to: {}", filename),
            "✅",
        );
    }
    
    pub fn send_export_error(&self, error: &str) {
        self.send_toast_notification(
            "GPUTop Export Failed",
            &format!("Export error: {}", error),
            "❌",
        );
    }
    
    pub fn send_process_killed(&self, process_name: &str, pid: u32) {
        self.send_toast_notification(
            "GPUTop Process Terminated",
            &format!("Killed process: {} (PID: {})", process_name, pid),
            "🛑",
        );
    }
    
    fn send_toast_notification(&self, title: &str, message: &str, icon: &str) {
        // For Windows, use PowerShell to send toast notifications
        #[cfg(target_os = "windows")]
        {
            let powershell_script = format!(
                r#"
                [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null;
                [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null;
                
                $template = @"
                <toast>
                    <visual>
                        <binding template="ToastGeneric">
                            <text>{}</text>
                            <text>{}</text>
                        </binding>
                    </visual>
                </toast>
                "@;
                
                $xml = New-Object Windows.Data.Xml.Dom.XmlDocument;
                $xml.LoadXml($template);
                $toast = New-Object Windows.UI.Notifications.ToastNotification $xml;
                [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("GPUTop").Show($toast);
                "#,
                title, message
            );
            
            // Execute PowerShell command silently
            let _ = Command::new("powershell")
                .args(&["-WindowStyle", "Hidden", "-Command", &powershell_script])
                .output();
        }
        
        // For non-Windows systems, we could implement libnotify or similar
        #[cfg(not(target_os = "windows"))]
        {
            // On Linux/Mac, we could use notify-send or osascript
            // For now, just log to console
            println!("📢 {} - {}", title, message);
        }
    }
    
    fn get_alert_icon(&self, severity: &HealthStatus) -> &'static str {
        match severity {
            HealthStatus::Excellent => "🟢",
            HealthStatus::Good => "🔵", 
            HealthStatus::Warning => "🟡",
            HealthStatus::Critical => "🔴",
        }
    }
    
    pub fn test_notification(&self) {
        self.send_toast_notification(
            "GPUTop Test Notification",
            "Notifications are working correctly!",
            "🧪",
        );
    }
}

// Simple notification queue to avoid spam
pub struct NotificationQueue {
    last_notification_time: std::time::Instant,
    min_interval: std::time::Duration,
}

impl NotificationQueue {
    pub fn new() -> Self {
        Self {
            last_notification_time: std::time::Instant::now(),
            min_interval: std::time::Duration::from_secs(10), // Minimum 10 seconds between notifications
        }
    }
    
    pub fn should_send_notification(&mut self) -> bool {
        let now = std::time::Instant::now();
        if now.duration_since(self.last_notification_time) >= self.min_interval {
            self.last_notification_time = now;
            true
        } else {
            false
        }
    }
}
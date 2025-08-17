use chrono::{DateTime, Local, TimeZone};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Excellent,  // 🟢 All metrics optimal
    Good,       // 🔵 Minor concerns
    Warning,    // 🟡 Attention needed
    Critical,   // 🔴 Immediate action required
}

#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    TemperatureHigh,
    TemperatureCritical,
    ThermalThrottling,
    PowerSpike,
    MemoryLeakSuspected,
    ClockInstability,
    FanIssue,
}

#[derive(Debug, Clone)]
pub struct HealthAlert {
    pub alert_type: AlertType,
    pub message: String,
    pub severity: HealthStatus,
    pub timestamp: DateTime<Local>,
    pub value: Option<f32>,
    pub threshold: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct TemperatureMetrics {
    pub current: f32,
    pub max_safe: f32,
    pub critical: f32,
    pub trend_5min: f32,  // degrees change over 5 minutes
    pub time_above_80c: u64,  // seconds spent above 80°C
    pub peak_today: f32,
}

#[derive(Debug, Clone)]
pub struct PowerMetrics {
    pub current_draw: f32,
    pub efficiency: f32,  // performance per watt
    pub power_spikes: u32,  // number of sudden power increases
    pub avg_draw_1hr: f32,
}

#[derive(Debug, Clone)]
pub struct MemoryHealthMetrics {
    pub usage_trend: f32,  // MB change per minute
    pub fragmentation_score: f32,  // 0.0-1.0, higher = more fragmented
    pub leak_suspicion: f32,  // 0.0-1.0 based on usage patterns
    pub peak_usage_today: u64,
}

#[derive(Debug, Clone)]
pub struct GpuHealthMetrics {
    pub overall_score: f32,  // 0.0-100.0
    pub status: HealthStatus,
    pub temperature: TemperatureMetrics,
    pub power: PowerMetrics,
    pub memory: MemoryHealthMetrics,
    pub thermal_throttling_detected: bool,
    pub uptime_hours: f32,
    pub alerts: Vec<HealthAlert>,
}

pub struct HealthMonitor {
    history_window: VecDeque<HealthSnapshot>,
    alert_history: VecDeque<HealthAlert>,
    monitoring_start: DateTime<Local>,
}

#[derive(Debug, Clone)]
struct HealthSnapshot {
    timestamp: DateTime<Local>,
    temperature: f32,
    power_draw: f32,
    memory_used: u64,
    gpu_utilization: f32,
    clock_speeds: (u32, u32), // gpu_clock, memory_clock
    is_throttling: bool,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            history_window: VecDeque::with_capacity(3600), // 1 hour at 1Hz
            alert_history: VecDeque::with_capacity(100),
            monitoring_start: Local::now(),
        }
    }

    pub fn update_metrics(&mut self, 
        temperature: f32,
        power_draw: Option<f32>,
        memory_used: u64,
        memory_total: u64,
        gpu_utilization: f32,
        gpu_clock: Option<u32>,
        memory_clock: Option<u32>,
        is_throttling: bool,
    ) -> GpuHealthMetrics {
        
        let snapshot = HealthSnapshot {
            timestamp: Local::now(),
            temperature,
            power_draw: power_draw.unwrap_or(0.0),
            memory_used,
            gpu_utilization,
            clock_speeds: (gpu_clock.unwrap_or(0), memory_clock.unwrap_or(0)),
            is_throttling,
        };

        self.history_window.push_back(snapshot.clone());
        
        // Keep only last hour of data
        while self.history_window.len() > 3600 {
            self.history_window.pop_front();
        }

        // Calculate health metrics
        let temperature_metrics = self.calculate_temperature_metrics(&snapshot);
        let power_metrics = self.calculate_power_metrics(&snapshot);
        let memory_metrics = self.calculate_memory_metrics(&snapshot, memory_total);
        
        // Generate alerts
        let mut alerts = Vec::new();
        self.check_temperature_alerts(&temperature_metrics, &mut alerts);
        self.check_power_alerts(&power_metrics, &mut alerts);
        self.check_memory_alerts(&memory_metrics, &mut alerts);
        
        if is_throttling {
            alerts.push(HealthAlert {
                alert_type: AlertType::ThermalThrottling,
                message: "GPU is thermal throttling - performance reduced".to_string(),
                severity: HealthStatus::Warning,
                timestamp: Local::now(),
                value: Some(temperature),
                threshold: Some(83.0),
            });
        }

        // Add new alerts to history
        for alert in &alerts {
            self.alert_history.push_back(alert.clone());
        }
        
        // Keep only last 100 alerts
        while self.alert_history.len() > 100 {
            self.alert_history.pop_front();
        }

        let overall_score = self.calculate_overall_health_score(
            &temperature_metrics, &power_metrics, &memory_metrics, is_throttling
        );
        
        let status = self.determine_health_status(overall_score, &alerts);
        
        let uptime = (Local::now() - self.monitoring_start).num_minutes() as f32 / 60.0;

        GpuHealthMetrics {
            overall_score,
            status,
            temperature: temperature_metrics,
            power: power_metrics,
            memory: memory_metrics,
            thermal_throttling_detected: is_throttling,
            uptime_hours: uptime,
            alerts,
        }
    }

    fn calculate_temperature_metrics(&self, current: &HealthSnapshot) -> TemperatureMetrics {
        let temp = current.temperature;
        
        // Calculate 5-minute trend
        let five_min_ago = current.timestamp - chrono::Duration::minutes(5);
        let trend_5min = self.history_window.iter()
            .find(|s| s.timestamp >= five_min_ago)
            .map(|s| temp - s.temperature)
            .unwrap_or(0.0);

        // Count time above 80°C in last hour
        let time_above_80c = self.history_window.iter()
            .filter(|s| s.temperature > 80.0)
            .count() as u64; // seconds

        // Find peak temperature today
        let today_start = current.timestamp.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_start = Local.from_local_datetime(&today_start).unwrap();
        
        let peak_today = self.history_window.iter()
            .filter(|s| s.timestamp >= today_start)
            .map(|s| s.temperature)
            .fold(temp, f32::max);

        TemperatureMetrics {
            current: temp,
            max_safe: 80.0,
            critical: 90.0,
            trend_5min,
            time_above_80c,
            peak_today,
        }
    }

    fn calculate_power_metrics(&self, current: &HealthSnapshot) -> PowerMetrics {
        let power = current.power_draw;
        let utilization = current.gpu_utilization;
        
        // Calculate efficiency (utilization per watt)
        let efficiency = if power > 0.0 { utilization / power } else { 0.0 };
        
        // Count power spikes (>20W increase in <10 seconds)
        let power_spikes = self.history_window.iter()
            .rev()
            .take(10)
            .zip(self.history_window.iter().rev().skip(1).take(10))
            .filter(|(curr, prev)| curr.power_draw - prev.power_draw > 20.0)
            .count() as u32;

        // Calculate 1-hour average
        let avg_draw_1hr = if self.history_window.is_empty() {
            power
        } else {
            self.history_window.iter()
                .map(|s| s.power_draw)
                .sum::<f32>() / self.history_window.len() as f32
        };

        PowerMetrics {
            current_draw: power,
            efficiency,
            power_spikes,
            avg_draw_1hr,
        }
    }

    fn calculate_memory_metrics(&self, current: &HealthSnapshot, total_memory: u64) -> MemoryHealthMetrics {
        let current_usage = current.memory_used;
        
        // Calculate usage trend (MB change per minute)
        let one_min_ago = current.timestamp - chrono::Duration::minutes(1);
        let usage_trend = self.history_window.iter()
            .find(|s| s.timestamp >= one_min_ago)
            .map(|s| (current_usage as f32 - s.memory_used as f32) / (1024.0 * 1024.0))
            .unwrap_or(0.0);

        // Simple fragmentation estimation based on usage patterns
        let usage_variance = if self.history_window.len() > 10 {
            let recent_usage: Vec<f32> = self.history_window.iter()
                .rev().take(60)
                .map(|s| s.memory_used as f32)
                .collect();
            
            let mean = recent_usage.iter().sum::<f32>() / recent_usage.len() as f32;
            let variance = recent_usage.iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f32>() / recent_usage.len() as f32;
            
            (variance.sqrt() / mean).min(1.0)
        } else {
            0.0
        };

        // Memory leak suspicion based on steady increase
        let leak_suspicion = if usage_trend > 10.0 { // >10MB/min increase
            (usage_trend / 50.0).min(1.0)
        } else {
            0.0
        };

        // Peak usage today
        let today_start = current.timestamp.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let today_start = Local.from_local_datetime(&today_start).unwrap();
        
        let peak_usage_today = self.history_window.iter()
            .filter(|s| s.timestamp >= today_start)
            .map(|s| s.memory_used)
            .max()
            .unwrap_or(current_usage);

        MemoryHealthMetrics {
            usage_trend,
            fragmentation_score: usage_variance,
            leak_suspicion,
            peak_usage_today,
        }
    }

    fn calculate_overall_health_score(&self, 
        temp: &TemperatureMetrics, 
        power: &PowerMetrics, 
        memory: &MemoryHealthMetrics,
        is_throttling: bool
    ) -> f32 {
        let mut score: f32 = 100.0;

        // Temperature penalties
        if temp.current > 90.0 { score -= 30.0; }
        else if temp.current > 85.0 { score -= 20.0; }
        else if temp.current > 80.0 { score -= 10.0; }
        
        if temp.trend_5min > 10.0 { score -= 15.0; } // Rapidly heating
        if temp.time_above_80c > 1800 { score -= 10.0; } // >30min above 80°C

        // Throttling penalty
        if is_throttling { score -= 25.0; }

        // Memory health penalties
        if memory.leak_suspicion > 0.7 { score -= 20.0; }
        else if memory.leak_suspicion > 0.3 { score -= 10.0; }

        if memory.fragmentation_score > 0.7 { score -= 15.0; }

        // Power efficiency penalties
        if power.efficiency < 0.5 { score -= 10.0; } // Low efficiency
        if power.power_spikes > 5 { score -= 5.0; } // Unstable power

        score.max(0.0).min(100.0)
    }

    fn determine_health_status(&self, score: f32, alerts: &[HealthAlert]) -> HealthStatus {
        let has_critical = alerts.iter().any(|a| a.severity == HealthStatus::Critical);
        let has_warning = alerts.iter().any(|a| a.severity == HealthStatus::Warning);

        if has_critical || score < 30.0 {
            HealthStatus::Critical
        } else if has_warning || score < 60.0 {
            HealthStatus::Warning
        } else if score < 85.0 {
            HealthStatus::Good
        } else {
            HealthStatus::Excellent
        }
    }

    fn check_temperature_alerts(&self, temp: &TemperatureMetrics, alerts: &mut Vec<HealthAlert>) {
        if temp.current >= temp.critical {
            alerts.push(HealthAlert {
                alert_type: AlertType::TemperatureCritical,
                message: format!("CRITICAL: GPU temperature {}°C exceeds safe limits!", temp.current),
                severity: HealthStatus::Critical,
                timestamp: Local::now(),
                value: Some(temp.current),
                threshold: Some(temp.critical),
            });
        } else if temp.current >= temp.max_safe {
            alerts.push(HealthAlert {
                alert_type: AlertType::TemperatureHigh,
                message: format!("WARNING: GPU temperature {}°C is high", temp.current),
                severity: HealthStatus::Warning,
                timestamp: Local::now(),
                value: Some(temp.current),
                threshold: Some(temp.max_safe),
            });
        }

        if temp.trend_5min > 15.0 {
            alerts.push(HealthAlert {
                alert_type: AlertType::TemperatureHigh,
                message: format!("Temperature rising rapidly (+{:.1}°C in 5min)", temp.trend_5min),
                severity: HealthStatus::Warning,
                timestamp: Local::now(),
                value: Some(temp.trend_5min),
                threshold: Some(10.0),
            });
        }
    }

    fn check_power_alerts(&self, power: &PowerMetrics, alerts: &mut Vec<HealthAlert>) {
        if power.power_spikes > 10 {
            alerts.push(HealthAlert {
                alert_type: AlertType::PowerSpike,
                message: format!("Detected {} power spikes - check power supply stability", power.power_spikes),
                severity: HealthStatus::Warning,
                timestamp: Local::now(),
                value: Some(power.power_spikes as f32),
                threshold: Some(5.0),
            });
        }
    }

    fn check_memory_alerts(&self, memory: &MemoryHealthMetrics, alerts: &mut Vec<HealthAlert>) {
        if memory.leak_suspicion > 0.8 {
            alerts.push(HealthAlert {
                alert_type: AlertType::MemoryLeakSuspected,
                message: "Possible memory leak detected - memory usage increasing steadily".to_string(),
                severity: HealthStatus::Warning,
                timestamp: Local::now(),
                value: Some(memory.leak_suspicion),
                threshold: Some(0.5),
            });
        }
    }

    pub fn get_recent_alerts(&self, limit: usize) -> Vec<HealthAlert> {
        self.alert_history.iter()
            .rev()
            .take(limit)
            .cloned()
            .collect()
    }
}

impl HealthStatus {
    pub fn emoji(&self) -> &'static str {
        match self {
            HealthStatus::Excellent => "🟢",
            HealthStatus::Good => "🔵",
            HealthStatus::Warning => "🟡",
            HealthStatus::Critical => "🔴",
        }
    }

    pub fn text(&self) -> &'static str {
        match self {
            HealthStatus::Excellent => "Excellent",
            HealthStatus::Good => "Good",
            HealthStatus::Warning => "Warning",
            HealthStatus::Critical => "Critical",
        }
    }
}

impl AlertType {
    pub fn emoji(&self) -> &'static str {
        match self {
            AlertType::TemperatureHigh => "🌡️",
            AlertType::TemperatureCritical => "🔥",
            AlertType::ThermalThrottling => "🐌",
            AlertType::PowerSpike => "⚡",
            AlertType::MemoryLeakSuspected => "🧠",
            AlertType::ClockInstability => "⏰",
            AlertType::FanIssue => "🌀",
        }
    }
}
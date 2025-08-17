use chrono::{DateTime, Local};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use anyhow::Result;
use crate::gpu::GpuInfo;
use crate::process::GpuProcess;
use crate::health::GpuHealthMetrics;

pub struct CsvExporter;

impl CsvExporter {
    pub fn export_current_snapshot(
        gpu: &GpuInfo,
        processes: &[GpuProcess],
        health: Option<&GpuHealthMetrics>,
        output_path: &str,
    ) -> Result<()> {
        let mut file = File::create(output_path)?;
        let timestamp = Local::now();
        
        // Write header
        writeln!(file, "GPUTop Export - {}", timestamp.format("%Y-%m-%d %H:%M:%S"))?;
        writeln!(file, "")?;
        
        // GPU Information
        Self::write_gpu_info(&mut file, gpu)?;
        
        // Health Metrics
        if let Some(health) = health {
            Self::write_health_metrics(&mut file, health)?;
        }
        
        // Process Information
        Self::write_process_info(&mut file, processes)?;
        
        Ok(())
    }
    
    pub fn export_processes_csv(processes: &[GpuProcess], output_path: &str) -> Result<()> {
        let mut file = File::create(output_path)?;
        
        // CSV Header
        writeln!(file, "timestamp,pid,user,command,gpu_usage_percent,memory_usage_mb,memory_usage_gb,encoder_usage_percent,decoder_usage_percent,priority,context_id,container_id,parent_pid")?;
        
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        
        // Process data
        for process in processes {
            let memory_mb = process.memory_usage / (1024 * 1024);
            let memory_gb = memory_mb as f64 / 1024.0;
            
            writeln!(
                file,
                "{},{},{},{},{:.1},{},{:.2},{:.1},{:.1},{},{},{},{}",
                timestamp,
                process.pid,
                Self::escape_csv(&process.user),
                Self::escape_csv(&process.command),
                process.gpu_usage,
                memory_mb,
                memory_gb,
                process.encoder_usage,
                process.decoder_usage,
                process.priority,
                process.context_id.map_or("".to_string(), |id| id.to_string()),
                process.container_id.as_deref().unwrap_or(""),
                process.parent_pid.map_or("".to_string(), |pid| pid.to_string())
            )?;
        }
        
        Ok(())
    }
    
    pub fn export_gpu_metrics_csv(gpu: &GpuInfo, health: Option<&GpuHealthMetrics>, output_path: &str) -> Result<()> {
        let mut file = File::create(output_path)?;
        
        // CSV Header
        writeln!(file, "timestamp,gpu_name,utilization_percent,memory_used_mb,memory_total_mb,memory_usage_percent,temperature_c,power_draw_w,gpu_clock_mhz,memory_clock_mhz,fan_speed_percent,throttled,health_score,health_status")?;
        
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        let memory_used_mb = gpu.memory_used / (1024 * 1024);
        let memory_total_mb = gpu.memory_total / (1024 * 1024);
        let memory_usage_percent = (gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0;
        
        let (health_score, health_status) = if let Some(h) = health {
            (h.overall_score.to_string(), h.status.text().to_string())
        } else {
            ("N/A".to_string(), "Unknown".to_string())
        };
        
        writeln!(
            file,
            "{},{},{:.1},{},{},{:.1},{:.1},{},{},{},{},{},{},{}",
            timestamp,
            Self::escape_csv(&gpu.name),
            gpu.utilization,
            memory_used_mb,
            memory_total_mb,
            memory_usage_percent,
            gpu.temperature,
            gpu.power_draw.map_or("N/A".to_string(), |p| format!("{:.1}", p)),
            gpu.gpu_clock.map_or("N/A".to_string(), |c| c.to_string()),
            gpu.memory_clock.map_or("N/A".to_string(), |c| c.to_string()),
            gpu.fan_speed.map_or("N/A".to_string(), |f| format!("{:.0}", f)),
            if gpu.throttled { "Yes" } else { "No" },
            health_score,
            health_status
        )?;
        
        Ok(())
    }
    
    pub fn export_health_alerts_csv(health: &GpuHealthMetrics, output_path: &str) -> Result<()> {
        let mut file = File::create(output_path)?;
        
        // CSV Header
        writeln!(file, "timestamp,alert_type,severity,message")?;
        
        // Alert data
        for alert in &health.alerts {
            writeln!(
                file,
                "{},{},{},{}",
                alert.timestamp.format("%Y-%m-%d %H:%M:%S"),
                format!("{:?}", alert.alert_type),
                alert.severity.text(),
                Self::escape_csv(&alert.message)
            )?;
        }
        
        Ok(())
    }
    
    fn write_gpu_info(file: &mut File, gpu: &GpuInfo) -> Result<()> {
        writeln!(file, "=== GPU INFORMATION ===")?;
        writeln!(file, "Name,{}", gpu.name)?;
        writeln!(file, "Driver Version,{}", gpu.driver_version)?;
        writeln!(file, "CUDA Version,{}", gpu.cuda_version.as_deref().unwrap_or("N/A"))?;
        writeln!(file, "Utilization,%{:.1}", gpu.utilization)?;
        writeln!(file, "Memory Used,{} MB", gpu.memory_used / (1024 * 1024))?;
        writeln!(file, "Memory Total,{} MB", gpu.memory_total / (1024 * 1024))?;
        writeln!(file, "Memory Usage,{:.1}%", (gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0)?;
        writeln!(file, "Temperature,{:.1}°C", gpu.temperature)?;
        
        if let Some(power) = gpu.power_draw {
            writeln!(file, "Power Draw,{:.1}W", power)?;
        }
        
        if let Some(clock) = gpu.gpu_clock {
            writeln!(file, "GPU Clock,{} MHz", clock)?;
        }
        
        if let Some(mem_clock) = gpu.memory_clock {
            writeln!(file, "Memory Clock,{} MHz", mem_clock)?;
        }
        
        if let Some(fan) = gpu.fan_speed {
            writeln!(file, "Fan Speed,{:.0}%", fan)?;
        }
        
        writeln!(file, "Throttled,{}", if gpu.throttled { "Yes" } else { "No" })?;
        writeln!(file, "")?;
        
        Ok(())
    }
    
    fn write_health_metrics(file: &mut File, health: &GpuHealthMetrics) -> Result<()> {
        writeln!(file, "=== HEALTH METRICS ===")?;
        writeln!(file, "Overall Health Score,{:.1}/100", health.overall_score)?;
        writeln!(file, "Health Status,{}", health.status.text())?;
        writeln!(file, "Uptime,{:.1} hours", health.uptime_hours)?;
        writeln!(file, "Thermal Throttling,{}", if health.thermal_throttling_detected { "Yes" } else { "No" })?;
        writeln!(file, "")?;
        
        writeln!(file, "=== TEMPERATURE HEALTH ===")?;
        writeln!(file, "Current Temperature,{:.1}°C", health.temperature.current)?;
        writeln!(file, "Temperature Trend (5min),{:+.1}°C", health.temperature.trend_5min)?;
        writeln!(file, "Time Above 80°C,{} minutes", health.temperature.time_above_80c / 60)?;
        writeln!(file, "Peak Temperature Today,{:.1}°C", health.temperature.peak_today)?;
        writeln!(file, "")?;
        
        writeln!(file, "=== POWER HEALTH ===")?;
        writeln!(file, "Current Power Draw,{:.1}W", health.power.current_draw)?;
        writeln!(file, "Power Efficiency,{:.2} util/W", health.power.efficiency)?;
        writeln!(file, "Power Spikes Count,{}", health.power.power_spikes)?;
        writeln!(file, "Average Power (1hr),{:.1}W", health.power.avg_draw_1hr)?;
        writeln!(file, "")?;
        
        writeln!(file, "=== MEMORY HEALTH ===")?;
        writeln!(file, "Memory Usage Trend,{:+.1} MB/min", health.memory.usage_trend)?;
        writeln!(file, "Memory Fragmentation,{:.1}%", health.memory.fragmentation_score * 100.0)?;
        writeln!(file, "Memory Leak Risk,{:.1}%", health.memory.leak_suspicion * 100.0)?;
        writeln!(file, "Peak Memory Usage Today,{:.1} GB", health.memory.peak_usage_today as f64 / (1024.0 * 1024.0 * 1024.0))?;
        writeln!(file, "")?;
        
        Ok(())
    }
    
    fn write_process_info(file: &mut File, processes: &[GpuProcess]) -> Result<()> {
        writeln!(file, "=== PROCESS INFORMATION ===")?;
        writeln!(file, "PID,User,Command,GPU%,Memory(MB),ENC%,DEC%,Priority")?;
        
        for process in processes {
            let memory_mb = process.memory_usage / (1024 * 1024);
            writeln!(
                file,
                "{},{},{},{:.1},{},{:.1},{:.1},{}",
                process.pid,
                Self::escape_csv(&process.user),
                Self::escape_csv(&process.command),
                process.gpu_usage,
                memory_mb,
                process.encoder_usage,
                process.decoder_usage,
                process.priority
            )?;
        }
        
        Ok(())
    }
    
    fn escape_csv(value: &str) -> String {
        if value.contains(',') || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace("\"", "\"\""))
        } else {
            value.to_string()
        }
    }
    
    pub fn get_export_filename(prefix: &str, extension: &str) -> String {
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        format!("{}_{}.{}", prefix, timestamp, extension)
    }
}
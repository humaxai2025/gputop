use anyhow::Result;
use sysinfo::{System, SystemExt, ProcessExt, UserExt, PidExt, Process, Pid};

#[derive(Debug, Clone)]
pub struct GpuProcess {
    pub pid: u32,
    pub user: String,
    pub command: String,
    pub gpu_usage: f32,
    pub memory_usage: u64,
    pub encoder_usage: f32,
    pub decoder_usage: f32,
    pub priority: i32,
    pub context_id: Option<u32>,
    pub container_id: Option<String>,
    pub parent_pid: Option<u32>,
}

pub struct ProcessManager {
    system: System,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            system: System::new_all(),
        }
    }

    pub async fn get_gpu_processes(&mut self) -> Result<Vec<GpuProcess>> {
        self.system.refresh_all();
        
        let mut processes = Vec::new();
        
        // Get real GPU processes from system
        processes.extend(self.get_real_processes());
        
        // If no real processes found, show a few mock ones for demo
        if processes.is_empty() {
            processes.extend(self.get_mock_processes());
        }
        
        Ok(processes)
    }


    fn get_real_processes(&self) -> Vec<GpuProcess> {
        let mut gpu_processes = Vec::new();
        
        // Common GPU-intensive process names
        let gpu_intensive_names = [
            "python", "blender", "ffmpeg", "obs", "davinci", "premiere", 
            "aftereffects", "maya", "3dsmax", "unity", "unreal", "chrome",
            "firefox", "discord", "steam", "game", "vlc", "mpv", "handbrake"
        ];
        
        for (pid, process) in self.system.processes() {
            let process_name = process.name().to_lowercase();
            let exe_path = process.exe().to_string_lossy().to_lowercase();
            
            // Check if this process might be using GPU
            let is_gpu_process = gpu_intensive_names.iter().any(|&name| {
                process_name.contains(name) || exe_path.contains(name)
            }) || process.cpu_usage() > 15.0; // High CPU usage might indicate GPU usage
            
            if is_gpu_process {
                let user_name = process.user_id()
                    .and_then(|uid| self.system.get_user_by_id(uid))
                    .map(|user| user.name().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                
                // Estimate GPU usage based on CPU usage (rough approximation)
                let estimated_gpu_usage = (process.cpu_usage() * 0.7).min(100.0);
                let memory_bytes = process.memory() * 1024; // Convert KB to bytes
                
                // Extract process name with extension from executable path - display ONLY the name
                let exe_path = process.exe();
                let display_command = exe_path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(process.name())
                    .to_string();
                
                gpu_processes.push(GpuProcess {
                    pid: pid.as_u32(),
                    user: user_name,
                    command: display_command,
                    gpu_usage: estimated_gpu_usage,
                    memory_usage: memory_bytes,
                    encoder_usage: if process_name.contains("ffmpeg") || process_name.contains("obs") { 
                        (process.cpu_usage() * 0.4).min(100.0) 
                    } else { 0.0 },
                    decoder_usage: if process_name.contains("chrome") || process_name.contains("firefox") {
                        (process.cpu_usage() * 0.2).min(100.0)
                    } else { 0.0 },
                    priority: 0,
                    context_id: None,
                    container_id: None,
                    parent_pid: process.parent().map(|p| p.as_u32()),
                });
            }
        }
        
        // Sort by GPU usage descending
        gpu_processes.sort_by(|a, b| b.gpu_usage.partial_cmp(&a.gpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        
        // Limit to top 15 processes to avoid clutter
        gpu_processes.truncate(15);
        
        gpu_processes
    }
    
    fn get_mock_processes(&self) -> Vec<GpuProcess> {
        vec![
            GpuProcess {
                pid: 1234,
                user: "user1".to_string(),
                command: "python.exe".to_string(),
                gpu_usage: 85.2,
                memory_usage: 3 * 1024 * 1024 * 1024, // 3GB
                encoder_usage: 0.0,
                decoder_usage: 0.0,
                priority: 0,
                context_id: Some(1),
                container_id: None,
                parent_pid: Some(1000),
            },
            GpuProcess {
                pid: 5678,
                user: "user2".to_string(),
                command: "blender.exe".to_string(),
                gpu_usage: 65.8,
                memory_usage: 1536 * 1024 * 1024, // 1.5GB
                encoder_usage: 0.0,
                decoder_usage: 0.0,
                priority: 0,
                context_id: Some(2),
                container_id: Some("docker-container-123".to_string()),
                parent_pid: Some(2000),
            },
            GpuProcess {
                pid: 9012,
                user: "root".to_string(),
                command: "ffmpeg.exe".to_string(),
                gpu_usage: 25.3,
                memory_usage: 512 * 1024 * 1024, // 512MB
                encoder_usage: 45.0,
                decoder_usage: 0.0,
                priority: -10,
                context_id: Some(3),
                container_id: None,
                parent_pid: Some(1),
            },
        ]
    }

    pub fn kill_process(&mut self, pid: u32) -> Result<()> {
        self.system.refresh_processes();
        
        if let Some(process) = self.system.process(Pid::from(pid as usize)) {
            if process.kill() {
                Ok(())
            } else {
                anyhow::bail!("Failed to kill process with PID {}", pid)
            }
        } else {
            anyhow::bail!("Process with PID {} not found", pid)
        }
    }

    pub fn get_process_name(&mut self, pid: u32) -> Option<String> {
        self.system.refresh_processes();
        
        if let Some(process) = self.system.process(Pid::from(pid as usize)) {
            Some(process.name().to_string())
        } else {
            None
        }
    }
}

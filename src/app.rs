use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use crate::export::CsvExporter;
use crate::gpu::{GpuInfo, GpuManager};
use crate::health::{HealthMonitor, GpuHealthMetrics, HealthStatus};
use crate::notifications::{NotificationManager, NotificationQueue};
use crate::process::{GpuProcess, ProcessManager};
use crate::settings::{SettingsManager, AppSettings};

#[derive(Debug, Clone)]
pub struct HistoryPoint {
    pub timestamp: DateTime<Local>,
    pub utilization: f32,
    pub memory_usage: f32,
    pub temperature: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    Processes,
    Performance,
    Hardware,
    Health,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortColumn {
    Pid,
    User,
    GpuUsage,
    MemoryUsage,
    Command,
}


pub struct App {
    pub should_quit: bool,
    pub current_gpu: usize,
    pub view_mode: ViewMode,
    pub sort_column: SortColumn,
    pub sort_ascending: bool,
    pub selected_process: Option<usize>,
    pub filter_text: String,
    pub show_command_palette: bool,
    pub show_help: bool,
    pub show_process_details: bool,
    pub show_settings: bool,
    pub update_interval: u64,
    pub debug_mode: bool,
    
    // Data
    pub gpu_manager: GpuManager,
    pub process_manager: ProcessManager,
    pub health_monitor: HealthMonitor,
    pub notification_manager: NotificationManager,
    pub notification_queue: NotificationQueue,
    pub settings_manager: SettingsManager,
    pub gpus: Vec<GpuInfo>,
    pub processes: Vec<GpuProcess>,
    pub history: VecDeque<HistoryPoint>,
    pub health_metrics: Option<GpuHealthMetrics>,
    
    // UI State
    pub panes_collapsed: bool,
    pub tree_view: bool,
    pub scroll_offset: usize,
    pub command_palette_input: String,
    
    // Status message display
    pub status_message: Option<String>,
    pub status_message_time: Option<Instant>,
    pub status_message_duration: Duration,
}

impl App {
    pub async fn new(update_interval: u64, selected_gpu: Option<usize>, debug: bool) -> Result<Self> {
        let gpu_manager = GpuManager::new().await?;
        let process_manager = ProcessManager::new();
        let settings_manager = SettingsManager::new()?;
        let gpus = gpu_manager.get_gpu_info().await?;
        
        let current_gpu = selected_gpu.unwrap_or(0);
        if current_gpu >= gpus.len() {
            anyhow::bail!("GPU index {} not found. Available GPUs: {}", current_gpu, gpus.len());
        }

        Ok(Self {
            should_quit: false,
            current_gpu,
            view_mode: ViewMode::Processes,
            sort_column: SortColumn::GpuUsage,
            sort_ascending: false,
            selected_process: None,
            filter_text: String::new(),
            show_command_palette: false,
            show_help: false,
            show_process_details: false,
            show_settings: false,
            update_interval,
            debug_mode: debug,
            
            gpu_manager,
            process_manager,
            health_monitor: HealthMonitor::new(),
            notification_manager: NotificationManager::new(),
            notification_queue: NotificationQueue::new(),
            settings_manager,
            gpus,
            processes: Vec::new(),
            history: VecDeque::with_capacity(300), // 5 minutes at 1Hz
            health_metrics: None,
            
            panes_collapsed: false,
            tree_view: false,
            scroll_offset: 0,
            command_palette_input: String::new(),
            
            status_message: None,
            status_message_time: None,
            status_message_duration: Duration::from_secs(10),
        })
    }

    pub async fn update(&mut self) -> Result<()> {
        // Update GPU information
        self.gpus = self.gpu_manager.get_gpu_info().await?;
        
        // Update processes
        self.processes = self.process_manager.get_gpu_processes().await?;
        
        // Sort processes
        self.sort_processes();
        
        // Add to history and update health metrics
        if let Some(gpu) = self.gpus.get(self.current_gpu) {
            let history_point = HistoryPoint {
                timestamp: Local::now(),
                utilization: gpu.utilization,
                memory_usage: (gpu.memory_used as f32 / gpu.memory_total as f32) * 100.0,
                temperature: gpu.temperature,
            };
            
            self.history.push_back(history_point);
            if self.history.len() > 300 {
                self.history.pop_front();
            }

            // Update health monitoring
            self.health_metrics = Some(self.health_monitor.update_metrics(
                gpu.temperature,
                gpu.power_draw,
                gpu.memory_used,
                gpu.memory_total,
                gpu.utilization,
                gpu.gpu_clock,
                gpu.memory_clock,
                gpu.throttled,
            ));

            // Health notifications disabled temporarily to avoid PowerShell issues
            // TODO: Re-enable when PowerShell notification issues are resolved
        }

        // Check if status message should be cleared
        self.update_status_message();

        Ok(())
    }

    pub async fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.show_command_palette {
            self.handle_command_palette_key(key).await?;
            return Ok(());
        }


        if self.show_help {
            self.handle_help_key(key).await?;
            return Ok(());
        }

        if self.show_process_details {
            self.handle_process_details_key(key).await?;
            return Ok(());
        }

        if self.show_settings {
            self.handle_settings_key(key).await?;
            return Ok(());
        }

        match key.code {
            // Navigation
            KeyCode::Up => self.select_previous(),
            KeyCode::Down => self.select_next(),
            KeyCode::Char('k') if !key.modifiers.contains(KeyModifiers::ALT) => self.select_previous(),
            KeyCode::Char('j') => self.select_next(),
            KeyCode::PageUp => self.page_up(),
            KeyCode::PageDown => self.page_down(),
            KeyCode::Home => self.select_first(),
            KeyCode::End => self.select_last(),
            
            // GPU switching
            KeyCode::F(1) => self.switch_gpu(0),
            KeyCode::F(2) => self.switch_gpu(1),
            KeyCode::F(3) => self.switch_gpu(2),
            KeyCode::F(4) => self.switch_gpu(3),
            
            // View modes
            KeyCode::Tab => self.next_view_mode(),
            KeyCode::BackTab => self.prev_view_mode(),
            
            // Actions
            KeyCode::Enter => self.show_process_details(),
            KeyCode::Delete => {
                if let Err(e) = self.kill_selected_process().await {
                    // Don't crash the app on kill errors, just log them
                    eprintln!("Error killing process: {}", e);
                }
            },
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::ALT) => {
                if let Err(e) = self.kill_selected_process().await {
                    eprintln!("Error killing process: {}", e);
                }
            },
            
            // UI toggles
            KeyCode::Char('t') => self.tree_view = !self.tree_view,
            KeyCode::Char('c') => self.panes_collapsed = !self.panes_collapsed,
            KeyCode::Char('h') => self.show_help = !self.show_help,
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::ALT) => self.show_settings = !self.show_settings,
            
            // Command palette
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.show_command_palette = true;
                self.command_palette_input.clear();
            },
            
            // Sorting
            KeyCode::Char('1') => self.set_sort_column(SortColumn::Pid),
            KeyCode::Char('2') => self.set_sort_column(SortColumn::User),
            KeyCode::Char('3') => self.set_sort_column(SortColumn::GpuUsage),
            KeyCode::Char('4') => self.set_sort_column(SortColumn::MemoryUsage),
            KeyCode::Char('5') => self.set_sort_column(SortColumn::Command),
            
            // Export functionality
            KeyCode::Char('e') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                // Don't let export errors crash the app
                self.show_status_message("🔄 Starting export...".to_string());
                self.export_current_data();
            },
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.export_processes_csv();
            },
            
            _ => {}
        }

        Ok(())
    }

    async fn handle_command_palette_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_command_palette = false;
                self.command_palette_input.clear();
            },
            KeyCode::Enter => {
                self.execute_command().await?;
                self.show_command_palette = false;
                self.command_palette_input.clear();
            },
            KeyCode::Backspace => {
                self.command_palette_input.pop();
            },
            KeyCode::Char(c) => {
                self.command_palette_input.push(c);
            },
            _ => {}
        }
        Ok(())
    }


    async fn handle_help_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('h') | KeyCode::Esc => {
                self.show_help = false;
            },
            _ => {}
        }
        Ok(())
    }

    async fn handle_process_details_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                self.show_process_details = false;
            },
            _ => {}
        }
        Ok(())
    }

    async fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.show_settings = false;
            },
            KeyCode::Char('r') => {
                // Reset to defaults
                self.settings_manager.reset_to_defaults()?;
            },
            KeyCode::Char('s') => {
                // Save settings
                self.settings_manager.save_settings()?;
                self.notification_manager.send_export_success("Settings saved to config file");
            },
            _ => {}
        }
        Ok(())
    }

    async fn execute_command(&mut self) -> Result<()> {
        let command = self.command_palette_input.trim().to_lowercase();
        
        match command.as_str() {
            "quit" | "q" => self.should_quit = true,
            "sort pid" => self.set_sort_column(SortColumn::Pid),
            "sort user" => self.set_sort_column(SortColumn::User),
            "sort gpu" => self.set_sort_column(SortColumn::GpuUsage),
            "sort memory" => self.set_sort_column(SortColumn::MemoryUsage),
            "sort command" => self.set_sort_column(SortColumn::Command),
            "tree" => self.tree_view = !self.tree_view,
            "collapse" => self.panes_collapsed = !self.panes_collapsed,
            "help" => self.show_help = !self.show_help,
            _ => {}
        }
        
        Ok(())
    }

    fn select_previous(&mut self) {
        if self.processes.is_empty() {
            return;
        }
        
        match self.selected_process {
            Some(i) if i > 0 => self.selected_process = Some(i - 1),
            _ => self.selected_process = Some(self.processes.len() - 1),
        }
    }

    fn select_next(&mut self) {
        if self.processes.is_empty() {
            return;
        }
        
        match self.selected_process {
            Some(i) if i < self.processes.len() - 1 => self.selected_process = Some(i + 1),
            _ => self.selected_process = Some(0),
        }
    }

    fn page_up(&mut self) {
        for _ in 0..10 {
            self.select_previous();
        }
    }

    fn page_down(&mut self) {
        for _ in 0..10 {
            self.select_next();
        }
    }

    fn select_first(&mut self) {
        if !self.processes.is_empty() {
            self.selected_process = Some(0);
        }
    }

    fn select_last(&mut self) {
        if !self.processes.is_empty() {
            self.selected_process = Some(self.processes.len() - 1);
        }
    }

    fn switch_gpu(&mut self, gpu_idx: usize) {
        if gpu_idx < self.gpus.len() {
            self.current_gpu = gpu_idx;
            self.selected_process = None;
        }
    }

    fn next_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Processes => ViewMode::Performance,
            ViewMode::Performance => ViewMode::Hardware,
            ViewMode::Hardware => ViewMode::Health,
            ViewMode::Health => ViewMode::Processes,
        };
    }

    fn prev_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Processes => ViewMode::Health,
            ViewMode::Performance => ViewMode::Processes,
            ViewMode::Hardware => ViewMode::Performance,
            ViewMode::Health => ViewMode::Hardware,
        };
    }

    fn show_process_details(&mut self) {
        if self.selected_process.is_some() && !self.processes.is_empty() {
            self.show_process_details = true;
        }
    }

    async fn kill_selected_process(&mut self) -> Result<()> {
        if let Some(selected_idx) = self.selected_process {
            if let Some(process) = self.processes.get(selected_idx) {
                let pid = process.pid;
                let process_name = process.command.clone();
                
                match self.process_manager.kill_process(pid) {
                    Ok(()) => {
                        eprintln!("Successfully killed process: {} (PID: {})", process_name, pid);
                        // Refresh the process list immediately to show the change
                        self.processes = self.process_manager.get_gpu_processes().await?;
                        self.sort_processes();
                        
                        // Adjust selection if needed
                        if self.selected_process.unwrap_or(0) >= self.processes.len() && !self.processes.is_empty() {
                            self.selected_process = Some(self.processes.len() - 1);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to kill process: {}", e);
                    }
                }
            }
        }
        Ok(())
    }

    fn export_current_data(&mut self) {
        if let Some(gpu) = self.gpus.get(self.current_gpu) {
            let filename = CsvExporter::get_export_filename("gputop_snapshot", "csv");
            
            match CsvExporter::export_current_snapshot(
                gpu,
                &self.processes,
                self.health_metrics.as_ref(),
                &filename,
            ) {
                Ok(()) => {
                    self.show_status_message(format!("✅ Export successful: {}", filename));
                }
                Err(e) => {
                    self.show_status_message(format!("❌ Export failed: {}", e));
                }
            }
        } else {
            self.show_status_message("❌ No GPU selected for export".to_string());
        }
    }

    fn export_processes_csv(&mut self) {
        let filename = CsvExporter::get_export_filename("gputop_processes", "csv");
        
        match CsvExporter::export_processes_csv(&self.processes, &filename) {
            Ok(()) => {
                self.show_status_message(format!("✅ Process export successful: {}", filename));
            }
            Err(e) => {
                self.show_status_message(format!("❌ Process export failed: {}", e));
            }
        }
    }


    fn set_sort_column(&mut self, column: SortColumn) {
        if self.sort_column == column {
            self.sort_ascending = !self.sort_ascending;
        } else {
            self.sort_column = column;
            self.sort_ascending = false;
        }
        self.sort_processes();
    }

    fn sort_processes(&mut self) {
        self.processes.sort_by(|a, b| {
            let cmp = match self.sort_column {
                SortColumn::Pid => a.pid.cmp(&b.pid),
                SortColumn::User => a.user.cmp(&b.user),
                SortColumn::GpuUsage => a.gpu_usage.partial_cmp(&b.gpu_usage).unwrap_or(std::cmp::Ordering::Equal),
                SortColumn::MemoryUsage => a.memory_usage.cmp(&b.memory_usage),
                SortColumn::Command => a.command.cmp(&b.command),
            };
            
            if self.sort_ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    }

    pub fn show_status_message(&mut self, message: String) {
        self.status_message = Some(message);
        self.status_message_time = Some(Instant::now());
    }

    pub fn update_status_message(&mut self) {
        if let Some(time) = self.status_message_time {
            if time.elapsed() >= self.status_message_duration {
                self.status_message = None;
                self.status_message_time = None;
            }
        }
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
        self.status_message_time = None;
    }

}

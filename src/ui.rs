use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    widgets::{
        Block, Borders, BorderType, Clear, Gauge, Paragraph, Row, Table,
        Tabs, Wrap,
    },
    Frame,
};
use crate::app::{App, ViewMode};
use crate::utils;

pub fn draw(f: &mut Frame, app: &App) {
    // Check if we need space for status message
    let status_message_height = if app.status_message.is_some() { 3 } else { 0 };
    
    // Optimized layout with no wasted space
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(9),  // Compact header
            Constraint::Min(8),     // Main content
            Constraint::Length(3),  // Footer
            Constraint::Length(status_message_height),  // Status message
        ])
        .split(f.size());

    draw_header(f, chunks[0], app);
    
    match app.view_mode {
        ViewMode::Processes => draw_processes(f, chunks[1], app),
        ViewMode::Performance => draw_performance(f, chunks[1], app),
        ViewMode::Hardware => draw_hardware(f, chunks[1], app),
        ViewMode::Health => draw_health(f, chunks[1], app),
    }
    
    draw_footer(f, chunks[2], app);
    
    // Draw status message if present
    if status_message_height > 0 {
        draw_status_message(f, chunks[3], app);
    }
    
    // Draw modals
    if app.show_help {
        draw_help_modal(f, app);
    }

    if app.show_settings {
        draw_settings_modal(f, app);
    }
    
    if app.show_command_palette {
        draw_command_palette(f, app);
    }
    
    if app.show_process_details {
        draw_process_details_modal(f, app);
    }
    
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let gpu = match app.gpus.get(app.current_gpu) {
        Some(gpu) => gpu,
        None => return,
    };

    // Compact layout - no wasted space
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // GPU info
            Constraint::Length(3),  // Metrics gauges  
            Constraint::Length(3),  // Additional metrics (merged)
        ])
        .split(area);

    // 🔥 GPU Information with modern colors
    let gpu_info = format!(
        "🔥 {} • 🚗 Driver: {} • 🎯 CUDA: {} • 🧠 Memory: {:.1}GB/{:.1}GB ({:.1}%)", 
        gpu.name, 
        gpu.driver_version,
        gpu.cuda_version.as_ref().unwrap_or(&"N/A".to_string()),
        gpu.memory_used as f64 / (1024.0 * 1024.0 * 1024.0),
        gpu.memory_total as f64 / (1024.0 * 1024.0 * 1024.0),
        (gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0
    );
    
    let info_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" 🖥️  GPU Information ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    
    let info_paragraph = Paragraph::new(gpu_info)
        .block(info_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(info_paragraph, chunks[0]);

    // ⚡ Metrics gauges in a single row
    let util_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),  // GPU usage
            Constraint::Percentage(33),  // Memory usage
            Constraint::Percentage(34),  // Temperature
        ])
        .split(chunks[1]);

    // GPU Usage with color coding
    let gpu_color = get_usage_color(gpu.utilization);
    let gpu_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .title(" ⚡ GPU ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .gauge_style(Style::default().fg(gpu_color).add_modifier(Modifier::BOLD))
        .ratio(gpu.utilization as f64 / 100.0)
        .label(format!("{:.1}%", gpu.utilization));
    f.render_widget(gpu_gauge, util_chunks[0]);

    // Memory Usage
    let mem_usage = (gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0;
    let mem_color = get_usage_color(mem_usage as f32);
    let mem_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .title(" 🧠 Memory ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .gauge_style(Style::default().fg(mem_color).add_modifier(Modifier::BOLD))
        .ratio(mem_usage / 100.0)
        .label(format!("{:.1}%", mem_usage));
    f.render_widget(mem_gauge, util_chunks[1]);

    // Temperature
    let temp_color = get_temp_color(gpu.temperature);
    let temp_gauge = Gauge::default()
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .title(" 🌡️  Temperature ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .gauge_style(Style::default().fg(temp_color).add_modifier(Modifier::BOLD))
        .ratio((gpu.temperature as f64 / 100.0).min(1.0))
        .label(format!("{:.0}°C", gpu.temperature));
    f.render_widget(temp_gauge, util_chunks[2]);

    // 📊 Compact additional metrics with health status
    let health_info = if let Some(health) = &app.health_metrics {
        format!("{} Health: {} ({:.0}/100)", 
                health.status.emoji(),
                health.status.text(),
                health.overall_score)
    } else {
        "⚪ Health: Initializing".to_string()
    };

    let metrics_text = format!(
        "⚡ Power: {}W • 🌀 Fan: {}% • 🔧 GPU Clock: {}MHz • 🧠 Mem Clock: {}MHz • {} • 🎯 Processes: {} • 🚦 Status: {}",
        gpu.power_draw.map_or("N/A".to_string(), |p| format!("{:.0}", p)),
        gpu.fan_speed.map_or("Auto".to_string(), |f| format!("{:.0}", f)),
        gpu.gpu_clock.map_or("N/A".to_string(), |c| c.to_string()),
        gpu.memory_clock.map_or("N/A".to_string(), |c| c.to_string()),
        health_info,
        app.processes.len(),
        if gpu.throttled { "🔴 Throttled" } else { "🟢 Normal" }
    );

    let metrics_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .title(" 📊 System Status ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    
    let metrics_paragraph = Paragraph::new(metrics_text)
        .block(metrics_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(metrics_paragraph, chunks[2]);
}

fn draw_processes(f: &mut Frame, area: Rect, app: &App) {
    let header_cells = [
        "PID", "👤 User", "⚡ GPU%", "🧠 MEM%", "📦 VRAM", "🎥 ENC%", "📺 DEC%", "🔧 Command"
    ]
    .iter()
    .map(|h| ratatui::widgets::Cell::from(*h)
        .style(Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)));
    
    let header = Row::new(header_cells)
        .height(1)
        .bottom_margin(1)
        .style(Style::default().bg(Color::DarkGray));
    
    let rows = app.processes.iter().enumerate().map(|(i, process)| {
        let style = if Some(i) == app.selected_process {
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        } else if i % 2 == 0 {
            Style::default()
                .bg(Color::Rgb(20, 20, 25))
                .fg(Color::White)
        } else {
            Style::default().fg(Color::White)
        };
        
        let memory_mb = process.memory_usage / (1024 * 1024);
        let memory_pct = if let Some(gpu) = app.gpus.get(app.current_gpu) {
            (process.memory_usage as f64 / gpu.memory_total as f64) * 100.0
        } else {
            0.0
        };
        
        let container_indicator = if process.container_id.is_some() { "🐳 " } else { "" };
        
        // Color code GPU usage
        let gpu_usage_color = if process.gpu_usage > 80.0 { "🔴" }
        else if process.gpu_usage > 50.0 { "🟡" }
        else if process.gpu_usage > 0.0 { "🟢" }
        else { "⚫" };
        
        Row::new([
            format!("{}", process.pid),
            process.user.clone(),
            format!("{} {:.1}%", gpu_usage_color, process.gpu_usage),
            format!("{:.1}%", memory_pct),
            format!("{}MB", memory_mb),
            format!("{:.1}%", process.encoder_usage),
            format!("{:.1}%", process.decoder_usage),
            format!("{}{}", container_indicator, process.command),
        ])
        .style(style)
    });

    let table = Table::new(rows)
        .header(header)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title(" 🔧 GPU Processes ")
            .title_style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .widths(&[
            Constraint::Length(8),   // PID
            Constraint::Length(12),  // User
            Constraint::Length(12),  // GPU %
            Constraint::Length(8),   // MEM %
            Constraint::Length(10),  // VRAM
            Constraint::Length(8),   // ENC %
            Constraint::Length(8),   // DEC %
            Constraint::Min(25),     // Command
        ]);

    f.render_widget(table, area);
}

fn draw_performance(f: &mut Frame, area: Rect, app: &App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),   // Real-time metrics bar
            Constraint::Min(6),      // Charts area
        ])
        .split(area);

    // Real-time metrics bar
    draw_realtime_metrics(f, main_chunks[0], app);
    
    // Charts area - 2x2 grid
    let charts_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50),  // Top row
            Constraint::Percentage(50),  // Bottom row
        ])
        .split(main_chunks[1]);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // GPU Utilization
            Constraint::Percentage(50),  // Memory Usage
        ])
        .split(charts_chunks[0]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Temperature
            Constraint::Percentage(50),  // Power & Clock
        ])
        .split(charts_chunks[1]);

    // Draw individual charts
    draw_gpu_utilization_chart(f, top_row[0], app);
    draw_memory_usage_chart(f, top_row[1], app);
    draw_temperature_chart(f, bottom_row[0], app);
    draw_power_clock_chart(f, bottom_row[1], app);
}

fn draw_realtime_metrics(f: &mut Frame, area: Rect, app: &App) {
    let gpu = match app.gpus.get(app.current_gpu) {
        Some(gpu) => gpu,
        None => return,
    };

    let current_time = chrono::Local::now().format("%H:%M:%S");
    let memory_pct = (gpu.memory_used as f64 / gpu.memory_total as f64) * 100.0;
    
    let metrics_text = format!(
        "🕐 {} • ⚡ GPU: {:.1}% • 🧠 Memory: {:.1}% ({:.1}GB/{:.1}GB) • 🌡️ Temp: {:.0}°C • ⚡ Power: {}W • 🔧 GPU: {}MHz • 🧠 Mem: {}MHz",
        current_time,
        gpu.utilization,
        memory_pct,
        gpu.memory_used as f64 / (1024.0 * 1024.0 * 1024.0),
        gpu.memory_total as f64 / (1024.0 * 1024.0 * 1024.0),
        gpu.temperature,
        gpu.power_draw.map_or("N/A".to_string(), |p| format!("{:.0}", p)),
        gpu.gpu_clock.map_or("N/A".to_string(), |c| c.to_string()),
        gpu.memory_clock.map_or("N/A".to_string(), |c| c.to_string())
    );

    let metrics_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" 📊 Real-time Metrics ")
        .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Black));
    
    let metrics_paragraph = Paragraph::new(metrics_text)
        .block(metrics_block)
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(metrics_paragraph, area);
}

fn draw_gpu_utilization_chart(f: &mut Frame, area: Rect, app: &App) {
    let data: Vec<(f64, f64)> = app.history.iter()
        .enumerate()
        .map(|(i, h)| (i as f64, h.utilization as f64))
        .collect();

    let chart_text = create_time_series_chart(&data, "GPU Utilization %", Color::Green, 0.0, 100.0);
    
    let chart_paragraph = Paragraph::new(chart_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title(" 📈 GPU Utilization (%) ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::Green))
        .wrap(Wrap { trim: true });
    f.render_widget(chart_paragraph, area);
}

fn draw_memory_usage_chart(f: &mut Frame, area: Rect, app: &App) {
    let data: Vec<(f64, f64)> = app.history.iter()
        .enumerate()
        .map(|(i, h)| (i as f64, h.memory_usage as f64))
        .collect();

    let chart_text = create_time_series_chart(&data, "Memory Usage %", Color::Blue, 0.0, 100.0);
    
    let chart_paragraph = Paragraph::new(chart_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Blue))
            .title(" 🧠 Memory Usage (%) ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::Blue))
        .wrap(Wrap { trim: true });
    f.render_widget(chart_paragraph, area);
}

fn draw_temperature_chart(f: &mut Frame, area: Rect, app: &App) {
    let data: Vec<(f64, f64)> = app.history.iter()
        .enumerate()
        .map(|(i, h)| (i as f64, h.temperature as f64))
        .collect();

    let chart_text = create_time_series_chart(&data, "Temperature °C", Color::Red, 20.0, 100.0);
    
    let chart_paragraph = Paragraph::new(chart_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Red))
            .title(" 🌡️ Temperature (°C) ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::Red))
        .wrap(Wrap { trim: true });
    f.render_widget(chart_paragraph, area);
}

fn draw_power_clock_chart(f: &mut Frame, area: Rect, app: &App) {
    let gpu = match app.gpus.get(app.current_gpu) {
        Some(gpu) => gpu,
        None => return,
    };

    let power_text = format!(
        "⚡ POWER & CLOCKS\n\n\
        🔥 Power Draw: {}W\n\
        🔧 GPU Clock: {}MHz\n\
        🧠 Memory Clock: {}MHz\n\
        🌀 Fan Speed: {}%\n\
        📊 Processes: {}\n\
        🚦 Status: {}",
        gpu.power_draw.map_or("N/A".to_string(), |p| format!("{:.0}", p)),
        gpu.gpu_clock.map_or("N/A".to_string(), |c| c.to_string()),
        gpu.memory_clock.map_or("N/A".to_string(), |c| c.to_string()),
        gpu.fan_speed.map_or("Auto".to_string(), |f| format!("{:.0}", f)),
        app.processes.len(),
        if gpu.throttled { "🔴 Throttled" } else { "🟢 Normal" }
    );
    
    let power_paragraph = Paragraph::new(power_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" ⚡ Power & Clocks ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(power_paragraph, area);
}

fn create_time_series_chart(data: &[(f64, f64)], _label: &str, color: Color, min_val: f64, max_val: f64) -> String {
    if data.is_empty() {
        return "No data available".to_string();
    }

    let height = 8; // Chart height in characters
    let width = 60; // Chart width in characters
    
    // Create Y-axis labels
    let y_labels = (0..=4)
        .map(|i| {
            let val = min_val + (max_val - min_val) * (4 - i) as f64 / 4.0;
            format!("{:5.0}", val)
        })
        .collect::<Vec<_>>();

    // Create the chart data
    let chart_data: Vec<u64> = data.iter()
        .map(|(_, val)| {
            let normalized = ((val - min_val) / (max_val - min_val) * 7.0).max(0.0).min(7.0) as u64;
            normalized
        })
        .collect();
    
    let sparkline = utils::create_sparkline(&chart_data);
    
    // Get current, min, max values for summary
    let current = data.last().map(|(_, v)| *v).unwrap_or(0.0);
    let min_data = data.iter().map(|(_, v)| *v).fold(f64::INFINITY, f64::min);
    let max_data = data.iter().map(|(_, v)| *v).fold(f64::NEG_INFINITY, f64::max);
    
    let chart_color = match color {
        Color::Green => "🟢",
        Color::Blue => "🔵", 
        Color::Red => "🔴",
        _ => "⚪",
    };
    
    format!(
        "┌─────────────────────────────────────────────────────┐\n\
         │ {}                                                    │\n\
         │ {}                                           │\n\
         │                                                     │\n\
         │ {} Current: {:.1}  Min: {:.1}  Max: {:.1}              │\n\
         │ ⏱️  Time Range: Last {} points                      │\n\
         └─────────────────────────────────────────────────────┘",
        sparkline,
        " ".repeat(52),
        chart_color,
        current,
        min_data,
        max_data,
        data.len().min(300)
    )
}

fn draw_hardware(f: &mut Frame, area: Rect, app: &App) {
    if let Some(gpu) = app.gpus.get(app.current_gpu) {
        let info_text = format!(
            "🖥️  GPU: {}\n\
            🏭 Vendor: {:?}\n\
            🚗 Driver Version: {}\n\
            🎯 CUDA Version: {}\n\
            🧠 Memory Total: {:.2} GB\n\
            📊 Memory Used: {:.2} GB\n\
            💿 Memory Free: {:.2} GB\n\
            🌡️  Temperature: {:.0}°C\n\
            🌀 Fan Speed: {}%\n\
            ⚡ Power Draw: {}W\n\
            🔥 GPU Clock: {}MHz\n\
            🧠 Memory Clock: {}MHz\n\
            🚦 Throttled: {}",
            gpu.name,
            gpu.vendor,
            gpu.driver_version,
            gpu.cuda_version.as_ref().unwrap_or(&"N/A".to_string()),
            gpu.memory_total as f64 / (1024.0 * 1024.0 * 1024.0),
            gpu.memory_used as f64 / (1024.0 * 1024.0 * 1024.0),
            (gpu.memory_total - gpu.memory_used) as f64 / (1024.0 * 1024.0 * 1024.0),
            gpu.temperature,
            gpu.fan_speed.map_or("Auto".to_string(), |f| format!("{:.0}", f)),
            gpu.power_draw.map_or("N/A".to_string(), |p| format!("{:.0}", p)),
            gpu.gpu_clock.map_or("N/A".to_string(), |c| c.to_string()),
            gpu.memory_clock.map_or("N/A".to_string(), |c| c.to_string()),
            if gpu.throttled { "🔴 Yes" } else { "🟢 No" }
        );

        let hardware_paragraph = Paragraph::new(info_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta))
                .title(" 🔍 Hardware Details ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(Color::Black)))
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: true });
        f.render_widget(hardware_paragraph, area);
    }
}

fn draw_health(f: &mut Frame, area: Rect, app: &App) {
    if let Some(health) = &app.health_metrics {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),   // Health overview
                Constraint::Min(6),      // Detailed metrics and alerts
            ])
            .split(area);

        // Health Overview
        draw_health_overview(f, chunks[0], health);
        
        // Detailed metrics and alerts
        draw_health_details(f, chunks[1], health, app);
    } else {
        let loading_text = "🔄 Health monitoring initializing...\n\nGPU health metrics will appear here once monitoring begins.\nThis may take a few seconds.";
        
        let loading_paragraph = Paragraph::new(loading_text)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" 🏥 GPU Health Monitor ")
                .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
                .style(Style::default().bg(Color::Black)))
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(loading_paragraph, area);
    }
}

fn draw_health_overview(f: &mut Frame, area: Rect, health: &crate::health::GpuHealthMetrics) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),  // Overall health score
            Constraint::Percentage(25),  // Temperature status
            Constraint::Percentage(25),  // Power status  
            Constraint::Percentage(25),  // Memory status
        ])
        .split(area);

    // Overall Health Score
    let health_score_text = format!(
        "{}\n\n{} {}\n{:.0}/100\n\nUptime: {:.1}h",
        health.status.emoji(),
        health.status.emoji(),
        health.status.text(),
        health.overall_score,
        health.uptime_hours
    );
    
    let score_color = match health.status {
        crate::health::HealthStatus::Excellent => Color::Green,
        crate::health::HealthStatus::Good => Color::Blue,
        crate::health::HealthStatus::Warning => Color::Yellow,
        crate::health::HealthStatus::Critical => Color::Red,
    };

    let health_paragraph = Paragraph::new(health_score_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(score_color))
            .title(" 🏥 Overall Health ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(health_paragraph, chunks[0]);

    // Temperature Health
    let temp_status = if health.temperature.current >= health.temperature.critical {
        ("🔥", "Critical", Color::Red)
    } else if health.temperature.current >= health.temperature.max_safe {
        ("🌡️", "High", Color::Yellow)
    } else if health.temperature.current > 70.0 {
        ("🟡", "Warm", Color::Yellow)
    } else {
        ("❄️", "Cool", Color::Green)
    };

    let temp_text = format!(
        "🌡️ Temperature\n\n{} {}\n{:.0}°C\n\nTrend: {:+.1}°C/5min\nPeak: {:.0}°C",
        temp_status.0,
        temp_status.1,
        health.temperature.current,
        health.temperature.trend_5min,
        health.temperature.peak_today
    );

    let temp_paragraph = Paragraph::new(temp_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(temp_status.2))
            .title(" 🌡️ Temperature ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(temp_paragraph, chunks[1]);

    // Power Health
    let power_text = format!(
        "⚡ Power\n\nCurrent: {:.0}W\nEfficiency: {:.1}\nAvg 1hr: {:.0}W\n\nSpikes: {}",
        health.power.current_draw,
        health.power.efficiency,
        health.power.avg_draw_1hr,
        health.power.power_spikes
    );

    let power_paragraph = Paragraph::new(power_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Magenta))
            .title(" ⚡ Power ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(power_paragraph, chunks[2]);

    // Memory Health
    let memory_leak_status = if health.memory.leak_suspicion > 0.7 {
        ("🔴", "Leak Likely")
    } else if health.memory.leak_suspicion > 0.3 {
        ("🟡", "Monitor")
    } else {
        ("🟢", "Healthy")
    };

    let memory_text = format!(
        "🧠 Memory\n\n{} {}\nTrend: {:+.0}MB/min\n\nFragmentation: {:.0}%\nPeak: {:.1}GB",
        memory_leak_status.0,
        memory_leak_status.1,
        health.memory.usage_trend,
        health.memory.fragmentation_score * 100.0,
        health.memory.peak_usage_today as f64 / (1024.0 * 1024.0 * 1024.0)
    );

    let memory_color = if health.memory.leak_suspicion > 0.7 {
        Color::Red
    } else if health.memory.leak_suspicion > 0.3 {
        Color::Yellow
    } else {
        Color::Cyan
    };

    let memory_paragraph = Paragraph::new(memory_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(memory_color))
            .title(" 🧠 Memory ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(memory_paragraph, chunks[3]);
}

fn draw_health_details(f: &mut Frame, area: Rect, health: &crate::health::GpuHealthMetrics, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50),  // Recent alerts
            Constraint::Percentage(50),  // Detailed metrics
        ])
        .split(area);

    // Recent Alerts
    let recent_alerts = app.health_monitor.get_recent_alerts(10);
    let alerts_text = if recent_alerts.is_empty() {
        "🟢 No recent alerts\n\nAll systems operating normally.\nHealth monitoring is active.".to_string()
    } else {
        let mut alert_lines = vec!["⚠️  Recent Alerts:\n".to_string()];
        for (i, alert) in recent_alerts.iter().enumerate() {
            if i >= 8 { // Limit to 8 alerts to fit in UI
                break;
            }
            let time_str = alert.timestamp.format("%H:%M:%S").to_string();
            alert_lines.push(format!(
                "{} [{}] {}",
                alert.alert_type.emoji(),
                time_str,
                alert.message
            ));
        }
        alert_lines.join("\n")
    };

    let alerts_paragraph = Paragraph::new(alerts_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" ⚠️  Alerts & Events ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(alerts_paragraph, chunks[0]);

    // Detailed Metrics
    let details_text = format!(
        "📊 DETAILED METRICS\n\n\
        🌡️ TEMPERATURE:\n\
        • Current: {:.1}°C\n\
        • Safe Limit: {:.1}°C\n\
        • Critical: {:.1}°C\n\
        • Time >80°C: {}min\n\n\
        ⚡ POWER & PERFORMANCE:\n\
        • Current Draw: {:.0}W\n\
        • Efficiency: {:.2} util/W\n\
        • Power Spikes: {}\n\n\
        🧠 MEMORY HEALTH:\n\
        • Usage Trend: {:+.0} MB/min\n\
        • Fragmentation: {:.1}%\n\
        • Leak Risk: {:.0}%\n\n\
        🚦 SYSTEM STATUS:\n\
        • Throttling: {}\n\
        • Monitoring: {:.1}h",
        health.temperature.current,
        health.temperature.max_safe,
        health.temperature.critical,
        health.temperature.time_above_80c / 60,
        health.power.current_draw,
        health.power.efficiency,
        health.power.power_spikes,
        health.memory.usage_trend,
        health.memory.fragmentation_score * 100.0,
        health.memory.leak_suspicion * 100.0,
        if health.thermal_throttling_detected { "🔴 Yes" } else { "🟢 No" },
        health.uptime_hours
    );

    let details_paragraph = Paragraph::new(details_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" 📊 Detailed Health Metrics ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)))
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(details_paragraph, chunks[1]);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help_text = match app.view_mode {
        ViewMode::Processes => "q=Quit • ↑↓=Nav • Enter=Details • Del=Kill • Ctrl+E=Export • h=Help",
        ViewMode::Performance => "q=Quit • F1-F4=GPU • Ctrl+E=Export • h=Help",
        ViewMode::Hardware => "q=Quit • F1-F4=GPU • Ctrl+E=Export • h=Help",
        ViewMode::Health => "q=Quit • F1-F4=GPU • Ctrl+E=Export • h=Help",
    };

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(area);

    // 🎮 Controls with bright colors
    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" 🎮 Controls ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD));
    f.render_widget(help_paragraph, chunks[0]);

    // 📋 Tabs
    let tabs = Tabs::new(vec!["🔧 Proc", "📊 Perf", "🖥️ HW", "🏥 Health"])
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" 📋 Views ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .select(match app.view_mode {
            ViewMode::Processes => 0,
            ViewMode::Performance => 1,
            ViewMode::Hardware => 2,
            ViewMode::Health => 3,
        })
        .style(Style::default().fg(Color::Gray))
        .highlight_style(Style::default()
            .fg(Color::Yellow)
            .bg(Color::Blue)
            .add_modifier(Modifier::BOLD));
    f.render_widget(tabs, chunks[1]);
}

fn draw_help_modal(f: &mut Frame, _app: &App) {
    let area = centered_rect(70, 90, f.size());
    f.render_widget(Clear, area);

    let help_text = "🚀 GPUTop - Professional GPU Monitoring\n\n\
🎮 NAVIGATION:\n\
↑↓ / k j          Navigate up/down\n\
PgUp/PgDn         Navigate by page\n\
Home/End          Go to first/last\n\
Tab/Shift+Tab     Switch view modes\n\
F1-F4             Switch GPU (0-3)\n\n\
🎯 ACTIONS:\n\
Enter             Show process details\n\
Delete / Alt+K    Kill selected process\n\
Ctrl+P            Open command palette\n\n\
📁 EXPORT:\n\
Ctrl+E            Export full snapshot to CSV\n\
Ctrl+S            Export processes to CSV\n\n\
📊 SORTING (Processes View):\n\
1                 Sort by PID\n\
2                 Sort by User\n\
3                 Sort by GPU Usage\n\
4                 Sort by Memory\n\
5                 Sort by Command\n\n\
🎨 UI CONTROLS:\n\
t                 Toggle tree view\n\
c                 Collapse/expand panes\n\
h                 Toggle this help\n\
Alt+S             Open settings panel\n\n\
🚪 GENERAL:\n\
q / Ctrl+C        Quit application\n\
Esc               Close modals\n\n\
Press 'h' or ESC to close this help";

    let help_paragraph = Paragraph::new(help_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Yellow))
            .title(" ❓ Help & Controls ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(help_paragraph, area);
}

fn draw_settings_modal(f: &mut Frame, app: &App) {
    let area = centered_rect(80, 90, f.size());
    f.render_widget(Clear, area);

    let settings = app.settings_manager.get_settings();
    
    let settings_text = format!(
        "⚙️  GPUTop Settings Configuration\n\n\
📊 HEALTH THRESHOLDS:\n\
Temperature Warning:     {:.1}°C\n\
Temperature Critical:    {:.1}°C\n\
Power Warning:           {:.1}%\n\
Power Critical:          {:.1}%\n\
Memory Warning:          {:.1}%\n\
Memory Critical:         {:.1}%\n\
Low Utilization:         {:.1}%\n\
High Utilization:        {:.1}%\n\n\
🔔 NOTIFICATIONS:\n\
Enabled:                 {}\n\
Min Interval:            {}s\n\
Export Notifications:    {}\n\
Process Notifications:   {}\n\n\
⏱️  PERFORMANCE:\n\
Update Interval:         {}ms\n\
Max History Points:      {}\n\n\
💾 ACTIONS:\n\
s = Save Settings    r = Reset to Defaults    Esc = Close\n\
\n\
Settings file: ~/.config/gputop/settings.json",
        settings.health_thresholds.temperature_warning,
        settings.health_thresholds.temperature_critical,
        settings.health_thresholds.power_warning,
        settings.health_thresholds.power_critical,
        settings.health_thresholds.memory_usage_warning,
        settings.health_thresholds.memory_usage_critical,
        settings.health_thresholds.utilization_low,
        settings.health_thresholds.utilization_high,
        if settings.notification_settings.enabled { "Yes" } else { "No" },
        settings.notification_settings.min_interval_seconds,
        if settings.notification_settings.show_export_notifications { "Yes" } else { "No" },
        if settings.notification_settings.show_process_notifications { "Yes" } else { "No" },
        settings.update_interval_ms,
        settings.max_history_points
    );

    let settings_paragraph = Paragraph::new(settings_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(" ⚙️  Settings ")
            .title_style(Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(settings_paragraph, area);
}

fn draw_command_palette(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 3, f.size());
    f.render_widget(Clear, area);

    let input_text = format!("❯ {}", app.command_palette_input);
    let input_paragraph = Paragraph::new(input_text)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Green))
            .title(" 🎛️  Command Palette ")
            .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .style(Style::default().bg(Color::Black)))
        .style(Style::default().fg(Color::White));
    f.render_widget(input_paragraph, area);
}

fn draw_process_details_modal(f: &mut Frame, app: &App) {
    if let Some(selected_idx) = app.selected_process {
        if let Some(process) = app.processes.get(selected_idx) {
            let area = centered_rect(80, 70, f.size());
            f.render_widget(Clear, area);

            let memory_mb = process.memory_usage / (1024 * 1024);
            let memory_gb = memory_mb as f64 / 1024.0;
            
            let gpu_memory_pct = if let Some(gpu) = app.gpus.get(app.current_gpu) {
                (process.memory_usage as f64 / gpu.memory_total as f64) * 100.0
            } else {
                0.0
            };

            let container_info = if let Some(container) = &process.container_id {
                format!("🐳 Container: {}", container)
            } else {
                "📦 Native Process".to_string()
            };

            let parent_info = if let Some(parent) = process.parent_pid {
                format!("👨‍👩‍👧‍👦 Parent PID: {}", parent)
            } else {
                "🌱 Root Process".to_string()
            };

            let details_text = format!(
                "📋 PROCESS DETAILS\n\n\
                🔧 Command: {}\n\
                🆔 PID: {}\n\
                👤 User: {}\n\
                {}\n\
                {}\n\n\
                📊 RESOURCE USAGE:\n\
                ⚡ GPU Usage: {:.1}%\n\
                🧠 Memory Usage: {:.1}% ({:.1} GB / {} MB)\n\
                🎥 Encoder Usage: {:.1}%\n\
                📺 Decoder Usage: {:.1}%\n\n\
                🔧 TECHNICAL INFO:\n\
                🎯 Priority: {}\n\
                📈 Context ID: {}\n\n\
                Press ESC or Enter to close",
                process.command,
                process.pid,
                process.user,
                container_info,
                parent_info,
                process.gpu_usage,
                gpu_memory_pct,
                memory_gb,
                memory_mb,
                process.encoder_usage,
                process.decoder_usage,
                process.priority,
                process.context_id.map_or("N/A".to_string(), |id| id.to_string())
            );

            let details_paragraph = Paragraph::new(details_text)
                .block(Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title(" 🔍 Process Information ")
                    .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                    .style(Style::default().bg(Color::Black)))
                .style(Style::default().fg(Color::White))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            f.render_widget(details_paragraph, area);
        }
    }
}

fn draw_status_message(f: &mut Frame, area: Rect, app: &App) {
    if let Some(message) = &app.status_message {
        let message_paragraph = Paragraph::new(message.clone())
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green))
                .title(" Status ")
                .title_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
        f.render_widget(message_paragraph, area);
    }
}


// Helper functions for color coding
fn get_usage_color(usage: f32) -> Color {
    if usage > 80.0 {
        Color::Red
    } else if usage > 60.0 {
        Color::Yellow
    } else if usage > 30.0 {
        Color::Green
    } else {
        Color::Blue
    }
}

fn get_temp_color(temp: f32) -> Color {
    if temp > 85.0 {
        Color::Red
    } else if temp > 75.0 {
        Color::Yellow
    } else {
        Color::Green
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
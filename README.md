# 🚀 GPUTop - Ultimate GPU Monitoring & Health CLI

A professional, real-time GPU monitoring tool built in Rust with a modern terminal user interface. GPUTop provides comprehensive GPU utilization tracking, process management, health monitoring, data export, and performance analytics in a beautiful, responsive CLI interface.

![GPUTop Demo](https://img.shields.io/badge/Platform-Linux%20%7C%20Windows%20%7C%20macOS-blue)
![Rust](https://img.shields.io/badge/Language-Rust-orange)
![License](https://img.shields.io/badge/License-MIT-green)
![Version](https://img.shields.io/badge/Version-1.0.1-brightgreen)

## ✨ Features

### 🔥 Core Monitoring
- **Real-time GPU metrics** - Utilization, memory, temperature, power draw, fan speed
- **Multi-GPU support** - Monitor up to 4 GPUs simultaneously (F1-F4 switching)
- **Process tracking** - Live GPU process monitoring with detailed statistics
- **Historical data** - Performance trends with sparkline charts (300 data points)
- **Cross-vendor support** - NVIDIA (NVML) with extensible architecture for AMD/Intel

### 🏥 Advanced Health Monitoring
- **GPU Health Score** - Comprehensive health assessment (0-100 scale)
- **Temperature analytics** - Trends, peaks, time above critical thresholds
- **Power analysis** - Efficiency metrics, spike detection, average consumption
- **Memory health** - Leak detection, fragmentation analysis, usage trends
- **Thermal throttling detection** - Real-time throttling status monitoring
- **Health alerts** - Intelligent alert system for critical conditions
- **Uptime tracking** - Monitor GPU operation time and stability

### 📊 Data Export & Analytics
- **CSV Export** - Full system snapshots with Ctrl+E
- **Process Export** - GPU process data export with Ctrl+S
- **Health Reports** - Export health metrics and alerts
- **Timestamped data** - All exports include precise timestamps
- **Multiple formats** - GPU metrics, process data, health analytics
- **Automated naming** - Time-stamped filenames for easy organization

### 🎨 Modern Interface
- **Responsive design** - Adapts to terminal size automatically
- **Interactive controls** - Mouse and keyboard navigation
- **Command palette** - Quick access to all functions (Ctrl+P)
- **Contextual help** - Built-in help system (h key)
- **Modern styling** - Clean borders, color-coded metrics, emojis
- **Status messages** - Real-time feedback for actions (10-second display)
- **Modal dialogs** - Settings, help, process details overlays

### 🔧 Process Management
- **Live process list** - Real-time GPU process monitoring
- **Sortable columns** - Sort by PID, user, GPU usage, memory, command
- **Process termination** - Terminate GPU processes (Delete key)
- **Container awareness** - Docker container detection (🐳 indicator)
- **Detailed metrics** - GPU%, memory%, encoder/decoder usage
- **Process details modal** - Full process information view

### ⚙️ Configuration & Settings
- **Settings management** - Configurable health thresholds
- **Notification system** - Desktop notifications for alerts
- **Custom thresholds** - Temperature, power, memory warning levels
- **Update intervals** - Configurable monitoring frequency
- **Persistent settings** - Settings saved to user config directory
- **Settings UI** - Interactive settings panel (Alt+S)

### 🎯 View Modes (4 Total)
1. **🔧 Processes** - Live GPU process monitoring and management
2. **📊 Performance** - Real-time charts and metrics dashboard  
3. **🖥️ Hardware** - Detailed GPU specifications and information
4. **🏥 Health** - Comprehensive health monitoring and alerts

## 🛠️ Installation

### Prerequisites
- **Rust** 1.70+ (Install from [rustup.rs](https://rustup.rs/))
- **GPU drivers** with monitoring APIs:
  - NVIDIA: Latest drivers with NVML support

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/gputop.git
cd gputop

# Build with NVIDIA support (default)
cargo build --release

# Build with specific features
cargo build --release --features nvidia

# Install globally
cargo install --path .
```

### Quick Start

```bash
# Run with default settings
./target/release/gputop

# Run with custom update interval (500ms)
./target/release/gputop --interval 500

# Monitor specific GPU (0-indexed)
./target/release/gputop --gpu 1

# Enable debug mode
./target/release/gputop --debug
```

## 🎮 Controls & Usage

### Navigation
| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Navigate process list |
| `Page Up`/`Page Down` | Navigate by page |
| `Home`/`End` | Go to first/last process |
| `Tab`/`Shift+Tab` | Switch view modes |
| `F1`-`F4` | Switch GPU (0-3) |

### Actions
| Key | Action |
|-----|--------|
| `Enter` | Show process details modal |
| `Delete` or `Alt+K` | Kill selected process |
| `Ctrl+P` | Open command palette |
| `Ctrl+E` | Export full system snapshot to CSV |
| `Ctrl+S` | Export processes to CSV |
| `h` | Toggle help modal |
| `Alt+S` | Open settings panel |
| `q` or `Ctrl+C` | Quit application |

### Sorting (Processes View)
| Key | Sort Column |
|-----|-------------|
| `1` | PID |
| `2` | User |
| `3` | GPU Usage |
| `4` | Memory Usage |
| `5` | Command |

### UI Controls
| Key | Action |
|-----|--------|
| `t` | Toggle tree view |
| `c` | Collapse/expand panes |
| `Esc` | Close modals |

## 📋 View Modes

### 🔧 Processes View
- Live GPU process monitoring with real-time updates
- Sortable process table with color-coded usage indicators
- Container detection with 🐳 indicator
- Process management (kill with Delete key)
- Memory and GPU usage per process
- Process details modal with comprehensive information

### 📊 Performance View
- Real-time metrics dashboard with live updates
- GPU utilization charts with sparklines (5-minute history)
- Memory usage trends and analysis
- Temperature monitoring with color coding
- Power and clock analysis in organized layout
- Current metrics bar with timestamp

### 🖥️ Hardware View
- Detailed GPU specifications and capabilities
- Driver and CUDA versions
- Memory information (total, used, free)
- Thermal and power status
- Clock frequencies and fan speeds
- Throttling status and vendor information

### 🏥 Health View (NEW!)
- **Overall Health Score** - Comprehensive 0-100 health rating
- **Temperature Analysis** - Current temp, trends, critical thresholds
- **Power Health** - Efficiency metrics, spike detection, consumption
- **Memory Health** - Leak detection, fragmentation, usage patterns
- **Recent Alerts** - Real-time health alert feed
- **Detailed Metrics** - Comprehensive health statistics
- **Uptime Tracking** - System stability monitoring

## 📊 Data Export Features

### Export Types
| Export | Hotkey | Content |
|--------|--------|---------|
| **Full Snapshot** | `Ctrl+E` | Complete system state: GPU info, processes, health metrics |
| **Process Data** | `Ctrl+S` | Current GPU processes with detailed statistics |
| **Health Report** | Via API | Health metrics, alerts, and analysis data |

### Export Format
- **CSV Format** - Standard comma-separated values
- **Timestamped** - All data includes precise timestamps
- **Auto-naming** - Files named with date/time (e.g., `gputop_snapshot_20240101_143022.csv`)
- **Comprehensive** - Includes all available metrics and metadata

### Export Data Includes
- GPU specifications and current state
- Process information (PID, user, command, usage)
- Health metrics and scores
- Temperature, power, and memory analytics
- Historical trends and alerts
- System configuration and uptime

## ⚙️ Settings & Configuration

### Settings Management
- **Interactive Settings Panel** - Access with `Alt+S`
- **Health Thresholds** - Customize temperature, power, memory limits  
- **Notification Settings** - Configure desktop notifications
- **Update Intervals** - Adjust monitoring frequency
- **Persistent Storage** - Settings saved to `~/.config/gputop/settings.json`

### Configurable Thresholds
```json
{
  "health_thresholds": {
    "temperature_warning": 80.0,
    "temperature_critical": 90.0,
    "power_warning": 85.0,
    "power_critical": 95.0,
    "memory_usage_warning": 80.0,
    "memory_usage_critical": 95.0
  }
}
```

### Command Line Options

```bash
gputop [OPTIONS]

OPTIONS:
    -i, --interval <INTERVAL>    Update interval in milliseconds [default: 1000]
    -g, --gpu <GPU>             GPU to monitor (0-indexed)
    -d, --debug                 Enable debug mode
    -h, --help                  Print help information
    -V, --version               Print version information
```

### Command Palette Commands

Access with `Ctrl+P`:
- `quit` or `q` - Exit application
- `sort pid` - Sort by Process ID
- `sort user` - Sort by User
- `sort gpu` - Sort by GPU usage
- `sort memory` - Sort by Memory usage
- `sort command` - Sort by Command
- `tree` - Toggle tree view
- `collapse` - Toggle pane collapse
- `help` - Show help modal

## 🏗️ Architecture

GPUTop is built with a modular architecture:

```
src/
├── main.rs          # Application entry point & CLI parsing
├── app.rs           # Application state & event handling  
├── gpu.rs           # GPU detection & monitoring
├── process.rs       # Process management & detection
├── ui.rs           # Terminal UI rendering
├── health.rs        # Health monitoring system
├── export.rs        # Data export functionality
├── settings.rs      # Configuration management
├── notifications.rs # Desktop notification system
└── utils.rs        # Utility functions & helpers
```

### Key Components

- **GPU Manager** - NVIDIA GPU detection via NVML with fallback mock data
- **Process Manager** - GPU process tracking with system process filtering and termination
- **Health Monitor** - Advanced health analytics and alerting system
- **Export System** - Comprehensive data export with multiple formats
- **Settings Manager** - Configuration persistence and management
- **Notification System** - Desktop alerts for critical conditions
- **UI Engine** - Modern terminal interface with ratatui and modal support
- **Event System** - Async event handling with tokio

## 🔌 GPU Vendor Support

### NVIDIA (Fully Implemented)
- **NVML integration** - Complete hardware monitoring via nvml-wrapper
- **Process tracking** - Real-time GPU process detection and management
- **Power monitoring** - Wattage, thermal, clock speeds, fan control
- **Multi-GPU** - Support for multiple NVIDIA GPUs with switching
- **Health analytics** - Advanced health scoring and trend analysis

### AMD & Intel (Planned)
- Framework ready for extension
- Placeholder in gpu.rs for additional vendor support
- Process detection works for any GPU vendor
- Health monitoring system vendor-agnostic

### Fallback Mode
- Mock GPU data when no hardware detected
- Demonstrates all features without GPU hardware
- System process filtering for GPU-intensive applications
- Full UI functionality for development and testing

## 🏥 Health Monitoring System

### Health Scoring Algorithm
The health score (0-100) combines multiple factors:
- **Temperature Health** (40% weight) - Based on current temp vs. safe limits
- **Power Efficiency** (30% weight) - GPU utilization per watt consumed
- **Memory Health** (30% weight) - Leak detection and fragmentation analysis

### Alert System
- **Temperature Alerts** - Warning at 80°C, critical at 90°C
- **Power Alerts** - Efficiency drops and consumption spikes
- **Memory Alerts** - Potential leaks and high fragmentation
- **Thermal Throttling** - Real-time throttling detection

### Health Status Levels
- 🟢 **Excellent** (90-100) - Optimal performance, no issues
- 🔵 **Good** (70-89) - Normal operation, minor concerns
- 🟡 **Warning** (50-69) - Attention needed, potential issues
- 🔴 **Critical** (0-49) - Immediate attention required

## 🔔 Notification System

### Desktop Notifications
- **Export Success/Failure** - File export status updates
- **Health Alerts** - Critical condition notifications  
- **Process Events** - Process termination confirmations
- **System Status** - Throttling and performance alerts

### Notification Settings
- **Enable/Disable** - Toggle notifications on/off
- **Minimum Interval** - Prevent notification spam
- **Selective Types** - Choose which events trigger notifications

## 🤝 Contributing

We welcome contributions! 

### Development Setup

```bash
# Clone and setup
git clone https://github.com/yourusername/gputop.git
cd gputop

# Install development dependencies
rustup component add rustfmt clippy

# Run with mock data (no GPU required)
cargo run

# Format code
cargo fmt

# Lint code
cargo clippy
```

### Adding New Features
- **Health Metrics** - Extend health monitoring in `src/health.rs`
- **Export Formats** - Add new export types in `src/export.rs`
- **GPU Vendors** - Add AMD/Intel support in `src/gpu.rs`
- **UI Components** - Enhance interface in `src/ui.rs`

## 📦 Dependencies

### Core Dependencies
- **ratatui** - Modern terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **tokio** - Async runtime for real-time updates
- **sysinfo** - System process information
- **clap** - Command-line argument parsing
- **chrono** - Date/time handling for timestamps
- **serde** - Serialization for settings and export

### GPU & System Support
- **nvml-wrapper** - NVIDIA GPU monitoring (optional feature)
- **nix** - Unix process signals (Unix only)
- **dirs** - User configuration directory access
- **anyhow** - Error handling and propagation

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **[ratatui](https://github.com/ratatui-org/ratatui)** - Modern terminal UI framework
- **[nvml-wrapper](https://github.com/Cldfire/nvml-wrapper)** - NVIDIA GPU monitoring
- **[sysinfo](https://github.com/GuillaumeGomez/sysinfo)** - System information
- **[tokio](https://github.com/tokio-rs/tokio)** - Async runtime

## 🐛 Known Issues

- Windows process termination requires proper permissions
- AMD/Intel GPU support not yet implemented  
- Mock data shown when no compatible GPU hardware detected
- Process filtering heuristic may include non-GPU processes
- PowerShell notification system temporarily disabled on Windows

## 📊 Performance

GPUTop is designed to be lightweight and efficient:

- **Memory usage**: ~8-15MB RAM (increased due to health monitoring)
- **CPU usage**: <1% on modern systems
- **Update frequency**: Configurable (default 1Hz)
- **Startup time**: <500ms
- **Data retention**: 300 history points (5 minutes at 1Hz)
- **Export speed**: Sub-second for typical datasets

## 🚀 Roadmap

### Upcoming Features
- **AMD GPU Support** - ROCm/ROC-SMI integration
- **Intel GPU Support** - Intel GPU monitoring APIs
- **Network Monitoring** - GPU cluster monitoring
- **Advanced Analytics** - Machine learning health predictions
- **REST API** - HTTP API for external integrations
- **Plugin System** - Extensible monitoring plugins

### Performance Improvements
- **Optimized Rendering** - Reduced terminal refresh overhead
- **Memory Optimization** - Circular buffer improvements
- **Background Processing** - Non-blocking health calculations
- **Export Streaming** - Large dataset export optimization

---

**Made with ❤️ in Rust | Version 1.0.1 - UI & Health Monitoring Release**
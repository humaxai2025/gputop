use anyhow::Result;

#[derive(Debug, Clone)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub driver_version: String,
    pub cuda_version: Option<String>,
    pub utilization: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub temperature: f32,
    pub fan_speed: Option<f32>,
    pub power_draw: Option<f32>,
    pub gpu_clock: Option<u32>,
    pub memory_clock: Option<u32>,
    pub throttled: bool,
    pub vendor: GpuVendor,
}

#[derive(Debug, Clone, PartialEq)]
pub enum GpuVendor {
    Nvidia,
    Amd,
    Intel,
    Unknown,
}

pub struct GpuManager {
    #[cfg(feature = "nvidia")]
    nvml: Option<nvml_wrapper::Nvml>,
}

impl GpuManager {
    pub async fn new() -> Result<Self> {
        Ok(Self {
            #[cfg(feature = "nvidia")]
            nvml: nvml_wrapper::Nvml::init().ok(),
        })
    }

    pub async fn get_gpu_info(&self) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();
        
        #[cfg(feature = "nvidia")]
        if let Some(nvml) = &self.nvml {
            gpus.extend(self.get_nvidia_info(nvml)?);
        }
        
        // Add AMD and Intel support here
        self.get_fallback_info(&mut gpus).await?;
        
        Ok(gpus)
    }

    #[cfg(feature = "nvidia")]
    fn get_nvidia_info(&self, nvml: &nvml_wrapper::Nvml) -> Result<Vec<GpuInfo>> {
        let mut gpus = Vec::new();
        let device_count = nvml.device_count()?;
        
        for i in 0..device_count {
            let device = nvml.device_by_index(i)?;
            let name = device.name()?;
            let memory_info = device.memory_info()?;
            let utilization = device.utilization_rates()?.gpu;
            let temperature = device.temperature(nvml_wrapper::enum_wrappers::device::TemperatureSensor::Gpu)?;
            
            let fan_speed = device.fan_speed(0).ok().map(|f| f as f32);
            let power_draw = device.power_usage().ok().map(|p| p as f32 / 1000.0);
            let gpu_clock = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Graphics).ok();
            let memory_clock = device.clock_info(nvml_wrapper::enum_wrappers::device::Clock::Memory).ok();
            
            gpus.push(GpuInfo {
                index: i as usize,
                name,
                driver_version: nvml.sys_driver_version()?,
                cuda_version: nvml.sys_cuda_driver_version().ok().map(|v| format!("{}.{}", v / 1000, (v % 1000) / 10)),
                utilization: utilization as f32,
                memory_used: memory_info.used,
                memory_total: memory_info.total,
                temperature: temperature as f32,
                fan_speed,
                power_draw,
                gpu_clock,
                memory_clock,
                throttled: false, // TODO: Implement throttling detection
                vendor: GpuVendor::Nvidia,
            });
        }
        
        Ok(gpus)
    }

    async fn get_fallback_info(&self, gpus: &mut Vec<GpuInfo>) -> Result<()> {
        // If no GPUs detected, add a mock GPU for demonstration
        if gpus.is_empty() {
            gpus.push(GpuInfo {
                index: 0,
                name: "Mock GPU".to_string(),
                driver_version: "1.0.0".to_string(),
                cuda_version: None,
                utilization: 45.0,
                memory_used: 2048 * 1024 * 1024, // 2GB
                memory_total: 8192 * 1024 * 1024, // 8GB
                temperature: 65.0,
                fan_speed: Some(60.0),
                power_draw: Some(150.0),
                gpu_clock: Some(1500),
                memory_clock: Some(7000),
                throttled: false,
                vendor: GpuVendor::Unknown,
            });
        }
        
        Ok(())
    }
}

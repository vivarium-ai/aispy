use anyhow::{Context, Result};
use nvml_wrapper::Nvml;

pub struct NvmlCollector {
    nvml: Nvml,
}

impl NvmlCollector {
    pub fn new() -> Result<Self> {
        let nvml = Nvml::init().context("failed to initialize NVML")?;

        Ok(Self { nvml })
    }

    pub fn device_count(&self) -> Result<u32> {
        self.nvml
            .device_count()
            .context("failed to query NVML device count")
    }
}

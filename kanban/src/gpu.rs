use crate::arg::*;
use crate::method::compile::*;
use std::io::prelude::*;

impl CompileTopMessage for GpuArg {
    fn messages(&self) -> Vec<String> {
        vec![self.message.clone()]
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn run(self) {
        log::info!("GPU checking...");

        if let Err(e) = pollster::block_on(self.check_gpu()) {
            log::error!("{}", e);
        }

        self.mkdir(self.dir_name());

        self.create_cargotoml();

        self.create_gpu_mainfile();

        self.create_shaderwgsl();

        self.create_idfile();

        let cwd = self.record_current_dir();

        self.cd(self.dir_name());

        log::info!("Compiling...");

        self.compile_with_cargo();

        log::info!("Compile done!");

        self.cd("./target/debug/");

        self.execute(".", &self.message);

        self.cd(&cwd);

        self.rmdir();
    }
}

impl GpuArg {
    pub fn create_cargotoml(&self) {
        let template = include_str!("template/gpu/Cargo.toml");
        let filled_template = template.replace("{ name }", &self.message);
        let output_path = format!("{}/Cargo.toml", self.dir_name());
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(filled_template.as_bytes()).unwrap();
    }

    pub fn create_gpu_mainfile(&self) {
        let template = include_str!("template/gpu/main.rs");
        let filled_template = template.replace("{ time }", &self.time.to_string());
        let output_path = format!("{}/main.rs", self.dir_name());
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(filled_template.as_bytes()).unwrap();
    }

    pub fn create_shaderwgsl(&self) {
        let template = include_str!("template/gpu/shader.wgsl");
        let output_path = format!("{}/shader.wgsl", self.dir_name());
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(template.as_bytes()).unwrap();
    }

    pub fn compile_with_cargo(&self) {
        std::process::Command::new("cargo")
            .arg("build")
            .output()
            .expect("failed to cargo build");
    }

    pub async fn check_gpu(&self) -> Result<(), anyhow::Error> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .ok_or_else(|| anyhow::anyhow!("Failed to find an appropriate GPU adapter"))?;
        let (_device, _queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device and Queue"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create device and queue: {}", e))?;

        Ok(())
    }
}

use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;
use std::io::prelude::*;
use std::thread::Builder;

impl CommonTopMessage for GpuArg {
    fn messages(&self) -> Vec<String> {
        vec![self.message.clone()]
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn method(&self) -> Method {
        self.method
    }

    fn thread(&self) -> usize {
        self.thread
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for GpuArg {
    fn run_by_compile(self) {
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

impl ProcnameTopMessage for GpuArg {
    fn run_by_procname(self) {
        let message = self.message.clone();
        let time = self.time;

        let builder = Builder::new().name(message);

        let handle = builder
            .spawn(move || {
                pollster::block_on(async move {
                    let start = std::time::Instant::now();
                    loop {
                        if start.elapsed().as_secs() >= time as u64 {
                            break;
                        }

                        // We attempt to replicate the logic from the template.
                        // However, we handle errors gracefully instead of unwrapping blindly,
                        // to avoid crashing the main process if GPU is unavailable or busy.
                        match GpuState::new().await {
                            Ok(state) => state.compute(),
                            Err(_) => {
                                // If we can't access GPU, we might fall back to CPU spin
                                // or just log and retry.
                                // Since run_by_compile would fail if GPU check fails,
                                // we can assume it might work, or we should just yield.
                                // We'll log once if possible but in a tight loop it's bad.
                                // Let's just yield.
                                std::thread::yield_now();
                            }
                        }
                    }
                });
            })
            .unwrap();

        handle.join().unwrap();
    }
}

struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
}

impl GpuState {
    async fn new() -> Result<Self, anyhow::Error> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|_| anyhow::anyhow!("Failed to find an appropriate GPU adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device and Queue"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create device and queue: {}", e))?;

        let shader_source = include_str!("template/gpu/shader.wgsl");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            pipeline,
        })
    }

    fn compute(&self) {
        let mut command_encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Command Encoder"),
                });

        {
            let mut compute_pass =
                command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("Compute Pass"),
                    timestamp_writes: None,
                });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }
        self.queue.submit(Some(command_encoder.finish()));
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
            .map_err(|_| anyhow::anyhow!("Failed to find an appropriate GPU adapter"))?;
        let (_device, _queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device and Queue"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::Performance,
                    trace: wgpu::Trace::Off,
                    experimental_features: wgpu::ExperimentalFeatures::disabled(),
                },
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create device and queue: {}", e))?;

        Ok(())
    }
}

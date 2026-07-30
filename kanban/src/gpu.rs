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

                        // Keep the thread alive and busy even when the GPU is
                        // unavailable: the point of this mode is the thread name
                        // showing up in top, not the compute work itself.
                        match GpuState::new().await {
                            Ok(state) => state.compute(),
                            Err(_) => std::thread::yield_now(),
                        }
                    }
                });
            })
            .unwrap();

        handle.join().unwrap();
    }
}

/// Everything needed to keep a GPU busy: a device, its queue and the compute
/// pipeline built from `template/gpu/shader.wgsl`.
///
/// `raw_gpu` shares this type, so the workspace holds a single copy of the wgpu
/// setup sequence. The generated crate under `template/gpu/` necessarily keeps
/// its own copy, since it is built standalone in a temporary directory.
pub(crate) struct GpuState {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    pub(crate) adapter_info: wgpu::AdapterInfo,
}

impl GpuState {
    pub(crate) async fn new() -> Result<Self, anyhow::Error> {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to find an appropriate GPU adapter: {}", e))?;
        let adapter_info = adapter.get_info();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Device and Queue"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::default(),
                trace: wgpu::Trace::Off,
            })
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
            immediate_size: 0,
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        Ok(Self {
            device,
            queue,
            pipeline,
            adapter_info,
        })
    }

    pub(crate) fn compute(&self) {
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
        run_checked(
            std::process::Command::new("cargo").arg("build"),
            "cargo build",
        );
    }

    /// Probe the GPU before doing any work on disk.
    ///
    /// Building the whole `GpuState` (rather than only asking for a device)
    /// means the shader and pipeline are validated here too, so a shader the
    /// driver rejects is reported up front instead of from inside the busy loop.
    /// The adapter is logged because wgpu happily falls back to a software
    /// rasterizer such as llvmpipe, which looks like success but never shows up
    /// in nvtop.
    pub async fn check_gpu(&self) -> Result<(), anyhow::Error> {
        let state = GpuState::new().await?;
        let info = &state.adapter_info;
        log::info!(
            "GPU adapter: {} ({:?}, backend {:?})",
            info.name,
            info.device_type,
            info.backend
        );
        Ok(())
    }
}

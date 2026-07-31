use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;
use std::thread::Builder;

impl CommonTopMessage for GpuArg {
    fn messages(&self) -> Vec<String> {
        kanban_core::single(&self.message)
    }

    fn dir_name(&self) -> &str {
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
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

        let _guard = self.temp_dir();

        self.create_cargotoml();

        self.create_gpu_mainfile();

        self.create_shaderwgsl();

        log::info!("Compiling...");

        self.compile_with_cargo();

        log::info!("Compile done!");

        let bin_dir = format!("{}/target/debug", self.dir_name());
        self.execute(&bin_dir, &self.message, self.thread(), self.time());
    }
}

impl ProcnameTopMessage for GpuArg {
    fn run_by_procname(self) {
        let message = self.message.clone();
        let time = self.time;

        let builder = Builder::new().name(fit_thread_name(&message));

        let handle = builder
            .spawn(move || {
                pollster::block_on(async move {
                    // Build the device once. Doing it per iteration meant
                    // creating and dropping a whole device for every dispatch,
                    // which dominated the actual compute work.
                    let state = GpuState::new().await;
                    if let Err(e) = &state {
                        log::error!("{}", e);
                    }

                    let start = std::time::Instant::now();
                    while start.elapsed().as_secs() < time as u64 {
                        // Keep the thread alive and busy even without a GPU:
                        // this mode is about the thread name showing up in top,
                        // not about the compute work itself.
                        match &state {
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
    bind_group: wgpu::BindGroup,
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

        // The shader writes its result here. Nothing ever reads the buffer back;
        // it exists so the arithmetic has an observable effect and survives
        // optimisation.
        let sink = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Sink Buffer"),
            size: std::mem::size_of::<f32>() as u64,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sink.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Pipeline Layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
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
            bind_group,
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
            compute_pass.set_bind_group(0, Some(&self.bind_group), &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }
        self.queue.submit(Some(command_encoder.finish()));
    }
}

impl GpuArg {
    pub fn create_cargotoml(&self) {
        // The binary name is what nvtop shows, so it becomes the message.
        let template =
            include_str!("template/gpu/Cargo.toml").replace("kanban_gpu_template", &self.message);
        write_generated(&format!("{}/Cargo.toml", self.dir_name()), &template);
    }

    pub fn create_gpu_mainfile(&self) {
        let template = include_str!("template/gpu/main.rs");
        write_generated(&format!("{}/main.rs", self.dir_name()), template);
    }

    pub fn create_shaderwgsl(&self) {
        let template = include_str!("template/gpu/shader.wgsl");
        write_generated(&format!("{}/shader.wgsl", self.dir_name()), template);
    }

    pub fn compile_with_cargo(&self) {
        // current_dir rather than chdir'ing the process: cargo needs to see the
        // generated manifest, but nothing else here should have to care where
        // the working directory happens to be.
        run_checked(
            std::process::Command::new("cargo")
                .arg("build")
                .current_dir(self.dir_name()),
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

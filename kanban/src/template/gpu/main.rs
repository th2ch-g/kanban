// The program kanban compiles for `gpu --method compile`. Its *filename* is the
// message, which is what nvtop displays; the body keeps the GPU busy.
//
// The duration arrives through the environment rather than being pasted in as
// text before compiling, which is what used to make this file invalid Rust and
// therefore invisible to cargo, clippy and rustfmt.

fn main() {
    pollster::block_on(run());
}

async fn run() {
    let time: u64 = std::env::var("KANBAN_TIME")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    let state = State::new().await;
    let start = std::time::Instant::now();
    while start.elapsed().as_secs() < time {
        state.compute();
    }
}

struct State {
    device: wgpu::Device,
    queue: wgpu::Queue,
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
}

impl State {
    async fn new() -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
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
            .unwrap();

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader Module"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

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

        Self {
            device,
            queue,
            pipeline,
            bind_group,
        }
    }

    fn compute(&self) {
        let mut command_encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Command Encoder"),
            });

        {
            let mut compute_pass = command_encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
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

use crate::arg::*;
use crate::gpu::GpuState;

impl RawGpuArg {
    pub fn run(&self) {
        pollster::block_on(self.core());
    }

    async fn core(&self) {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed().as_secs() >= (self.time as u64) {
                break;
            }
            let state = GpuState::new().await.unwrap();
            state.compute();
        }
    }
}

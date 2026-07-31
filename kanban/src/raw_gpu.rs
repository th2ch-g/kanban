use crate::arg::*;
use crate::gpu::GpuState;

impl RawGpuArg {
    pub fn run(&self) {
        pollster::block_on(self.core());
    }

    async fn core(&self) {
        // One device for the whole run: rebuilding it per dispatch cost far more
        // than the dispatch itself.
        let state = GpuState::new().await.unwrap();

        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < (self.time as u64) {
            state.compute();
        }
    }
}

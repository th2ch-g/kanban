use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

/// How long each frame stays on screen. Short enough to read as motion.
const SECONDS_PER_FRAME: usize = 2;

impl CommonTopMessage for WaveArg {
    fn messages(&self) -> Vec<String> {
        kanban_core::wave(&self.message, self.length)
    }

    fn dir_name(&self) -> &str {
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
    }

    /// Threads per frame, as asked for on the command line.
    fn thread(&self) -> usize {
        self.thread
    }

    /// Seconds per frame. `WaveArg` has no -t at all - the help says the run
    /// time is determined automatically, and it is: frames x this constant.
    fn time(&self) -> usize {
        SECONDS_PER_FRAME
    }
}

impl CompileTopMessage for WaveArg {
    fn run_by_compile(self) {
        // This used to be a near-verbatim copy of template_run. The only real
        // difference is that the frames run one at a time, which is what makes
        // the message appear to scroll.
        let threads = self.thread();
        self.clone().template_run(
            self.time(),
            ThreadPlan::Uniform(threads),
            ExecutionOrder::Sequential,
        );
    }
}

impl ProcnameTopMessage for WaveArg {}

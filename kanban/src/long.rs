use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for LongArg {
    fn messages(&self) -> Vec<String> {
        kanban_core::long(&self.message, self.length)
    }

    fn dir_name(&self) -> &str {
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
    }

    /// One process per row, not the user's thread count: every chunk has to be
    /// on screen for the message to read as a whole.
    fn thread(&self) -> usize {
        self.messages().len()
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for LongArg {
    fn run_by_compile(self) {
        self.clone()
            .template_run(self.time, ThreadPlan::Decreasing, ExecutionOrder::Parallel)
    }
}

impl ProcnameTopMessage for LongArg {}

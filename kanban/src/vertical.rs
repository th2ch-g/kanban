use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for VerticalArg {
    fn messages(&self) -> Vec<String> {
        kanban_core::vertical(&self.message)
    }

    fn dir_name(&self) -> &str {
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
    }

    /// One process per row of the transposition.
    fn thread(&self) -> usize {
        self.messages().len()
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for VerticalArg {
    fn run_by_compile(self) {
        self.clone()
            .template_run(self.time, ThreadPlan::Decreasing, ExecutionOrder::Parallel);
    }
}

impl ProcnameTopMessage for VerticalArg {}

use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for Multiple2Arg {
    fn messages(&self) -> Vec<String> {
        kanban_core::multiple2(&self.message)
    }

    fn dir_name(&self) -> &str {
        &self.common.dir_name
    }

    fn method(&self) -> Method {
        self.common.method
    }

    fn thread(&self) -> usize {
        self.message.len()
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for Multiple2Arg {
    fn run_by_compile(self) {
        self.clone()
            .template_run(self.time, ThreadPlan::One, ExecutionOrder::Parallel);
    }
}

impl ProcnameTopMessage for Multiple2Arg {}

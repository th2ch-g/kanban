use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for Multiple2Arg {
    fn messages(&self) -> Vec<String> {
        self.message.clone()
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn method(&self) -> Method {
        self.method
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
        self.clone().template_run(self.time, true);
    }
}

impl ProcnameTopMessage for Multiple2Arg {}

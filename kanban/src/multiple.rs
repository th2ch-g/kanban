use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for MultipleArg {
    fn messages(&self) -> Vec<String> {
        vec![self.message.clone(); self.thread]
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

impl CompileTopMessage for MultipleArg {
    fn run_by_compile(self) {
        self.clone().template_run(self.time, true);
    }
}

impl ProcnameTopMessage for MultipleArg {}

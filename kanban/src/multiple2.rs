use crate::arg::*;
use crate::method::compile::*;

impl CompileTopMessage for Multiple2Arg {
    fn messages(&self) -> Vec<String> {
        self.message.clone()
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn run(self) {
        self.clone().template_run(self.time, true);
    }
}

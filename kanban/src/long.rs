use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;

impl CommonTopMessage for LongArg {
    fn messages(&self) -> Vec<String> {
        if self.message.len() <= self.length {
            return vec![self.message.to_string()];
        }

        self.message
            .as_bytes()
            .chunks(self.length)
            .map(|chunk| String::from_utf8_lossy(chunk).to_string())
            .collect::<Vec<String>>()
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn method(&self) -> Method {
        self.method
    }

    fn thread(&self) -> usize {
        // For LongArg, we want to display all chunks.
        // Similar to Multiple2Arg, thread count should be number of chunks.
        self.messages().len()
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for LongArg {
    fn run_by_compile(self) {
        self.clone().template_run(self.time, false)
    }
}

impl ProcnameTopMessage for LongArg {}

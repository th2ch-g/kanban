use crate::arg::*;
use crate::method::compile::*;

impl CompileTopMessage for LongArg {
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

    fn run(self) {
        self.clone().template_run(self.time, false)
    }
}

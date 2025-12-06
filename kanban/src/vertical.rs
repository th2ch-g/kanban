use crate::arg::*;
use crate::method::compile::*;
use crate::method::procname::*;
use crate::method::*;
use itertools::Itertools;

impl CommonTopMessage for VerticalArg {
    fn messages(&self) -> Vec<String> {
        let maxlen = self.message.iter().map(|s| s.len()).max().unwrap_or(0);
        let mut result = vec![String::new(); maxlen];

        for s in self
            .message
            .iter()
            .cloned()
            .sorted_by_key(|s| std::cmp::Reverse(s.len()))
        {
            for (i, c) in s.chars().enumerate() {
                result[i].push(c);
            }
            for res in result.iter_mut().take(maxlen).skip(s.len()) {
                res.push(' ');
            }
        }
        result
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn method(&self) -> Method {
        self.method
    }

    fn thread(&self) -> usize {
        let maxlen = self.message.iter().map(|s| s.len()).max().unwrap_or(0);
        maxlen
    }

    fn time(&self) -> usize {
        self.time
    }
}

impl CompileTopMessage for VerticalArg {
    fn run_by_compile(self) {
        self.clone().template_run(self.time, false);
    }
}

impl ProcnameTopMessage for VerticalArg {}

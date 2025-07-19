use crate::arg::*;
use crate::method::compile::*;
use itertools::Itertools;

impl CompileTopMessage for VerticalArg {
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

    fn run(self) {
        self.clone().template_run(self.time, false);
    }
}

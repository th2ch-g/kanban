use crate::arg::*;
use crate::method::compile::*;

impl CompileTopMessage for SingleArg {
    fn messages(&self) -> Vec<String> {
        vec![self.message.clone()]
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn run(self) {
        self.mkdir(self.dir_name());

        self.create_mainfile(self.dir_name(), self.thread, self.time);

        self.create_idfile();

        self.compile(self.dir_name(), &self.message);

        let current_dir = self.record_current_dir();

        self.cd(&self.dir_name);

        self.execute(".", &self.message);

        self.cd(&current_dir);

        self.rmdir();
    }
}

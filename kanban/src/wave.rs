use crate::arg::*;
use crate::method::compile::*;

impl CompileTopMessage for WaveArg {
    fn messages(&self) -> Vec<String> {
        let msg_len = self.message.len();
        let mut message_list = Vec::new();

        if msg_len < self.length {
            for i in 0..msg_len {
                let tmp = format!(
                    "{}{}{}",
                    &self.message[i..],
                    " ".repeat(self.length - msg_len),
                    &self.message[..i]
                );
                message_list.push(tmp);
            }

            for i in 0..(self.length - msg_len) {
                let tmp = format!(
                    "{}{}{}",
                    " ".repeat(self.length - msg_len - i),
                    self.message,
                    " ".repeat(i)
                );
                message_list.push(tmp);
            }
        } else {
            for i in 0..=msg_len {
                let tmp = format!("{} {}", &self.message[i..], &self.message[..i]);
                message_list.push(tmp[..self.length.min(tmp.len())].to_string());
            }
        }
        message_list
    }

    fn dir_name(&self) -> &str {
        &self.dir_name
    }

    fn run(self)
    where
        Self: Sync + Send,
    {
        let dir_name_t = std::sync::Arc::new(self.dir_name().to_string().clone());
        let message_list_t = std::sync::Arc::new(self.messages().clone());
        let self_t = std::sync::Arc::new(self.clone());

        self_t.mkdir(self_t.dir_name());
        self_t.mkdir(&format!("{}/{}", self_t.dir_name(), "run"));

        self_t.create_idfile();

        let mut thrs = Vec::new();

        for i in 0..self_t.messages().len() {
            let dir_name_r = std::sync::Arc::clone(&dir_name_t);
            let message_list_r = std::sync::Arc::clone(&message_list_t);
            let self_r = std::sync::Arc::clone(&self_t);

            thrs.push(std::thread::spawn(move || {
                self_r.mkdir(&format!("{}/{}", dir_name_r, i));
                self_r.create_mainfile(&format!("{}/{}", dir_name_r, i), self.thread, 2);
                self_r.compile_with_subdir(&dir_name_r, &i.to_string(), &message_list_r[i]);
            }));
        }

        thrs.into_iter().for_each(|h| h.join().unwrap());

        let current_dir = self_t.record_current_dir();
        self_t.cd(&format!("{}/{}", self_t.dir_name(), "run"));

        for message in self.messages() {
            self_t.execute(".", &message);
        }

        self_t.cd(&current_dir);

        self_t.rmdir();
    }
}

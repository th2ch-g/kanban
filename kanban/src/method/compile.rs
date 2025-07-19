use std::io::prelude::*;

pub trait CompileTopMessage
where
    Self: 'static,
{
    fn run(self); // due to parallel process
    fn messages(&self) -> Vec<String>;
    fn dir_name(&self) -> &str;

    fn compile(&self, dir_name: &str, message: &str) {
        std::process::Command::new("rustc")
            .arg(format!("{}/{}", dir_name, "ms.rs"))
            .arg("-o")
            .arg(format!("{}/{}", dir_name, message))
            .output()
            .expect("failed to compile");
    }

    fn compile_with_subdir(&self, dir_name: &str, subdir: &str, message: &str) {
        std::process::Command::new("rustc")
            .arg(format!("{}/{}/{}", dir_name, subdir, "ms.rs"))
            .arg("-o")
            .arg(format!("{}/{}/{}", dir_name, "run", message))
            .output()
            .expect("failed to compile2");
    }

    fn record_current_dir(&self) -> String {
        let current_dir = std::path::PathBuf::from("./");
        let current_dir = std::fs::canonicalize(current_dir);
        match current_dir {
            Ok(s) => s.to_string_lossy().to_string(),
            Err(_) => {
                log::error!("failed to record current directory");
                String::from("err")
            }
        }
    }

    fn cd(&self, dir_name: &str) {
        let cd_result = std::env::set_current_dir(dir_name);
        match cd_result {
            Ok(_) => (),
            Err(_) => {
                log::error!("failed to cd");
            }
        }
    }

    fn execute(&self, dir_name: &str, message: &str) {
        std::process::Command::new(format!("{}/{}", dir_name, message))
            .output()
            .expect("failed to run");
    }

    fn rmdir(&self) {
        let dir_path = std::path::Path::new(self.dir_name());
        let idfile_path = dir_path.join("kanban.idfile");

        if idfile_path.exists() && std::fs::remove_dir_all(self.dir_name()).is_err() {
            log::warn!("failed to rmdir but continue");
        }
    }

    fn mkdir(&self, dir_name: &str) {
        let mkdir_result = std::fs::create_dir(dir_name);
        match mkdir_result {
            Ok(_) => (),
            Err(_) => {
                log::error!("failed to create directory");
                log::error!("check authority");
                std::process::exit(1);
            }
        }
    }

    fn create_idfile(&self) {
        let template = include_str!("../template/kanban.idfile");
        let output_path = format!("{}/kanban.idfile", self.dir_name());
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(template.as_bytes()).unwrap();
    }

    fn create_mainfile(&self, dir_name: &str, thread: usize, time: usize) {
        let template = include_str!("../template/ms.rs");
        let filled_template = template
            .replace("{ thread }", &thread.to_string())
            .replace("{ time }", &time.to_string());
        let output_path = format!("{}/ms.rs", dir_name);
        let mut output_file = std::fs::File::create(&output_path).unwrap();
        output_file.write_all(filled_template.as_bytes()).unwrap();
    }

    fn template_run(self, time: usize, single_bool: bool)
    where
        Self: Sync + Send + Sized,
    {
        let dir_name_t = std::sync::Arc::new(self.dir_name().to_string().clone());
        let time_t = std::sync::Arc::new(time);
        let message_list_t = std::sync::Arc::new(self.messages().to_owned());
        let self_t = std::sync::Arc::new(self);

        self_t.mkdir(self_t.dir_name());
        self_t.mkdir(&format!("{}/{}", self_t.dir_name(), "run"));

        self_t.create_idfile();

        let mut thrs = Vec::new();
        let mut count = self_t.messages().len();
        for i in 0..self_t.messages().len() {
            let dir_name_r = std::sync::Arc::clone(&dir_name_t);
            let time_r = std::sync::Arc::clone(&time_t);
            let message_list_r = std::sync::Arc::clone(&message_list_t);
            let self_r = std::sync::Arc::clone(&self_t);
            if single_bool {
                thrs.push(std::thread::spawn(move || {
                    self_r.mkdir(&format!("{}/{}", dir_name_r, i));
                    self_r.create_mainfile(&format!("{}/{}", dir_name_r, i), 1, *time_r);
                    self_r.compile_with_subdir(&dir_name_r, &i.to_string(), &message_list_r[i]);
                }));
            } else {
                thrs.push(std::thread::spawn(move || {
                    self_r.mkdir(&format!("{}/{}", dir_name_r, i));
                    self_r.create_mainfile(&format!("{}/{}", dir_name_r, i), count, *time_r);
                    self_r.compile_with_subdir(&dir_name_r, &i.to_string(), &message_list_r[i]);
                }));
            }
            count -= 1;
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());

        let current_dir = self_t.record_current_dir();
        self_t.cd(&format!("{}/{}", self_t.dir_name(), "run"));

        let mut thrs = Vec::new();
        for i in 0..self_t.messages().len() {
            let message_list_r = std::sync::Arc::clone(&message_list_t);
            let self_r = std::sync::Arc::clone(&self_t);
            thrs.push(std::thread::spawn(move || {
                self_r.execute(".", &message_list_r[i]);
            }));
        }
        thrs.into_iter().for_each(|h| h.join().unwrap());

        self_t.cd(&current_dir);

        self_t.rmdir();
    }
}

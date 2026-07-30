pub mod compile;
pub mod copy;
pub mod procname;

use clap::ValueEnum;

/// Run an external command and fail loudly when it does not succeed.
///
/// `Command::output()` only reports whether the process could be *spawned*, so a
/// compiler exiting non-zero used to slip through unnoticed and surface later as
/// an unrelated "failed to run" panic, with its diagnostics discarded. Checking
/// the exit status here and echoing stderr keeps the failure at its origin.
pub fn run_checked(cmd: &mut std::process::Command, what: &str) {
    let output = match cmd.output() {
        Ok(output) => output,
        Err(e) => panic!("failed to spawn {}: {}", what, e),
    };

    if !output.status.success() {
        log::error!("{} failed with {}", what, output.status);
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.trim().is_empty() {
            log::error!("{}", stderr.trim_end());
        }
        panic!("{} failed", what);
    }
}

pub trait CommonTopMessage
where
    Self: 'static,
{
    fn method(&self) -> Method;
    fn messages(&self) -> Vec<String>;
    fn dir_name(&self) -> &str;
    fn thread(&self) -> usize;
    fn time(&self) -> usize;
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Method {
    Compile,
    Procname,
    Copy,
}

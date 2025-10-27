pub mod compile;
pub mod copy;
pub mod procname;

use clap::ValueEnum;

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

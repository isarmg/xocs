extern crate blog_cli_arguments as clap;
use blog_cli_arguments::Args;
use blog_cli_arguments::builder::TypedValueParser as _;

use crate::foreign_crate;

#[derive(Args, Debug)]
pub(crate) struct BuiltInParsers {
    /// Support for discrete numbers
    #[arg(
        long,
        default_value_t = 22,
        value_parser = blog_cli_arguments::builder::PossibleValuesParser::new(["22", "80"])
            .map(|s| s.parse::<usize>().unwrap()),
    )]
    port: usize,

    /// Support enums from a foreign crate that don't implement `ValueEnum`
    #[arg(
        long,
        default_value_t = foreign_crate::LogLevel::Info,
        value_parser = blog_cli_arguments::builder::PossibleValuesParser::new(["trace", "debug", "info", "warn", "error"])
            .map(|s| s.parse::<foreign_crate::LogLevel>().unwrap()),
    )]
    log_level: foreign_crate::LogLevel,
}

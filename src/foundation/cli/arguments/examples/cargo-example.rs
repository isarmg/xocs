fn main() {
    let cmd = blog_cli_arguments::Command::new("cargo")
        .bin_name("cargo")
        .styles(COMMAND_LINE_STYLING)
        .subcommand_required(true)
        .subcommand(
            blog_cli_arguments::command!("example").arg(
                blog_cli_arguments::arg!(--"manifest-path" <PATH>)
                    .value_parser(blog_cli_arguments::value_parser!(std::path::PathBuf)),
            ),
        );
    let matches = cmd.get_matches();
    let matches = match matches.subcommand() {
        Some(("example", matches)) => matches,
        _ => unreachable!("clap should ensure we don't get here"),
    };
    let manifest_path = matches.get_one::<std::path::PathBuf>("manifest-path");
    println!("{manifest_path:?}");
}

// See also `command_line_cargo::style::COMMAND_LINE_STYLING`
pub const COMMAND_LINE_STYLING: blog_cli_arguments::builder::styling::Styles = blog_cli_arguments::builder::styling::Styles::styled()
    .header(command_line_cargo::style::HEADER)
    .usage(command_line_cargo::style::USAGE)
    .literal(command_line_cargo::style::LITERAL)
    .placeholder(command_line_cargo::style::PLACEHOLDER)
    .error(command_line_cargo::style::ERROR)
    .valid(command_line_cargo::style::VALID)
    .invalid(command_line_cargo::style::INVALID);

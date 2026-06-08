use clap::{Arg, Command};
use ssg::{formatted_text::expand_math_markdown, version};
use std::{fs, path::PathBuf};

struct CliArgs {
    input: PathBuf,
    math_shorthand: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let markdown = fs::read_to_string(&args.input)?;
    print!("{}", expand_math_markdown(&markdown, args.math_shorthand));
    Ok(())
}

fn parse_args() -> Result<CliArgs, Box<dyn std::error::Error>> {
    let matches = cli_command().get_matches();
    let input = matches
        .get_one::<PathBuf>("input")
        .cloned()
        .ok_or("Missing input path")?;
    let math_shorthand = matches.get_flag("math-shorthand");

    Ok(CliArgs {
        input,
        math_shorthand,
    })
}

fn cli_command() -> Command {
    Command::new("ssg-math-expand")
        .version(version::VERSION)
        .author("Hadi Moshayedi")
        .about("Expands SSG math shorthand and :::math blocks before HTML rendering")
        .arg(
            Arg::new("math-shorthand")
                .long("math-shorthand")
                .help("Enable shorthand by default unless the file opts out")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("input")
                .help("Markdown file to expand")
                .required(true)
                .index(1)
                .value_parser(clap::value_parser!(PathBuf)),
        )
}

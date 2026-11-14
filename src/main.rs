use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn usage() -> &'static str {
    "cwctl - Console Management Plugin for Class Widgets 2

USAGE:
    cwctl [OPTIONS]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
"
}

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let mut output = String::new();

    for argument in args {
        match argument.as_str() {
            "-h" | "--help" => output.push_str(usage()),
            "-V" | "--version" => output.push_str(&format!("cwctl {VERSION}\n")),
            value if value.starts_with('-') => {
                return Err(format!("error: unknown option '{value}'\n\n{}", usage()));
            }
            command => {
                return Err(format!(
                    "error: command '{command}' is not implemented yet\n\n{}",
                    usage()
                ));
            }
        }
    }

    if output.is_empty() {
        output.push_str(usage());
    }

    Ok(output)
}

fn main() {
    match run(env::args().skip(1)) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprint!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    fn arguments(arguments: &[&str]) -> Vec<String> {
        arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect()
    }

    #[test]
    fn prints_help_without_arguments() {
        let output = run(arguments(&[])).expect("help should be returned");

        assert!(output.contains("USAGE:"));
        assert!(output.contains("--version"));
    }

    #[test]
    fn prints_version() {
        let output = run(arguments(&["--version"])).expect("version should be returned");

        assert!(output.starts_with("cwctl "));
    }

    #[test]
    fn rejects_unknown_options() {
        let error = run(arguments(&["--unknown"])).expect_err("unknown options should fail");

        assert!(error.contains("unknown option"));
    }
}

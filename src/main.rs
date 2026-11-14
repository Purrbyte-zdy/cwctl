use std::env;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, PartialEq, Eq)]
enum Command {
    Plugin(PluginCommand),
}

#[derive(Debug, PartialEq, Eq)]
enum PluginCommand {
    Install {
        name: String,
        version: Option<String>,
        url: Option<String>,
    },
    List,
    Update {
        name: String,
    },
    Uninstall {
        name: String,
    },
    Info {
        name: String,
    },
}

fn usage() -> &'static str {
    "cwctl - Console Management Plugin for Class Widgets 2

USAGE:
    cwctl [OPTIONS]
    cwctl plugin <COMMAND>

COMMANDS:
    plugin install <name> [--version <version>] [--url <source>]
    plugin list
    plugin update <name>
    plugin uninstall <name>
    plugin info <name>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information
"
}

fn parse_command(arguments: &[String]) -> Result<Command, String> {
    if arguments.first().map(String::as_str) != Some("plugin") {
        return Err(format!("error: expected 'plugin' command\n\n{}", usage()));
    }

    let subcommand = arguments
        .get(1)
        .ok_or_else(|| format!("error: missing plugin command\n\n{}", usage()))?;

    match subcommand.as_str() {
        "install" => parse_install(&arguments[2..]),
        "list" => require_no_arguments("plugin list", &arguments[2..])
            .map(|_| Command::Plugin(PluginCommand::List)),
        "update" => parse_named_command("update", &arguments[2..])
            .map(|name| Command::Plugin(PluginCommand::Update { name })),
        "uninstall" => parse_named_command("uninstall", &arguments[2..])
            .map(|name| Command::Plugin(PluginCommand::Uninstall { name })),
        "info" => parse_named_command("info", &arguments[2..])
            .map(|name| Command::Plugin(PluginCommand::Info { name })),
        value => Err(format!(
            "error: unknown plugin command '{value}'\n\n{}",
            usage()
        )),
    }
}

fn parse_install(arguments: &[String]) -> Result<Command, String> {
    let name = arguments
        .first()
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| "error: plugin install requires a name".to_owned())?;
    let mut version = None;
    let mut url = None;
    let mut index = 1;

    while index < arguments.len() {
        let option = &arguments[index];
        let value = arguments
            .get(index + 1)
            .filter(|value| !value.starts_with('-'));
        match option.as_str() {
            "--version" => version = Some(required_option_value(option, value)?),
            "--url" => url = Some(required_option_value(option, value)?),
            value => return Err(format!("error: unknown option '{value}'")),
        }
        index += 2;
    }

    Ok(Command::Plugin(PluginCommand::Install {
        name,
        version,
        url,
    }))
}

fn parse_named_command(command: &str, arguments: &[String]) -> Result<String, String> {
    let name = arguments
        .first()
        .filter(|value| !value.starts_with('-'))
        .cloned()
        .ok_or_else(|| format!("error: plugin {command} requires a name"))?;
    require_no_arguments(&format!("plugin {command}"), &arguments[1..])?;
    Ok(name)
}

fn require_no_arguments(command: &str, arguments: &[String]) -> Result<(), String> {
    if let Some(argument) = arguments.first() {
        return Err(format!(
            "error: unexpected argument '{argument}' for {command}"
        ));
    }
    Ok(())
}

fn required_option_value(option: &str, value: Option<&String>) -> Result<String, String> {
    value
        .cloned()
        .ok_or_else(|| format!("error: option '{option}' requires a value"))
}

fn run(args: impl IntoIterator<Item = String>) -> Result<String, String> {
    let arguments: Vec<String> = args.into_iter().collect();

    match arguments.as_slice() {
        [] => Ok(usage().to_owned()),
        [argument] if argument == "-h" || argument == "--help" => Ok(usage().to_owned()),
        [argument] if argument == "-V" || argument == "--version" => {
            Ok(format!("cwctl {VERSION}\n"))
        }
        _ => {
            parse_command(&arguments)?;
            Err("error: plugin management is not implemented yet".to_owned())
        }
    }
}

fn main() {
    match run(env::args().skip(1)) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_command, run, Command, PluginCommand};

    fn arguments(arguments: &[&str]) -> Vec<String> {
        arguments
            .iter()
            .map(|argument| (*argument).to_owned())
            .collect()
    }

    #[test]
    fn prints_help_without_arguments() {
        let output = run(arguments(&[])).expect("help should be returned");

        assert!(output.contains("plugin install"));
        assert!(output.contains("--version"));
    }

    #[test]
    fn prints_version() {
        let output = run(arguments(&["--version"])).expect("version should be returned");

        assert_eq!(output, "cwctl 27.0.0\n");
    }

    #[test]
    fn parses_install_options() {
        let command = parse_command(&arguments(&[
            "plugin",
            "install",
            "sample",
            "--version",
            "1.2.3",
            "--url",
            "https://example.test/sample",
        ]))
        .expect("install command should parse");

        assert_eq!(
            command,
            Command::Plugin(PluginCommand::Install {
                name: "sample".to_owned(),
                version: Some("1.2.3".to_owned()),
                url: Some("https://example.test/sample".to_owned()),
            })
        );
    }

    #[test]
    fn parses_named_plugin_commands() {
        let command = parse_command(&arguments(&["plugin", "info", "sample"]))
            .expect("info command should parse");

        assert_eq!(
            command,
            Command::Plugin(PluginCommand::Info {
                name: "sample".to_owned()
            })
        );
    }

    #[test]
    fn rejects_missing_install_option_value() {
        let error = parse_command(&arguments(&["plugin", "install", "sample", "--url"]))
            .expect_err("missing option value should fail");

        assert!(error.contains("requires a value"));
    }

    #[test]
    fn rejects_unknown_commands() {
        let error =
            run(arguments(&["plugin", "publish"])).expect_err("unknown commands should fail");

        assert!(error.contains("unknown plugin command"));
    }
}

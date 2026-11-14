use std::{env, fs, path::PathBuf};

mod aria2;
mod store;

use aria2::Aria2Client;
use reedline::{DefaultCompleter, DefaultPrompt, FileBackedHistory, Reedline, Signal};
use store::{Plugin, PluginStore};

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
        [argument] if argument == "shell" => {
            run_interactive()?;
            Ok(String::new())
        }
        _ => execute(parse_command(&arguments)?),
    }
}

fn run_interactive() -> Result<(), String> {
    let history_path = interactive_history_path()?;
    let history = FileBackedHistory::with_file(1_000, history_path)
        .map_err(|error| format!("error: unable to open interactive history: {error}"))?;
    let completer = DefaultCompleter::new(
        [
            "exit",
            "quit",
            "plugin",
            "plugin install",
            "plugin list",
            "plugin update",
            "plugin uninstall",
            "plugin info",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect(),
    );
    let mut line_editor = Reedline::create()
        .with_completer(Box::new(completer))
        .with_history(Box::new(history));
    let prompt = DefaultPrompt::default();

    while let Signal::Success(line) = line_editor
        .read_line(&prompt)
        .map_err(|error| format!("error: interactive console failed: {error}"))?
    {
        let arguments: Vec<String> = line.split_whitespace().map(ToOwned::to_owned).collect();
        if arguments.is_empty() {
            continue;
        }
        if matches!(arguments[0].as_str(), "exit" | "quit") {
            break;
        }
        match run(arguments) {
            Ok(output) => print!("{output}"),
            Err(error) => eprintln!("{error}"),
        }
    }

    Ok(())
}

fn interactive_history_path() -> Result<PathBuf, String> {
    let data_dir = if let Some(path) = env::var_os("CWCTL_HOME") {
        PathBuf::from(path)
    } else if let Some(path) = env::var_os("XDG_DATA_HOME") {
        PathBuf::from(path).join("cwctl")
    } else {
        let home = env::var_os("HOME")
            .ok_or_else(|| "error: unable to determine the user data directory".to_owned())?;
        PathBuf::from(home).join(".local/share/cwctl")
    };
    fs::create_dir_all(&data_dir)
        .map_err(|error| format!("error: unable to create history directory: {error}"))?;
    Ok(data_dir.join("history"))
}

fn execute(command: Command) -> Result<String, String> {
    match command {
        Command::Plugin(PluginCommand::Install { name, version, url }) => {
            let version = version.unwrap_or_else(|| "latest".to_owned());
            validate_plugin_metadata(&name, &version, url.as_deref())?;
            let store = PluginStore::from_environment()?;
            let download_id = if let Some(source) = &url {
                Some(Aria2Client::from_environment()?.add_uri(source)?)
            } else {
                None
            };
            store.install(Plugin {
                name: name.clone(),
                version: version.clone(),
                source: url,
                download_id,
            })?;
            Ok(format!("Installed plugin '{name}' ({version}).\n"))
        }
        Command::Plugin(PluginCommand::List) => {
            let store = PluginStore::from_environment()?;
            format_plugin_list(&store.list()?)
        }
        Command::Plugin(PluginCommand::Info { name }) => {
            let store = PluginStore::from_environment()?;
            let plugin = store
                .find(&name)?
                .ok_or_else(|| format!("error: plugin '{name}' is not installed"))?;
            Ok(format_plugin_info(&plugin))
        }
        Command::Plugin(PluginCommand::Update { name }) => {
            let store = PluginStore::from_environment()?;
            let plugin = store
                .update(&name, "latest".to_owned())?
                .ok_or_else(|| format!("error: plugin '{name}' is not installed"))?;
            Ok(format!(
                "Updated plugin '{}' to {}.\n",
                plugin.name, plugin.version
            ))
        }
        Command::Plugin(PluginCommand::Uninstall { name }) => {
            let store = PluginStore::from_environment()?;
            if !store.uninstall(&name)? {
                return Err(format!("error: plugin '{name}' is not installed"));
            }
            Ok(format!("Uninstalled plugin '{name}'.\n"))
        }
    }
}

fn validate_plugin_metadata(name: &str, version: &str, source: Option<&str>) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("error: plugin name cannot be empty".to_owned());
    }
    if name.contains('/') || name.contains('\\') {
        return Err("error: plugin name cannot contain path separators".to_owned());
    }
    if version.trim().is_empty() {
        return Err("error: plugin version cannot be empty".to_owned());
    }
    if let Some(source) = source {
        if !(source.starts_with("http://") || source.starts_with("https://")) {
            return Err("error: plugin source must be an HTTP(S) URL".to_owned());
        }
    }
    Ok(())
}

fn format_plugin_list(plugins: &[Plugin]) -> Result<String, String> {
    if plugins.is_empty() {
        return Ok("No plugins installed.\n".to_owned());
    }

    let mut output = String::new();
    for plugin in plugins {
        output.push_str(&format!("{} {}\n", plugin.name, plugin.version));
    }
    Ok(output)
}

fn format_plugin_info(plugin: &Plugin) -> String {
    let mut output = format!("Name: {}\nVersion: {}\n", plugin.name, plugin.version);
    if let Some(source) = &plugin.source {
        output.push_str(&format!("Source: {source}\n"));
    }
    if let Some(download_id) = &plugin.download_id {
        output.push_str(&format!("Download ID: {download_id}\n"));
    }
    output
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
    use super::{parse_command, run, Command, Plugin, PluginCommand};

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

    #[test]
    fn lists_no_plugins_when_manifest_is_missing() {
        let output = super::format_plugin_list(&[]).expect("empty plugin list should succeed");

        assert_eq!(output, "No plugins installed.\n");
    }

    #[test]
    fn formats_plugin_info() {
        let plugin = Plugin {
            name: "sample".to_owned(),
            version: "1.2.3".to_owned(),
            source: Some("https://example.test/sample".to_owned()),
            download_id: None,
        };

        assert_eq!(
            super::format_plugin_info(&plugin),
            "Name: sample\nVersion: 1.2.3\nSource: https://example.test/sample\n"
        );
    }

    #[test]
    fn validates_plugin_metadata() {
        assert!(super::validate_plugin_metadata(
            "sample",
            "1.2.3",
            Some("https://example.test/sample")
        )
        .is_ok());
        assert!(super::validate_plugin_metadata("sample/name", "1.2.3", None).is_err());
        assert!(super::validate_plugin_metadata("sample", "", None).is_err());
        assert!(
            super::validate_plugin_metadata("sample", "1.2.3", Some("file:///tmp/sample")).is_err()
        );
    }
}

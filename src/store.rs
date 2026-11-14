use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

const MANIFEST_FILE: &str = "plugins.json";

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Serialize)]
pub struct Plugin {
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
struct Manifest {
    plugins: Vec<Plugin>,
}

#[derive(Debug)]
pub struct PluginStore {
    manifest_path: PathBuf,
}

impl PluginStore {
    pub fn from_environment() -> Result<Self, String> {
        let data_dir = if let Some(path) = env::var_os("CWCTL_HOME") {
            PathBuf::from(path)
        } else if let Some(path) = env::var_os("XDG_DATA_HOME") {
            PathBuf::from(path).join("cwctl")
        } else {
            let home = env::var_os("HOME")
                .ok_or_else(|| "error: unable to determine the user data directory".to_owned())?;
            PathBuf::from(home).join(".local/share/cwctl")
        };

        Ok(Self::new(data_dir))
    }

    pub fn new(data_dir: impl AsRef<Path>) -> Self {
        Self {
            manifest_path: data_dir.as_ref().join(MANIFEST_FILE),
        }
    }

    pub fn list(&self) -> Result<Vec<Plugin>, String> {
        self.load().map(|manifest| manifest.plugins)
    }

    pub fn find(&self, name: &str) -> Result<Option<Plugin>, String> {
        Ok(self.list()?.into_iter().find(|plugin| plugin.name == name))
    }

    pub fn install(&self, plugin: Plugin) -> Result<(), String> {
        let mut manifest = self.load()?;
        manifest.plugins.retain(|item| item.name != plugin.name);
        manifest.plugins.push(plugin);
        manifest
            .plugins
            .sort_by(|left, right| left.name.cmp(&right.name));
        self.save(&manifest)
    }

    pub fn uninstall(&self, name: &str) -> Result<bool, String> {
        let mut manifest = self.load()?;
        let original_len = manifest.plugins.len();
        manifest.plugins.retain(|plugin| plugin.name != name);
        if manifest.plugins.len() == original_len {
            return Ok(false);
        }
        self.save(&manifest)?;
        Ok(true)
    }

    pub fn update(&self, name: &str, version: String) -> Result<Option<Plugin>, String> {
        let mut manifest = self.load()?;
        let Some(plugin) = manifest
            .plugins
            .iter_mut()
            .find(|plugin| plugin.name == name)
        else {
            return Ok(None);
        };
        plugin.version = version;
        let updated = plugin.clone();
        self.save(&manifest)?;
        Ok(Some(updated))
    }

    fn load(&self) -> Result<Manifest, String> {
        let content = match fs::read_to_string(&self.manifest_path) {
            Ok(content) => content,
            Err(error) if error.kind() == ErrorKind::NotFound => return Ok(Manifest::default()),
            Err(error) => return Err(format!("error: unable to read plugin manifest: {error}")),
        };

        serde_json::from_str(&content)
            .map_err(|error| format!("error: invalid plugin manifest: {error}"))
    }

    fn save(&self, manifest: &Manifest) -> Result<(), String> {
        let parent = self
            .manifest_path
            .parent()
            .ok_or_else(|| "error: plugin manifest has no parent directory".to_owned())?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("error: unable to create plugin data directory: {error}"))?;

        let temporary_path = self.manifest_path.with_extension("json.tmp");
        let content = serde_json::to_string_pretty(manifest)
            .map_err(|error| format!("error: unable to serialize plugin manifest: {error}"))?;
        fs::write(&temporary_path, content)
            .map_err(|error| format!("error: unable to write plugin manifest: {error}"))?;
        fs::rename(&temporary_path, &self.manifest_path)
            .map_err(|error| format!("error: unable to replace plugin manifest: {error}"))
    }

    #[cfg(test)]
    fn write_manifest(&self, manifest: &Manifest) -> std::io::Result<()> {
        if let Some(parent) = self.manifest_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(manifest).expect("manifest should serialize");
        fs::write(&self.manifest_path, content)
    }
}

#[cfg(test)]
mod tests {
    use super::{Manifest, Plugin, PluginStore};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_directory() -> std::path::PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("cwctl-test-{suffix}"))
    }

    #[test]
    fn missing_manifest_is_empty() {
        let directory = temporary_directory();
        let store = PluginStore::new(&directory);

        assert!(store
            .list()
            .expect("missing manifest should be valid")
            .is_empty());
    }

    #[test]
    fn reads_plugins_from_manifest() {
        let directory = temporary_directory();
        let store = PluginStore::new(&directory);
        let plugin = Plugin {
            name: "sample".to_owned(),
            version: "1.2.3".to_owned(),
            source: Some("https://example.test/sample".to_owned()),
        };
        store
            .write_manifest(&Manifest {
                plugins: vec![plugin.clone()],
            })
            .expect("manifest should be written");

        assert_eq!(
            store.find("sample").expect("manifest should load"),
            Some(plugin)
        );
        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }

    #[test]
    fn installs_and_replaces_plugins() {
        let directory = temporary_directory();
        let store = PluginStore::new(&directory);
        store
            .install(Plugin {
                name: "sample".to_owned(),
                version: "1.0.0".to_owned(),
                source: None,
            })
            .expect("plugin should be installed");
        store
            .install(Plugin {
                name: "sample".to_owned(),
                version: "2.0.0".to_owned(),
                source: None,
            })
            .expect("plugin should be replaced");

        let plugins = store.list().expect("manifest should load");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].version, "2.0.0");
        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }

    #[test]
    fn uninstalls_plugins() {
        let directory = temporary_directory();
        let store = PluginStore::new(&directory);
        store
            .install(Plugin {
                name: "sample".to_owned(),
                version: "1.0.0".to_owned(),
                source: None,
            })
            .expect("plugin should be installed");

        assert!(store.uninstall("sample").expect("plugin should uninstall"));
        assert!(!store
            .uninstall("sample")
            .expect("missing plugin is not removed"));
        assert!(store.list().expect("manifest should load").is_empty());
        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }

    #[test]
    fn updates_plugin_version() {
        let directory = temporary_directory();
        let store = PluginStore::new(&directory);
        store
            .install(Plugin {
                name: "sample".to_owned(),
                version: "1.0.0".to_owned(),
                source: Some("https://example.test/sample".to_owned()),
            })
            .expect("plugin should be installed");

        let updated = store
            .update("sample", "latest".to_owned())
            .expect("plugin should update")
            .expect("plugin should exist");

        assert_eq!(updated.version, "latest");
        assert_eq!(
            updated.source,
            Some("https://example.test/sample".to_owned())
        );
        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }

    #[test]
    fn rejects_invalid_manifest() {
        let directory = temporary_directory();
        fs::create_dir_all(&directory).expect("temporary directory should be created");
        fs::write(directory.join("plugins.json"), "{not json")
            .expect("invalid manifest should be written");
        let store = PluginStore::new(&directory);

        let error = store.list().expect_err("invalid manifest should fail");

        assert!(error.contains("invalid plugin manifest"));
        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }
}

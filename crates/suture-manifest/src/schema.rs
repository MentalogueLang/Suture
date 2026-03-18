use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectManifest {
    pub package_name: Option<String>,
    pub package_version: Option<String>,
    pub dependencies: BTreeMap<String, String>,
}

impl ProjectManifest {
    pub fn path_for_dir(dir: &Path) -> PathBuf {
        dir.join("suture.toml")
    }

    pub fn read(path: &Path) -> io::Result<Self> {
        let source = fs::read_to_string(path)?;
        Self::parse(&source).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid manifest `{}`", path.display()),
            )
        })
    }

    pub fn read_or_default(path: &Path) -> io::Result<Self> {
        if path.exists() {
            Self::read(path)
        } else {
            Ok(Self::default())
        }
    }

    pub fn write(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.to_toml())
    }

    pub fn set_dependency(&mut self, name: impl Into<String>, version: impl Into<String>) {
        self.dependencies.insert(name.into(), version.into());
    }

    pub fn remove_dependency(&mut self, name: &str) -> bool {
        self.dependencies.remove(name).is_some()
    }

    pub fn to_toml(&self) -> String {
        let mut output = String::new();
        if self.package_name.is_some() || self.package_version.is_some() {
            output.push_str("[package]\n");
            if let Some(name) = &self.package_name {
                output.push_str(&format!("name = \"{}\"\n", escape_toml(name)));
            }
            if let Some(version) = &self.package_version {
                output.push_str(&format!("version = \"{}\"\n", escape_toml(version)));
            }
            output.push('\n');
        }

        output.push_str("[dependencies]\n");
        for (name, version) in &self.dependencies {
            output.push_str(&format!("{name} = \"{}\"\n", escape_toml(version)));
        }
        output
    }

    fn parse(source: &str) -> Option<Self> {
        let mut manifest = Self::default();
        let mut section = None;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                section = Some(
                    trimmed
                        .trim_start_matches('[')
                        .trim_end_matches(']')
                        .trim()
                        .to_string(),
                );
                continue;
            }

            let (key, value) = parse_key_value(trimmed)?;
            match section.as_deref() {
                Some("package") => match key {
                    "name" => manifest.package_name = Some(value),
                    "version" => manifest.package_version = Some(value),
                    _ => {}
                },
                Some("dependencies") => {
                    manifest.dependencies.insert(key.to_string(), value);
                }
                _ => {}
            }
        }

        Some(manifest)
    }
}

fn parse_key_value(line: &str) -> Option<(&str, String)> {
    let (key, rest) = line.split_once('=')?;
    Some((key.trim(), trim_quotes(rest.trim())))
}

fn trim_quotes(value: &str) -> String {
    let mut trimmed = value.trim().to_string();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed = trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed
}

fn escape_toml(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

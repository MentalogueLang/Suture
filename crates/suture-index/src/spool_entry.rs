use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoolEntry {
    pub name: String,
    pub version: String,
    pub summary: Option<String>,
    pub source_git: String,
    pub source_tag: Option<String>,
    pub source_rev: Option<String>,
    pub build_entry: Option<String>,
    pub dependencies: Vec<(String, String)>,
    pub dev_dependencies: Vec<(String, String)>,
    pub artifacts: Vec<SpoolArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoolArtifact {
    pub target: String,
    pub url: String,
    pub checksum: Option<String>,
}

impl SpoolEntry {
    pub fn from_parts(
        name: &str,
        version: &str,
        git: &str,
        tag: Option<&str>,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            summary: None,
            source_git: git.to_string(),
            source_tag: tag.map(|value| value.to_string()),
            source_rev: None,
            build_entry: None,
            dependencies: Vec::new(),
            dev_dependencies: Vec::new(),
            artifacts: Vec::new(),
        }
    }

    pub fn to_toml(&self) -> String {
        let mut output = String::new();
        output.push_str("[package]\n");
        output.push_str(&format!("name = \"{}\"\n", self.name));
        output.push_str(&format!("version = \"{}\"\n", self.version));
        if let Some(summary) = &self.summary {
            output.push_str(&format!("summary = \"{}\"\n", summary));
        }
        output.push('\n');
        output.push_str("[source]\n");
        output.push_str(&format!("git = \"{}\"\n", self.source_git));
        if let Some(tag) = &self.source_tag {
            output.push_str(&format!("tag = \"{}\"\n", tag));
        }
        if let Some(rev) = &self.source_rev {
            output.push_str(&format!("rev = \"{}\"\n", rev));
        }
        if let Some(entry) = &self.build_entry {
            output.push('\n');
            output.push_str("[build]\n");
            output.push_str(&format!("entry = \"{}\"\n", entry));
        }

        if !self.dependencies.is_empty() {
            output.push('\n');
            output.push_str("[dependencies]\n");
            for (name, version) in &self.dependencies {
                output.push_str(&format!("{name} = \"{}\"\n", version));
            }
        }

        if !self.dev_dependencies.is_empty() {
            output.push('\n');
            output.push_str("[dev-dependencies]\n");
            for (name, version) in &self.dev_dependencies {
                output.push_str(&format!("{name} = \"{}\"\n", version));
            }
        }

        for artifact in &self.artifacts {
            output.push('\n');
            output.push_str("[[artifacts]]\n");
            output.push_str(&format!("target = \"{}\"\n", artifact.target));
            output.push_str(&format!("url = \"{}\"\n", artifact.url));
            if let Some(checksum) = &artifact.checksum {
                output.push_str(&format!("checksum = \"{}\"\n", checksum));
            }
        }
        output
    }
}

impl SpoolEntry {
    pub fn read(path: &Path) -> io::Result<Self> {
        let source = fs::read_to_string(path)?;
        Self::parse(&source).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid spool entry `{}`", path.display()),
            )
        })
    }

    fn parse(source: &str) -> Option<Self> {
        let mut section = None;
        let mut name = None;
        let mut version = None;
        let mut summary = None;
        let mut source_git = None;
        let mut source_tag = None;
        let mut source_rev = None;
        let mut build_entry = None;
        let mut dependencies = Vec::new();
        let mut dev_dependencies = Vec::new();
        let mut artifacts = Vec::new();
        let mut current_artifact: Option<SpoolArtifact> = None;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if trimmed.starts_with("[[") && trimmed.ends_with("]]") {
                if let Some(artifact) = current_artifact.take() {
                    artifacts.push(artifact);
                }
                let name = trimmed.trim_start_matches('[').trim_end_matches(']');
                section = Some(name.trim().to_string());
                if section.as_deref() == Some("artifacts") {
                    current_artifact = Some(SpoolArtifact {
                        target: String::new(),
                        url: String::new(),
                        checksum: None,
                    });
                }
                continue;
            }
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if let Some(artifact) = current_artifact.take() {
                    artifacts.push(artifact);
                }
                let name = trimmed.trim_start_matches('[').trim_end_matches(']');
                section = Some(name.trim().to_string());
                continue;
            }

            let Some((key, value)) = parse_key_value(trimmed) else {
                continue;
            };

            match section.as_deref() {
                Some("package") => match key {
                    "name" => name = Some(value),
                    "version" => version = Some(value),
                    "summary" => summary = Some(value),
                    _ => {}
                },
                Some("source") => match key {
                    "git" => source_git = Some(value),
                    "tag" => source_tag = Some(value),
                    "rev" => source_rev = Some(value),
                    _ => {}
                },
                Some("build") => match key {
                    "entry" => build_entry = Some(value),
                    _ => {}
                },
                Some("dependencies") => dependencies.push((key.to_string(), value)),
                Some("dev-dependencies") => dev_dependencies.push((key.to_string(), value)),
                Some("artifacts") => {
                    if let Some(artifact) = current_artifact.as_mut() {
                        match key {
                            "target" => artifact.target = value,
                            "url" => artifact.url = value,
                            "checksum" => artifact.checksum = Some(value),
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if let Some(artifact) = current_artifact.take() {
            artifacts.push(artifact);
        }

        Some(Self {
            name: name?,
            version: version?,
            summary,
            source_git: source_git?,
            source_tag,
            source_rev,
            build_entry,
            dependencies,
            dev_dependencies,
            artifacts,
        })
    }
}

fn parse_key_value(line: &str) -> Option<(&str, String)> {
    let (key, rest) = line.split_once('=')?;
    let value = trim_quotes(rest.trim());
    Some((key.trim(), value))
}

fn trim_quotes(value: &str) -> String {
    let mut trimmed = value.trim().to_string();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed = trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed
}

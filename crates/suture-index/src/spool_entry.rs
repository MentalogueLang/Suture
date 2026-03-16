use std::fs;
use std::io;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpoolEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: Option<String>,
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
        let mut name = None;
        let mut version = None;
        let mut source_url = None;
        let mut checksum = None;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some(value) = parse_value(trimmed, "name") {
                name = Some(value);
                continue;
            }
            if let Some(value) = parse_value(trimmed, "version") {
                version = Some(value);
                continue;
            }
            if let Some(value) = parse_value(trimmed, "source") {
                source_url = Some(value);
                continue;
            }
            if let Some(value) = parse_value(trimmed, "checksum") {
                checksum = Some(value);
                continue;
            }
        }

        Some(Self {
            name: name?,
            version: version?,
            source: source_url?,
            checksum,
        })
    }
}

fn parse_value(line: &str, key: &str) -> Option<String> {
    let rest = line.strip_prefix(key)?.trim_start();
    let value = rest.strip_prefix('=')?.trim();
    Some(trim_quotes(value))
}

fn trim_quotes(value: &str) -> String {
    let mut trimmed = value.trim().to_string();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        trimmed = trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed
}

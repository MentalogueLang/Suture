use std::env;
use std::path::Path;

use suture_index::SpoolEntry;

pub fn run(args: &[String]) -> Result<(), String> {
    if args.is_empty() {
        return Err(usage());
    }

    let mut rate_limit = "2".to_string();
    if looks_like_spool_file(&args[0]) {
        let mut index = 1;
        while index < args.len() {
            match args[index].as_str() {
                "--rate" => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        return Err("missing value after --rate".to_string());
                    };
                    rate_limit = value.trim().to_string();
                }
                other => return Err(format!("unknown flag `{other}`")),
            }
            index += 1;
        }

        let path = Path::new(&args[0]);
        let entry = SpoolEntry::read(path).map_err(|error| error.to_string())?;
        let body = build_toml_comment(&entry.to_toml(), &rate_limit);
        return submit_comment(&body).map(|_| {
            println!("submitted spool upload request for {} {}", entry.name, entry.version);
        });
    }

    if args.len() < 3 {
        return Err(usage());
    }

    let name = args[0].trim();
    let version = args[1].trim().trim_start_matches('v');
    let git = args[2].trim();
    if name.is_empty() || version.is_empty() || git.is_empty() {
        return Err("name, version, and git url are required".to_string());
    }

    let mut tag = None;
    let mut summary = None;
    let mut build_entry = None;
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--tag" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after --tag".to_string());
                };
                tag = Some(value.trim().to_string());
            }
            "--summary" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after --summary".to_string());
                };
                summary = Some(value.trim().to_string());
            }
            "--entry" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after --entry".to_string());
                };
                build_entry = Some(value.trim().to_string());
            }
            "--rate" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    return Err("missing value after --rate".to_string());
                };
                rate_limit = value.trim().to_string();
            }
            other => return Err(format!("unknown flag `{other}`")),
        }
        index += 1;
    }

    let mut entry = SpoolEntry::from_parts(name, version, git, tag.as_deref());
    entry.summary = summary;
    entry.build_entry = build_entry;
    let body = build_toml_comment(&entry.to_toml(), &rate_limit);
    submit_comment(&body)?;
    println!("submitted spool upload request for {name} {version}");
    Ok(())
}

fn usage() -> String {
    [
        "usage:",
        "  suture publish <spool.toml> [--rate <minutes>]",
        "  suture publish <name> <version> <git-url> [--tag <tag>] [--summary <text>] [--entry <path>] [--rate <minutes>]",
        "env:",
        "  SUTURE_SPOOLS_REPO (default MentalogueLang/Spools-Index)",
        "  SUTURE_SPOOLS_ISSUE (default 1)",
        "  SUTURE_SPOOLS_TOKEN (or GH_TOKEN/GITHUB_TOKEN)",
    ]
    .join("\n")
}

fn build_toml_comment(source: &str, rate_limit: &str) -> String {
    format!(
        "/spool-upload-toml rate_limit_minutes={}\n```toml\n{}\n```",
        escape_value(rate_limit),
        source.trim_end()
    )
}

fn escape_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

fn submit_comment(body: &str) -> Result<(), String> {
    let repo = env::var("SUTURE_SPOOLS_REPO")
        .unwrap_or_else(|_| "MentalogueLang/Spools-Index".to_string());
    let issue = env::var("SUTURE_SPOOLS_ISSUE").unwrap_or_else(|_| "1".to_string());
    let token = env::var("SUTURE_SPOOLS_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .or_else(|_| env::var("GH_TOKEN"))
        .map_err(|_| "missing token; set SUTURE_SPOOLS_TOKEN or GH_TOKEN".to_string())?;
    post_comment(&repo, &issue, body, &token)
}

fn post_comment(repo: &str, issue: &str, body: &str, token: &str) -> Result<(), String> {
    let url = format!("https://api.github.com/repos/{repo}/issues/{issue}/comments");
    let payload = format!("{{\"body\":\"{}\"}}", escape_json(body));
    let response = ureq::post(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "suture-cli")
        .set("Authorization", &format!("Bearer {token}"))
        .send_string(&payload);
    match response {
        Ok(response) => {
            let _ = response;
            Ok(())
        }
        Err(error) => {
            let status = match error {
                ureq::Error::Status(code, _) => code,
                ureq::Error::Transport(_) => 500,
            };
            Err(format!("github api error (status {status})"))
        }
    }
}

fn escape_json(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn looks_like_spool_file(value: &str) -> bool {
    value.ends_with(".toml") || Path::new(value).exists()
}

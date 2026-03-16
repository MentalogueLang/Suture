use std::env;

pub fn run(args: &[String]) -> Result<(), String> {
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
    let mut rate_limit = "2".to_string();
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

    let repo = env::var("SUTURE_SPOOLS_REPO")
        .unwrap_or_else(|_| "MentalogueLang/Spools-Index".to_string());
    let issue = env::var("SUTURE_SPOOLS_ISSUE").unwrap_or_else(|_| "1".to_string());
    let token = env::var("SUTURE_SPOOLS_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .or_else(|_| env::var("GH_TOKEN"))
        .map_err(|_| "missing token; set SUTURE_SPOOLS_TOKEN or GH_TOKEN".to_string())?;

    let command = build_comment(name, version, git, tag.as_deref(), summary.as_deref(), &rate_limit);
    post_comment(&repo, &issue, &command, &token)?;
    println!("submitted spool upload request for {name} {version}");
    Ok(())
}

fn usage() -> String {
    [
        "usage:",
        "  suture publish <name> <version> <git-url> [--tag <tag>] [--summary <text>] [--rate <minutes>]",
        "env:",
        "  SUTURE_SPOOLS_REPO (default MentalogueLang/Spools-Index)",
        "  SUTURE_SPOOLS_ISSUE (default 1)",
        "  SUTURE_SPOOLS_TOKEN (or GH_TOKEN/GITHUB_TOKEN)",
    ]
    .join("\n")
}

fn build_comment(
    name: &str,
    version: &str,
    git: &str,
    tag: Option<&str>,
    summary: Option<&str>,
    rate_limit: &str,
) -> String {
    let mut parts = vec![
        "/spool-upload".to_string(),
        format!("name=\"{}\"", escape_value(name)),
        format!("version=\"{}\"", escape_value(version)),
        format!("git=\"{}\"", escape_value(git)),
        format!("rate_limit_minutes={}", escape_value(rate_limit)),
    ];
    if let Some(tag) = tag {
        parts.push(format!("tag=\"{}\"", escape_value(tag)));
    }
    if let Some(summary) = summary {
        parts.push(format!("summary=\"{}\"", escape_value(summary)));
    }
    parts.join(" ")
}

fn escape_value(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
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
            let status = error.status().unwrap_or(500);
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

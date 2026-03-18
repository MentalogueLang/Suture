pub mod add;
pub mod remove;
pub mod pull;
pub mod publish;
pub mod spool;
pub mod audit;

pub fn dispatch(args: &[String]) -> Result<(), String> {
    let Some((command, rest)) = args.split_first() else {
        return Err(usage());
    };

    match command.as_str() {
        "add" => add::run(rest),
        "remove" => remove::run(rest),
        "pull" => pull::run(rest),
        "publish" => publish::run(rest),
        "spool" => spool::run(rest),
        "audit" => audit::run(rest),
        "help" | "--help" | "-h" => Err(usage()),
        other => Err(format!("unknown command `{other}`\n\n{}", usage())),
    }
}

pub fn usage() -> String {
    [
        "usage:",
        "  suture add <spool> <version>",
        "  suture remove <spool>",
        "  suture pull",
        "  suture publish <spool.toml> [--rate <minutes>]",
        "  suture publish <name> <version> <git-url> [--tag <tag>] [--summary <text>] [--entry <path>] [--rate <minutes>]",
        "  suture spool add <name> <version> <git-url> [--tag <tag>] [--summary <text>] [--entry <path>]",
        "  suture spool list",
        "  suture audit",
    ]
    .join("\n")
}

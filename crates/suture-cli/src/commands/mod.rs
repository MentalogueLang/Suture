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
        "  suture add <thread> <version>",
        "  suture remove <thread>",
        "  suture pull",
        "  suture publish",
        "  suture spool add <name> <version> <git-url> [--tag <tag>]",
        "  suture spool list",
        "  suture audit",
    ]
    .join("\n")
}

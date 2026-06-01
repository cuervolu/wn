use miette::IntoDiagnostic;
use owo_colors::OwoColorize;

const REPO_OWNER: &str = "cuervolu";
const REPO_NAME: &str = "wn";

pub fn run_update(force: bool) -> miette::Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    println!("{} v{}", "Versión actual:".dimmed(), current.cyan(),);
    println!("{}", "Buscando actualizaciones...".dimmed());

    let mut updater = configure_updater(current, force);

    if force {
        let latest = latest_release_version(current)?;
        let tag = forced_target_version_tag(&latest);
        updater.target_version_tag(&tag);
    }

    let status = updater
        .build()
        .into_diagnostic()?
        .update()
        .into_diagnostic()?;

    if status.updated() {
        println!(
            "{} Actualizado a v{}",
            "✓".green().bold(),
            status.version().cyan().bold(),
        );
    } else {
        println!(
            "{} Wena choro! Ya tienes la última versión (v{}).",
            "✓".green(),
            current.cyan(),
        );
    }

    Ok(())
}

fn configure_updater(current: &str, force: bool) -> self_update::backends::github::UpdateBuilder {
    let mut updater = self_update::backends::github::Update::configure();
    updater
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .identifier("wn-cli")
        .bin_name("wn")
        .show_output(false)
        .show_download_progress(true)
        .current_version(current)
        .no_confirm(force);
    updater
}

fn latest_release_version(current: &str) -> miette::Result<String> {
    let updater = configure_updater(current, true);
    let latest = updater
        .build()
        .into_diagnostic()?
        .get_latest_release()
        .into_diagnostic()?;

    Ok(latest.version)
}

fn forced_target_version_tag(version: &str) -> String {
    let version = version.trim();

    if version.starts_with('v') {
        version.to_owned()
    } else {
        format!("v{version}")
    }
}

#[cfg(test)]
mod tests {
    use super::forced_target_version_tag;

    #[test]
    fn forced_target_version_tag_adds_v_prefix() {
        assert_eq!(forced_target_version_tag("1.2.3"), "v1.2.3");
    }

    #[test]
    fn forced_target_version_tag_keeps_existing_v_prefix() {
        assert_eq!(forced_target_version_tag("v1.2.3"), "v1.2.3");
    }

    #[test]
    fn forced_target_version_tag_trims_version() {
        assert_eq!(forced_target_version_tag(" 1.2.3 "), "v1.2.3");
    }
}

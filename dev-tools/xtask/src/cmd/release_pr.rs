use std::collections::{BTreeMap, HashSet};
use std::process::Command;

use anyhow::Context;
use cargo_metadata::camino::Utf8PathBuf;

use crate::cmd::{IGNORED_PACKAGES, change_logs};

#[derive(Debug, Clone, clap::Parser)]
pub struct ReleasePr {}

struct Package {
    version: String,
    changelog_path: Option<Utf8PathBuf>,
    previous_version: Option<String>,
}

impl ReleasePr {
    pub fn run(self) -> anyhow::Result<()> {
        let metadata = crate::utils::metadata()?;

        let workspace_package_ids = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();

        let mut packages = BTreeMap::new();

        for package in metadata.packages.iter().filter(|p| workspace_package_ids.contains(&p.id)) {
            if package.publish.as_ref().is_some_and(|p| p.is_empty()) || IGNORED_PACKAGES.contains(&package.name.as_str()) {
                continue;
            }

            let previous_version =
                is_published_on_crates_io(&package.name, &package.version.to_string()).context("crates.io check failed")?;
            if previous_version.as_ref().is_none_or(|v| v != &package.version.to_string()) {
                eprintln!("\tnot published");
                let path = package.manifest_path.parent().unwrap().join("CHANGELOG.md");
                packages.insert(
                    package.name.clone(),
                    Package {
                        version: package.version.to_string(),
                        changelog_path: path.exists().then_some(path),
                        previous_version,
                    },
                );
            } else {
                eprintln!("\talready published");
            }
        }

        if packages.is_empty() {
            eprintln!("no packages need to be published");
            return Ok(());
        }

        let mut release_plz = Command::new("release-plz");

        release_plz
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .arg("update")
            .arg("--disable-dependant-updates");

        for package in packages.keys() {
            release_plz.arg("-p").arg(package);
        }

        change_logs::generate::Generate {
            exclude_packages: Vec::new(),
            packages: packages
                .iter()
                .filter_map(|(name, p)| p.changelog_path.as_ref().and(Some(name)))
                .cloned()
                .collect(),
        }
        .run()
        .context("changelogs")?;

        let mut pr_body = String::from("## 🤖 New release\n\n");

        for (name, package) in packages {
            pr_body.push_str(&format!("* `{name}` -> {}\n", package.version));
            if let Some(changelog_path) = &package.changelog_path {
                let changelog = std::fs::read_to_string(changelog_path).expect("missing changelog");

                let mut updated = String::new();

                updated.push_str("## [Unreleased]\n\n");

                updated.push_str(&format!(
                    "## [{version}](https://github.com/ScuffleCloud/scuffle/releases/tag/{name}-v{version}) - {date}\n\n",
                    version = package.version,
                    date = chrono::Utc::now().date_naive().format("%Y-%m-%d")
                ));
                if let Some(previous_version) = &package.previous_version {
                    updated.push_str(&format!(
                        "[View diff on diff.rs](https://diff.rs/{name}/{previous_version}/{name}/{}/Cargo.toml)\n",
                        package.version
                    ));
                }

                let changelog = changelog.replace("## [Unreleased]\n", &updated);

                std::fs::write(changelog_path, changelog).context("write changelog")?;
            }
        }

        println!("{pr_body}");

        Ok(())
    }
}

fn is_published_on_crates_io(crate_name: &str, version: &str) -> anyhow::Result<Option<String>> {
    let url = crate_index_url(crate_name);

    eprintln!("checking {crate_name}@{version} on crates.io");
    let output = reqwest::blocking::get(url).context("get")?;

    if output.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    let output = output.error_for_status().context("status")?;

    let output = output.text().context("text")?;
    let mut last_version = None;
    for line in output.split('\n').map(|line| line.trim()).filter(|line| !line.is_empty()) {
        let krate = serde_json::from_str::<serde_json::Value>(line).context("json")?;
        let vers = krate.get("vers").unwrap_or(&serde_json::Value::Null).as_str();
        if let Some(vers) = vers {
            if vers == version {
                return Ok(Some(version.to_string()));
            }
            last_version = Some(vers.to_owned());
        }
    }

    Ok(last_version)
}

fn crate_index_url(crate_name: &str) -> String {
    let name = crate_name.to_lowercase();
    let len = name.len();

    match len {
        0 => panic!("Invalid crate name"),
        1 => format!("https://index.crates.io/1/{name}"),
        2 => format!("https://index.crates.io/2/{name}"),
        3 => format!("https://index.crates.io/3/{}/{}", &name[0..1], name),
        _ => {
            let prefix = &name[0..2];
            let suffix = &name[2..4];
            format!("https://index.crates.io/{prefix}/{suffix}/{name}")
        }
    }
}

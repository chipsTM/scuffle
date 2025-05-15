use std::collections::{HashMap, HashSet};

use anyhow::Context;
use cargo_metadata::DependencyKind;

use crate::cmd::IGNORED_PACKAGES;

#[derive(Debug, Clone, clap::Parser)]
pub struct WorkspaceDeps {
    #[clap(long, short, value_delimiter = ',')]
    #[clap(alias = "package")]
    /// Packages to test
    packages: Vec<String>,
    #[clap(long, short, value_delimiter = ',')]
    #[clap(alias = "exclude-package")]
    /// Packages to exclude from testing
    exclude_packages: Vec<String>,
}

impl WorkspaceDeps {
    pub fn run(self) -> anyhow::Result<()> {
        let start = std::time::Instant::now();

        let metadata = crate::utils::metadata()?;

        let workspace_package_ids = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();

        let workspace_packages = metadata
            .packages
            .iter()
            .filter(|p| workspace_package_ids.contains(&p.id))
            .map(|p| (&p.id, p))
            .collect::<HashMap<_, _>>();

        let path_to_package = workspace_packages
            .values()
            .map(|p| (p.manifest_path.parent().unwrap(), &p.id))
            .collect::<HashMap<_, _>>();

        for package in metadata.packages.iter().filter(|p| workspace_package_ids.contains(&p.id)) {
            if (IGNORED_PACKAGES.contains(&package.name.as_str()) || self.exclude_packages.contains(&package.name))
                && (self.packages.is_empty() || !self.packages.contains(&package.name))
            {
                continue;
            }

            let toml = std::fs::read_to_string(&package.manifest_path)
                .with_context(|| format!("failed to read manifest for {}", package.name))?;
            let mut doc = toml
                .parse::<toml_edit::DocumentMut>()
                .with_context(|| format!("failed to parse manifest for {}", package.name))?;
            let mut changes = false;

            for dependency in package
                .dependencies
                .iter()
                .filter(|dep| dep.kind == DependencyKind::Development)
            {
                let Some(path) = dependency.path.as_deref() else {
                    continue;
                };

                if path_to_package.get(path).and_then(|id| workspace_packages.get(id)).is_none() {
                    continue;
                }

                doc["dev-dependencies"].as_table_mut().unwrap().remove(&dependency.name);
                println!("Removed dev-dependency `{}` in package `{}`", dependency.name, package.name);
                changes = true;
            }

            if changes {
                std::fs::write(&package.manifest_path, doc.to_string())
                    .with_context(|| format!("failed to write manifest for {}", package.name))?;
                println!("Removed dev-dependencies in {} @ {}", package.name, package.manifest_path);
            }
        }

        println!("Done in {:?}", start.elapsed());

        Ok(())
    }
}

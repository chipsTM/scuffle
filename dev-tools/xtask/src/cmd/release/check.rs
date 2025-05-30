use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt::Write;
use std::io::Read;
use std::process::Stdio;

use anyhow::Context;
use cargo_metadata::camino::{Utf8Path, Utf8PathBuf};
use cargo_metadata::{DependencyKind, semver};

use super::utils::Package;
use crate::cmd::release::update::{Fragment, PackageChangeLog};
use crate::cmd::release::utils::{
    GitReleaseArtifact, LicenseKind, PackageError, PackageErrorMissing, PackageFile, VersionBump, dep_kind_to_name,
};
use crate::utils::{self, cargo_cmd, concurrently, git_workdir_clean, relative_to};

#[derive(Debug, Clone, clap::Parser)]
pub struct Check {
    /// The pull request number
    #[arg(long, short = 'n')]
    pr_number: Option<u64>,
    /// The base branch to compare against to determine
    /// if something has changed.
    #[arg(long, default_value = "origin/main")]
    base_branch: String,
    /// Check everything, even if there are no changes
    /// from this branch to the base branch.
    #[arg(long)]
    all: bool,
    /// Packages to include in the check
    /// by default all packages are included
    #[arg(long = "package", short = 'p')]
    packages: Vec<String>,
    /// Allow the command to execute even if there are uncomitted changes in the workspace
    #[arg(long)]
    allow_dirty: bool,
    /// Output markdown to stdout (used for CI to generate PR comments / Summaries)
    #[arg(long, conflicts_with = "fix")]
    stdout_markdown: bool,
    /// Attempts to fix some of the issues.
    #[arg(long, requires = "pr_number")]
    fix: bool,
    /// Return a non-zero exit status at the end if a check failed.
    #[arg(long)]
    exit_status: bool,
    /// Concurrency to run at. By default, this is the total number of cpus on the host.
    #[arg(long, default_value_t = num_cpus::get())]
    concurrency: usize,
    /// Author to use for the changelog entries
    #[arg(long = "author")]
    authors: Vec<String>,
}

impl Check {
    pub fn run(mut self) -> anyhow::Result<()> {
        if !self.allow_dirty {
            git_workdir_clean()?;
        }

        self.authors.iter_mut().for_each(|author| {
            if !author.starts_with("@") {
                *author = format!("@{author}");
            }
        });

        let metadata = utils::metadata().context("metadata")?;
        let check_run = CheckRun::new(&metadata, &self.packages).context("check run")?;
        check_run.process(
            self.concurrency,
            &metadata.workspace_root,
            if self.all { Some(&self.base_branch) } else { None },
        )?;

        if self.fix && self.pr_number.is_none() {
            anyhow::bail!("--fix needs --pr-number to be provided");
        }

        let mut new_releases_markdown = Vec::new();
        let mut errors_markdown = Vec::new();

        let mut fragment = if let Some(pr_number) = self.pr_number {
            let fragment = Fragment::new(pr_number, &metadata.workspace_root)?;

            let mut unknown_packages = Vec::new();

            for (package, logs) in fragment.items().context("fragment items")? {
                let Some(pkg) = check_run.get_package(&package) else {
                    unknown_packages.push(package);
                    continue;
                };

                pkg.report_change();
                if logs.iter().any(|l| l.breaking) {
                    pkg.report_breaking_change();
                }
            }

            if !unknown_packages.is_empty() {
                errors_markdown.push("## Changelog Entry\n".into());
                for package in unknown_packages {
                    errors_markdown.push(format!("* unknown package entry `{package}`"))
                }
            }

            Some(fragment)
        } else {
            None
        };

        for package in check_run.groups().flatten() {
            let _span = tracing::info_span!("check", package = %package.name).entered();
            if self.fix {
                package
                    .fix(&check_run, &metadata.workspace_root, fragment.as_mut().unwrap())
                    .with_context(|| format!("fix {}", package.name.clone()))?;
            } else {
                package
                    .report(&mut new_releases_markdown, &mut errors_markdown, fragment.as_mut())
                    .with_context(|| format!("report {}", package.name.clone()))?;
            }
        }

        if let Some(mut fragment) = fragment {
            if fragment.changed() {
                tracing::info!(
                    "{} {}",
                    if fragment.deleted() { "creating" } else { "updating" },
                    relative_to(fragment.path(), &metadata.workspace_root),
                );
                fragment.save().context("save changelog")?;
            }
        }

        if self.stdout_markdown {
            print!(
                "{}",
                fmtools::fmt(|f| {
                    f.write_str("# ⭐ Package Changes\n\n")?;
                    if !new_releases_markdown.is_empty() {
                        for line in &new_releases_markdown {
                            f.write_str(line)?;
                            f.write_char('\n')?;
                        }
                        f.write_char('\n')?;
                    } else {
                        f.write_str("no changes\n\n")?;
                    }

                    if !errors_markdown.is_empty() {
                        f.write_str("# 💥 Errors \n\n")?;
                        for line in &errors_markdown {
                            f.write_str(line)?;
                            f.write_char('\n')?;
                        }
                        f.write_char('\n')?;
                    }

                    Ok(())
                })
            );
        }

        if self.exit_status && !errors_markdown.is_empty() {
            anyhow::bail!("exit requested at any error");
        }

        tracing::info!("complete");

        Ok(())
    }
}

impl Package {
    #[tracing::instrument(skip_all, fields(package = %self.name))]
    pub fn check(
        &self,
        packages: &BTreeMap<String, Self>,
        workspace_root: &Utf8Path,
        base_branch: Option<&str>,
    ) -> anyhow::Result<()> {
        if !base_branch.is_none_or(|branch| self.has_branch_changes(branch)) {
            tracing::debug!("skipping due to no changes run with --all to check this package");
            return Ok(());
        }

        let start = std::time::Instant::now();
        tracing::debug!("starting validating");

        let license = if self.license.is_none() && self.license_file.is_none() {
            self.report_issue(PackageErrorMissing::License);
            LicenseKind::from_text(LicenseKind::MIT_OR_APACHE2)
        } else if let Some(license) = &self.license {
            LicenseKind::from_text(license)
        } else {
            None
        };

        if let Some(license) = license {
            for kind in license {
                if !self
                    .manifest_path
                    .with_file_name(PackageFile::License(kind).to_string())
                    .exists()
                {
                    self.report_issue(PackageFile::License(kind));
                }
            }
        }

        if self.should_release() && !self.manifest_path.with_file_name(PackageFile::Readme.to_string()).exists() {
            self.report_issue(PackageFile::Readme);
        }

        if self.changelog_path().is_some_and(|path| !path.exists()) {
            self.report_issue(PackageFile::Changelog);
        }

        if self.should_release() && self.description.is_none() {
            self.report_issue(PackageErrorMissing::Description);
        }

        if self.should_release() && self.readme.is_none() {
            self.report_issue(PackageErrorMissing::Readme);
        }

        if self.should_release() && self.repository.is_none() {
            self.report_issue(PackageErrorMissing::Repopository);
        }

        if self.should_release() && self.authors.is_empty() {
            self.report_issue(PackageErrorMissing::Author);
        }

        if self.should_release() && self.documentation.is_none() {
            self.report_issue(PackageErrorMissing::Documentation);
        }

        match self.git_release() {
            Ok(Some(release)) => {
                for artifact in &release.artifacts {
                    match artifact {
                        GitReleaseArtifact::File { path, .. } => {
                            if !self.manifest_path.parent().unwrap().join(path).exists() {
                                self.report_issue(PackageError::GitReleaseArtifactFileMissing { path: path.to_string() });
                            }
                        }
                    }
                }
            }
            Ok(None) => {}
            Err(err) => {
                self.report_issue(PackageError::GitRelease {
                    error: format!("{err:#}"),
                });
            }
        }

        for dep in &self.dependencies {
            match &dep.kind {
                DependencyKind::Build | DependencyKind::Normal => {
                    if let Some(Some(pkg)) = dep.path.is_some().then(|| packages.get(&dep.name)) {
                        if dep.req.comparators.is_empty() && self.should_publish() {
                            self.report_issue(PackageError::missing_version(dep));
                        } else if pkg.group() == self.group()
                            && dep.req.comparators
                                != [semver::Comparator {
                                    major: self.version.major,
                                    minor: Some(self.version.minor),
                                    patch: Some(self.version.patch),
                                    op: semver::Op::Exact,
                                    pre: self.version.pre.clone(),
                                }]
                        {
                            self.report_issue(PackageError::grouped_version(dep));
                        }
                    } else if self.should_publish() {
                        if dep.registry.is_some()
                            || dep.req.comparators.is_empty()
                            || dep.source.as_ref().is_some_and(|s| !s.is_crates_io())
                        {
                            self.report_issue(PackageError::not_publish(dep));
                        }
                    }
                }
                DependencyKind::Development => {
                    if !dep.req.comparators.is_empty() && dep.path.is_some() && packages.contains_key(&dep.name) {
                        self.report_issue(PackageError::has_version(dep));
                    }
                }
                _ => continue,
            }
        }

        if let Some(commit) = self.last_git_commit().context("lookup commit")? {
            tracing::debug!("found git changes at {commit}");
            self.report_change();
        }

        static SINGLE_THREAD: std::sync::RwLock<()> = std::sync::RwLock::new(());

        if self.should_semver_checks() {
            if let Some(version) = self.last_published_version() {
                static ONCE: std::sync::Once = std::sync::Once::new();
                ONCE.call_once(|| {
                    std::thread::spawn(move || {
                        tracing::info!("running cargo-semver-checks");
                    });
                });

                tracing::debug!(
                    "running semver-checks against baseline-version: {}, current-version: {}",
                    version.vers,
                    self.version
                );

                let _guard = SINGLE_THREAD.read().unwrap();

                let semver_checks = cargo_cmd()
                    .arg("semver-checks")
                    .arg("-p")
                    .arg(self.name.as_ref())
                    .arg("--baseline-version")
                    .arg(version.vers.to_string())
                    .stderr(Stdio::piped())
                    .stdout(Stdio::piped())
                    .output()
                    .context("semver-checks")?;

                let stdout = String::from_utf8_lossy(&semver_checks.stdout);
                let stdout = stdout.trim().replace(workspace_root.as_str(), ".");
                if !semver_checks.status.success() {
                    let stderr = String::from_utf8_lossy(&semver_checks.stderr);
                    let stderr = stderr.trim().replace(workspace_root.as_str(), ".");
                    if stdout.is_empty() {
                        anyhow::bail!("semver-checks failed\n{stderr}");
                    } else {
                        self.set_semver_output(stderr.contains("requires new major version"), stdout.to_owned());
                    }
                } else {
                    self.set_semver_output(false, stdout.to_owned());
                }
            } else {
                tracing::info!("skipping semver-checks due to no published version was found on crates.io");
            }
        }

        if self.should_min_version_check() {
            let cargo_toml_str = std::fs::read_to_string(&self.manifest_path).context("read Cargo.toml")?;
            let mut cargo_toml_edit = cargo_toml_str.parse::<toml_edit::DocumentMut>().context("parse Cargo.toml")?;

            // Remove dev-dependencies to prevent them from effecting cargo's version resolution.
            cargo_toml_edit.remove("dev-dependencies");
            if let Some(target) = cargo_toml_edit.get_mut("target").and_then(|t| t.as_table_like_mut()) {
                for (_, item) in target.iter_mut() {
                    if let Some(table) = item.as_table_like_mut() {
                        table.remove("dev-dependencies");
                    }
                }
            }

            let mut dep_packages = Vec::new();
            for dep in &self.dependencies {
                let Some(pkg) = packages.get(&dep.name) else {
                    continue;
                };

                if dep.path.is_none() {
                    continue;
                }

                let is_version_bump =
                    self.should_publish() && self.last_published_version().is_some_and(|v| v.vers != self.version);
                if dep.req == pkg.unreleased_req() && !is_version_bump {
                    dep_packages.push(&dep.name);
                    continue;
                }

                let root = if let Some(target) = &dep.target {
                    &mut cargo_toml_edit["target"][&target.to_string()]
                } else {
                    cargo_toml_edit.as_item_mut()
                };

                let kind = match dep.kind {
                    DependencyKind::Build => "build-dependencies",
                    DependencyKind::Normal => "dependencies",
                    _ => continue,
                };

                let item = root[kind][&dep.name].as_table_like_mut().unwrap();
                let versions = pkg.published_versions();

                tracing::debug!(
                    "min-version-check: finding best version for {} = '{}' outof [{}]",
                    dep.name,
                    dep.req,
                    versions.iter().map(|v| v.vers.to_string()).collect::<Vec<_>>().join(", ")
                );

                if let Some(version) = versions.iter().find(|v| dep.req.matches(&v.vers)).map(|v| &v.vers) {
                    let pinned = semver::VersionReq {
                        comparators: vec![semver::Comparator {
                            op: semver::Op::Exact,
                            major: version.major,
                            minor: Some(version.minor),
                            patch: Some(version.patch),
                            pre: version.pre.clone(),
                        }],
                    };

                    item.remove("path");
                    item.insert("version", pinned.to_string().into());
                } else {
                    dep_packages.push(&dep.name);
                }
            }

            static ONCE: std::sync::Once = std::sync::Once::new();
            ONCE.call_once(|| {
                std::thread::spawn(move || {
                    tracing::info!("running min versions check");
                });
            });

            let cargo_toml_edit = cargo_toml_edit.to_string();
            let _guard = if cargo_toml_str != cargo_toml_edit {
                let guard = SINGLE_THREAD.write().unwrap();
                let undo = WriteUndo::new(
                    &self.manifest_path,
                    cargo_toml_edit.as_bytes(),
                    cargo_toml_str.into_bytes(),
                )?;
                Some((guard, undo))
            } else {
                None
            };

            let (mut read, write) = std::io::pipe()?;

            let mut cmd = cargo_cmd();
            cmd.env("RUSTC_BOOTSTRAP", "1")
                .stderr(write.try_clone()?)
                .stdout(write)
                .arg("-Zunstable-options")
                .arg("-Zpackage-workspace")
                .arg("publish")
                .arg("--dry-run")
                .arg("--allow-dirty")
                .arg("--all-features")
                .arg("--lockfile-path")
                .arg(workspace_root.join("target").join("release-checks").join("Cargo.lock"))
                .arg("--target-dir")
                .arg(workspace_root.join("target").join("release-checks"))
                .arg("-p")
                .arg(self.name.as_ref());

            for package in &dep_packages {
                cmd.arg("-p").arg(package);
            }

            let mut child = cmd.spawn().context("spawn")?;

            drop(cmd);

            let mut output = String::new();
            read.read_to_string(&mut output).context("invalid read")?;

            let result = child.wait().context("wait")?;
            if !result.success() {
                self.set_min_versions_output(output);
            }
        }

        if let Some(next_version) = self.next_version() {
            tracing::debug!(after = ?start.elapsed(), "validation finished, package needs a version bump ({next_version})");
        } else {
            tracing::debug!(after = ?start.elapsed(), "validation finished");
        }

        Ok(())
    }

    fn fix(&self, check_run: &CheckRun, workspace_root: &Utf8Path, fragment: &mut Fragment) -> anyhow::Result<()> {
        let cargo_toml_raw = std::fs::read_to_string(&self.manifest_path).context("read cargo toml")?;
        let mut cargo_toml = cargo_toml_raw.parse::<toml_edit::DocumentMut>().context("parse toml")?;
        if let Some(min_versions_output) = self.min_versions_output() {
            tracing::error!("min version error cannot be automatically fixed.");
            eprintln!("{min_versions_output}");
        }

        #[derive(PartialEq, PartialOrd, Eq, Ord)]
        enum ChangelogEntryType {
            DevDeps,
            Deps,
            CargoToml,
        }

        let mut changelogs = BTreeSet::new();

        for error in self.errors() {
            match error {
                PackageError::DevDependencyHasVersion { name, target } => {
                    let deps = if let Some(target) = target {
                        &mut cargo_toml["target"][target.to_string()]
                    } else {
                        cargo_toml.as_item_mut()
                    };

                    if deps["dev-dependencies"][&name]
                        .as_table_like_mut()
                        .expect("table like")
                        .remove("version")
                        .is_some()
                    {
                        changelogs.insert(ChangelogEntryType::DevDeps);
                    }
                }
                PackageError::DependencyMissingVersion { .. } => {}
                PackageError::DependencyGroupedVersion { .. } => {}
                PackageError::DependencyNotPublishable { .. } => {}
                PackageError::Missing(PackageErrorMissing::Author) => {
                    cargo_toml["package"]["authors"] =
                        toml_edit::Array::from_iter(["Scuffle <opensource@scuffle.cloud>"]).into();
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::Missing(PackageErrorMissing::Description) => {
                    cargo_toml["package"]["description"] = format!("{} is a work-in-progress!", self.name).into();
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::Missing(PackageErrorMissing::Documentation) => {
                    cargo_toml["package"]["documentation"] = format!("https://docs.rs/{}", self.name).into();
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::Missing(PackageErrorMissing::License) => {
                    cargo_toml["package"]["license"] = "MIT OR Apache-2.0".into();
                    for file in [
                        PackageFile::License(LicenseKind::Mit),
                        PackageFile::License(LicenseKind::Apache2),
                    ] {
                        let path = self.manifest_path.with_file_name(file.to_string());
                        let file_path = workspace_root.join(file.to_string());
                        let relative_path = relative_to(&file_path, path.parent().unwrap());
                        tracing::info!("creating {path}");
                        std::os::unix::fs::symlink(relative_path, path).context("license symlink")?;
                    }
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::Missing(PackageErrorMissing::ChangelogEntry) => {}
                PackageError::Missing(PackageErrorMissing::Readme) => {
                    cargo_toml["package"]["readme"] = "README.md".into();
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::Missing(PackageErrorMissing::Repopository) => {
                    cargo_toml["package"]["repository"] = "https://github.com/scufflecloud/scuffle".into();
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::MissingFile(file @ PackageFile::Changelog) => {
                    const CHANGELOG_TEMPLATE: &str = include_str!("./changelog_template.md");
                    let path = self.manifest_path.with_file_name(file.to_string());
                    tracing::info!("creating {}", relative_to(&path, workspace_root));
                    std::fs::write(path, CHANGELOG_TEMPLATE).context("changelog write")?;
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::MissingFile(file @ PackageFile::Readme) => {
                    const README_TEMPLATE: &str = include_str!("./readme_template.md");
                    let path = self.manifest_path.with_file_name(file.to_string());
                    tracing::info!("creating {}", relative_to(&path, workspace_root));
                    std::fs::write(path, README_TEMPLATE).context("readme write")?;
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::MissingFile(file @ PackageFile::License(_)) => {
                    let path = self.manifest_path.with_file_name(file.to_string());
                    let file_path = workspace_root.join(file.to_string());
                    let relative_path = relative_to(&file_path, path.parent().unwrap());
                    tracing::info!("creating {}", relative_to(&path, workspace_root));
                    std::os::unix::fs::symlink(relative_path, path).context("license symlink")?;
                    changelogs.insert(ChangelogEntryType::CargoToml);
                }
                PackageError::GitRelease { .. } => {}
                PackageError::GitReleaseArtifactFileMissing { .. } => {}
            }
        }

        for dep in &self.dependencies {
            if !matches!(dep.kind, DependencyKind::Normal | DependencyKind::Build) {
                continue;
            }

            let Some(dep_pkg) = check_run.get_package(&dep.name) else {
                continue;
            };

            let version = dep_pkg.version.clone();
            let req = if dep_pkg.group() == self.group() {
                semver::VersionReq {
                    comparators: vec![semver::Comparator {
                        major: version.major,
                        minor: Some(version.minor),
                        patch: Some(version.patch),
                        pre: version.pre.clone(),
                        op: semver::Op::Exact,
                    }],
                }
            } else if !dep.req.matches(&version) {
                semver::VersionReq {
                    comparators: vec![semver::Comparator {
                        major: version.major,
                        minor: Some(version.minor),
                        patch: Some(version.patch),
                        pre: version.pre.clone(),
                        op: semver::Op::Caret,
                    }],
                }
            } else {
                continue;
            };

            if req == dep.req {
                continue;
            }

            let table = if let Some(target) = &dep.target {
                &mut cargo_toml["target"][target.to_string()][dep_kind_to_name(&dep.kind)]
            } else {
                &mut cargo_toml[dep_kind_to_name(&dep.kind)]
            };

            changelogs.insert(ChangelogEntryType::Deps);
            table[&dep.name]["version"] = req.to_string().into();
        }

        if self.changelog_path().is_some() {
            for changelog in changelogs {
                fragment.add_log(
                    &self.name,
                    &match changelog {
                        ChangelogEntryType::CargoToml => PackageChangeLog::new("docs", "cleaned up documentation"),
                        ChangelogEntryType::Deps => PackageChangeLog::new("chore", "cleaned up grouped dependencies"),
                        ChangelogEntryType::DevDeps => PackageChangeLog::new("chore", "cleaned up dev-dependencies"),
                    },
                );
            }
        }

        let cargo_toml_updated = cargo_toml.to_string();
        if cargo_toml_updated != cargo_toml_raw {
            tracing::info!(
                "{}",
                fmtools::fmt(|f| {
                    f.write_str("updating ")?;
                    f.write_str(relative_to(&self.manifest_path, workspace_root).as_str())?;
                    Ok(())
                })
            );
            std::fs::write(&self.manifest_path, cargo_toml.to_string()).context("manifest write")?;
        }

        Ok(())
    }

    fn report(
        &self,
        new_releases_markdown: &mut Vec<String>,
        errors_markdown: &mut Vec<String>,
        fragment: Option<&mut Fragment>,
    ) -> anyhow::Result<()> {
        let semver_output = self.semver_output();
        if let Some(version_bump) = self.version_bump() {
            let changes_text = match version_bump {
                VersionBump::Major => "has breaking changes",
                VersionBump::Minor => "has changes",
            };
            tracing::info!("{changes_text}");
            new_releases_markdown.push(
                fmtools::fmt(|f| {
                    write!(f, "* `{}`: {changes_text}", self.name)?;
                    if let Some((true, logs)) = &semver_output {
                        let mut f = indent_write::fmt::IndentWriter::new("  ", f);
                        f.write_str("\n\n<details><summary>cargo-semver-checks</summary>\n\n")?;
                        write!(f, "```\n{logs}\n```\n\n")?;
                        f.write_str("</details>\n")?;
                    }
                    Ok(())
                })
                .to_string(),
            );
        }

        let mut errors = self.errors();
        if let Some(fragment) = &fragment {
            if !fragment.has_package(&self.name) && self.version_bump().is_some() && self.changelog_path().is_some() {
                tracing::warn!(package = %self.name, "changelog entry must be provided");
                errors.insert(0, PackageError::Missing(PackageErrorMissing::ChangelogEntry));
            }
        }

        let min_versions_output = self.min_versions_output();

        if !errors.is_empty() || min_versions_output.is_some() {
            errors_markdown.push(format!("## {}\n", self.name));
            for error in errors.iter() {
                errors_markdown.push(format!("* {error}"))
            }
            if let Some(min_versions_output) = min_versions_output {
                errors_markdown.push(format!("* min package versions issue\n\n<details><summary>Output</summary>\n\n```\n{min_versions_output}\n```\n\n</details>\n"))
            }
            errors_markdown.push("".into());
        }

        Ok(())
    }
}

pub struct CheckRun {
    packages: BTreeMap<String, Package>,
    accepted_groups: HashSet<String>,
    groups: BTreeMap<String, Vec<Package>>,
}

impl CheckRun {
    pub fn new(metadata: &cargo_metadata::Metadata, allowed_packages: &[String]) -> anyhow::Result<Self> {
        let members = metadata.workspace_members.iter().cloned().collect::<HashSet<_>>();
        let packages = metadata
            .packages
            .iter()
            .filter(|p| members.contains(&p.id))
            .map(|p| Ok((p.name.as_ref().to_owned(), Package::new(p.clone())?)))
            .collect::<anyhow::Result<BTreeMap<_, _>>>()?;

        let accepted_groups = packages
            .values()
            .filter(|p| allowed_packages.contains(&p.name) || allowed_packages.is_empty())
            .map(|p| p.group().to_owned())
            .collect::<HashSet<_>>();

        let groups = packages
            .values()
            .cloned()
            .fold(BTreeMap::<_, Vec<_>>::new(), |mut groups, package| {
                if !accepted_groups.contains(package.group()) {
                    return groups;
                }

                let entry = groups.entry(package.group().to_owned()).or_default();
                if package.name.as_ref() == package.group() {
                    entry.insert(0, package);
                } else {
                    entry.push(package);
                }

                groups
            });

        Ok(Self {
            accepted_groups,
            groups,
            packages,
        })
    }

    pub fn process(&self, concurrency: usize, workspace_root: &Utf8Path, base_branch: Option<&str>) -> anyhow::Result<()> {
        concurrently::<_, _, anyhow::Result<()>>(concurrency, self.packages.values(), |p| p.fetch_published())?;

        concurrently::<_, _, anyhow::Result<()>>(concurrency, self.groups.values().flatten(), |p| {
            p.check(&self.packages, workspace_root, base_branch)
        })?;

        Ok(())
    }

    pub fn packages(&self) -> impl Iterator<Item = &'_ Package> {
        self.groups.values().flatten()
    }

    pub fn get_package(&self, name: impl AsRef<str>) -> Option<&Package> {
        self.packages.get(name.as_ref())
    }

    pub fn is_accepted_group(&self, group: impl AsRef<str>) -> bool {
        self.accepted_groups.contains(group.as_ref())
    }

    pub fn all_packages(&self) -> impl Iterator<Item = &'_ Package> {
        self.packages.values()
    }

    pub fn groups(&self) -> impl Iterator<Item = &'_ [Package]> {
        self.groups.values().map(|g| g.as_slice())
    }
}

struct WriteUndo {
    og: Vec<u8>,
    path: Utf8PathBuf,
}

impl WriteUndo {
    fn new(path: &Utf8Path, content: &[u8], og: Vec<u8>) -> anyhow::Result<Self> {
        std::fs::write(path, content).context("write")?;
        Ok(Self {
            og,
            path: path.to_path_buf(),
        })
    }
}

impl Drop for WriteUndo {
    fn drop(&mut self) {
        if let Err(err) = std::fs::write(&self.path, &self.og) {
            tracing::error!(path = %self.path, "failed to undo write: {err}");
        }
    }
}

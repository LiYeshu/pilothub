use std::path::PathBuf;

use anyhow::Result;

pub mod apm;
pub mod runtime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManagerScope {
    Project,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerContext {
    pub working_dir: PathBuf,
    pub scope: PackageManagerScope,
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerCommand {
    pub program: String,
    pub args: Vec<String>,
    pub working_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageManagerAvailability {
    pub available: bool,
    pub version: Option<String>,
    pub message: Option<String>,
}

pub trait PackageManager {
    fn id(&self) -> &'static str;

    fn availability(&self) -> PackageManagerAvailability;

    fn install(
        &self,
        package: &str,
        context: &PackageManagerContext,
    ) -> Result<PackageManagerCommand>;

    fn uninstall(
        &self,
        package: &str,
        context: &PackageManagerContext,
    ) -> Result<PackageManagerCommand>;

    fn update(
        &self,
        package: Option<&str>,
        accept_changes: bool,
        context: &PackageManagerContext,
    ) -> Result<PackageManagerCommand>;

    fn list(&self, context: &PackageManagerContext) -> Result<PackageManagerCommand>;

    fn doctor(&self, context: &PackageManagerContext) -> Result<PackageManagerCommand>;
}

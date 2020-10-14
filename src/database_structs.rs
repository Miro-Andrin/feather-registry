/*
This files containst structs that represent entries in the database.
If you want to see the layout of the database, see the @TODO

*/
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Dependency {
    /// Name of the dependency.
    /// If the dependency is renamed from the original package name,
    /// this is the original name. The new package name is stored in
    /// the `explicit_name_in_toml` field.
    name: String,
    /// The semver requirement for this dependency.
    version_req: String,
    /// Array of features (as strings) enabled for this dependency.
    features: Vec<String>,
    /// Boolean of whether or not this is an optional dependency.
    optional: bool,
    /// Boolean of whether or not default features are enabled.
    default_feautres: bool,
    /// The target platform for the dependency.
    /// null if not a target dependency.
    /// Otherwise, a string such as "cfg(windows)".
    target: Option<String>,
    /// The dependency kind.
    /// "dev", "build", or "normal".
    kind: String,
    /// The URL of the index of the registry where this dependency is
    /// from as a string. If not specified or null, it is assumed the
    /// dependency is in the current registry.
    registry: Option<String>,
    /// The URL of the index of the registry where this dependency is
    /// from as a string. If not specified or null, it is assumed the
    /// dependency is in the current registry.
    explicit_name_in_toml: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct CrateMeta {
    /// The name of the package.
    name: String,
    /// The version of the package being published.
    vers: String,
    /// Array of direct dependencies of the package.
    deps: Vec<Dependency>,
    /// Set of features defined for the package.
    /// Each feature maps to an array of features or dependencies it enables.
    /// Cargo does not impose limitations on feature names, but crates.io
    /// requires alphanumeric ASCII, `_` or `-` characters.
    features: BTreeMap<String, Vec<String>>,
    /// List of strings of the authors.
    /// May be empty. feather.io requires at least one entry.
    authors: Vec<String>,
    /// Description field from the manifest.
    /// feather.io requires at least some content.
    description: Option<String>,
    /// String of the URL to the website for this package's documentation
    documentation: Option<String>,
    /// String of the URL to the website for this package's home page.
    homepage: Option<String>,
    /// String of the content of the README file.
    readme: Option<String>,
    /// String of a relative path to a README file in the crate.
    readme_file: Option<String>,
    /// Array of strings of keywords for the package.
    keywords: Vec<String>,
    /// Array of strings of categories for the package.
    categories: Vec<String>,
    /// String of the license for the package.
    /// feather.io requires either `license` or `license_file` to be set.
    license: Option<String>,
    /// String of a relative path to a license file in the crate.
    license_file: Option<String>,
    /// String of the URL to the website for the source repository of this package.
    repository: Option<String>,
    /// Optional object of "status" badges. Each value is an object of
    /// arbitrary string to string mappings.
    /// crates.io has special interpretation of the format of the badges.
    badges: BTreeMap<String, BTreeMap<String, String>>,
    /// The `links` string value from the package's manifest, or null if not
    links: Option<String>,
}

impl CrateMeta {
    pub fn get_path(&self) -> path::PathBuf {
        let mut path = path::PathBuf::with_capacity(self.name.len() + 6);
        match self.name.len() {
            0 => panic!(),
            1 => path.push("1"),
            2 => path.push("2"),
            3 => path.push("3"),
            _ => {
                path.push(&self.name[..2]);
                path.push(&self.name[2..4]);
            }
        };
        path.push(&self.name);
        path
    }
}

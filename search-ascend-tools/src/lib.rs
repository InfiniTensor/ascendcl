#![doc = include_str!("../README.md")]
#![deny(warnings, unsafe_code, missing_docs)]

use std::{
    env::var_os,
    path::{Path, PathBuf},
};

/// Returns the path to the Ascend toolkit home directory.
#[inline]
pub fn find_ascend_toolkit_home() -> Option<PathBuf> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    if let Some(ascend) = var_os("ASCEND_TOOLKIT_HOME").map(PathBuf::from) {
        if verify(&ascend) {
            return Some(ascend);
        }
    }
    if let Some(mut ascend) = var_os("HOME").map(PathBuf::from) {
        ascend.push("Ascend");
        ascend.push("ascend-toolkit");
        ascend.push("latest");
        if verify(&ascend) {
            return Some(ascend);
        }
    }
    let ascend = Path::new("/usr/local/Ascend/ascend-toolkit/latest");
    if verify(ascend) {
        return Some(ascend.into());
    }
    None
}

#[inline(always)]
fn verify(ascend_toolkit_home: impl AsRef<Path>) -> bool {
    ascend_toolkit_home.as_ref().join("version.cfg").is_file()
}

/// Tells build scripts to re-run if the ASCEND_TOOLKIT_HOME environment variable changes.
#[inline(always)]
pub fn watch_env_var() {
    println!("cargo:rerun-if-env-changed=ASCEND_TOOLKIT_HOME");
}

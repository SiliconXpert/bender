// Copyright (c) 2024 ETH Zurich
// Michael Rogenmoser <michaero@iis.ee.ethz.ch>

use std;
use std::path::Path;

use crate::config::{Locked, LockedPackage, LockedSource, PrefixPaths};
use crate::error::*;

/// Read a lock file.
pub fn read_lockfile(path: &Path, root_dir: &Path) -> Result<Locked> {
    debugln!("read_lockfile: {:?}", path);
    use std::fs::File;
    let file = File::open(path)
        .map_err(|cause| Error::chain(format!("Cannot open lockfile {:?}.", path), cause))?;
    let locked_loaded: Result<Locked> = serde_yaml::from_reader(file)
        .map_err(|cause| Error::chain(format!("Syntax error in lockfile {:?}.", path), cause));
    // Make relative paths absolute
    Ok(Locked {
        packages: locked_loaded?
            .packages
            .iter()
            .map(|pack| {
                Ok(if let LockedSource::Path(path) = &pack.1.source {
                    (
                        pack.0.clone(),
                        LockedPackage {
                            revision: pack.1.revision.clone(),
                            version: pack.1.version.clone(),
                            source: LockedSource::Path(if path.is_relative() {
                                path.clone().prefix_paths(root_dir)?
                            } else {
                                path.clone()
                            }),
                            dependencies: pack.1.dependencies.clone(),
                        },
                    )
                } else {
                    (pack.0.clone(), pack.1.clone())
                })
            })
            .collect::<Result<_>>()?,
    })
}

/// Write a lock file.
pub fn write_lockfile(locked: &Locked, path: &Path, root_dir: &Path) -> Result<()> {
    debugln!("write_lockfile: {:?}", path);
    // Adapt paths within main repo to be relative
    let adapted_locked = Locked {
        packages: locked
            .packages
            .iter()
            .map(|pack| {
                if let LockedSource::Path(path) = &pack.1.source {
                    (
                        pack.0.clone(),
                        LockedPackage {
                            revision: pack.1.revision.clone(),
                            version: pack.1.version.clone(),
                            source: LockedSource::Path(
                                path.strip_prefix(root_dir).unwrap_or(path).to_path_buf(),
                            ),
                            dependencies: pack.1.dependencies.clone(),
                        },
                    )
                } else {
                    (pack.0.clone(), pack.1.clone())
                }
            })
            .collect(),
    };

    use std::fs::File;
    let file = File::create(path)
        .map_err(|cause| Error::chain(format!("Cannot create lockfile {:?}.", path), cause))?;
    serde_yaml::to_writer(file, &adapted_locked)
        .map_err(|cause| Error::chain(format!("Cannot write lockfile {:?}.", path), cause))?;
    Ok(())
}

// Copyright (c) 2017-2018 ETH Zurich
// Fabian Schuiki <fschuiki@iis.ee.ethz.ch>

//! The `packages` subcommand.

use std::io::Write;

use clap::{Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexSet;
use tabwriter::TabWriter;
use tokio::runtime::Runtime;

use crate::error::*;
use crate::sess::{DependencySource, Session, SessionIo};
use crate::src::SourceGroup;
use crate::target::TargetSpec;

/// Assemble the `packages` subcommand.
pub fn new() -> Command {
    Command::new("packages")
        .about("Information about the dependency graph")
        .arg(Arg::new("graph")
            .short('g')
            .long("graph")
            .num_args(0)
            .action(ArgAction::SetTrue)
            .help("Print the dependencies for each package")
        )
        .arg(Arg::new("flat")
            .short('f')
            .long("flat")
            .num_args(0)
            .action(ArgAction::SetTrue)
            .help("Do not group packages by topological rank")
            .long_help("Do not group packages by topological rank. If the `--graph` option is specified, print multiple lines per package, one for each dependency.")
        )
        .arg(Arg::new("version")
            .long("version")
            .num_args(0)
            .action(ArgAction::SetTrue)
            .help("Print the version of each package")
            .long_help("Print the version of each package. Implies --flat. More detailed information is available per dependency using the `parents` subcommand.")
        )
        .arg(Arg::new("targets")
            .long("targets")
            .num_args(0)
            .action(ArgAction::SetTrue)
            .help("Print the targets available for each package")
        )
}

/// Execute the `packages` subcommand.
pub fn run(sess: &Session, matches: &ArgMatches) -> Result<()> {
    let graph = matches.get_flag("graph");
    let flat = matches.get_flag("flat");
    let version = matches.get_flag("version");
    if graph && version {
        return Err(Error::new("cannot specify both --graph and --version"));
    }
    let targets = matches.get_flag("targets");
    if targets {
        if graph {
            return Err(Error::new("cannot specify both --graph and --targets"));
        }
        let rt = Runtime::new()?;
        let io = SessionIo::new(sess);
        let srcs = rt.block_on(io.sources(false, &[]))?;
        let mut target_str = String::from("");
        for pkgs in sess.packages().iter() {
            let pkg_names = pkgs.iter().map(|&id| sess.dependency_name(id));
            for pkg_name in pkg_names {
                target_str.push_str(&format!(
                    "{}:\t{:?}\n",
                    pkg_name,
                    srcs.filter_packages(&IndexSet::from([pkg_name.into()]))
                        .unwrap_or_else(|| SourceGroup {
                            package: Default::default(),
                            independent: true,
                            target: TargetSpec::Wildcard,
                            include_dirs: Default::default(),
                            export_incdirs: Default::default(),
                            defines: Default::default(),
                            libraries: Default::default(),
                            files: Default::default(),
                            dependencies: Default::default(),
                            version: None,
                        })
                        .get_avail_targets()
                ));
            }
        }
        target_str.push_str(&format!(
            "{}:\t{:?}\n",
            &sess.manifest.package.name,
            srcs.filter_packages(&IndexSet::from([sess.manifest.package.name.clone()]))
                .unwrap_or_else(|| SourceGroup {
                    package: Default::default(),
                    independent: true,
                    target: TargetSpec::Wildcard,
                    include_dirs: Default::default(),
                    export_incdirs: Default::default(),
                    defines: Default::default(),
                    libraries: Default::default(),
                    files: Default::default(),
                    dependencies: Default::default(),
                    version: None,
                })
                .get_avail_targets()
        ));
        let mut tw = TabWriter::new(vec![]);
        write!(&mut tw, "{}", target_str).unwrap();
        tw.flush().unwrap();
        print!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
    } else if graph {
        let mut graph_str = String::from("");
        for (&pkg, deps) in sess.graph().iter() {
            let pkg_name = sess.dependency_name(pkg);
            let dep_names = deps.iter().map(|&id| sess.dependency_name(id));
            if flat {
                // Print one line per dependency.
                for dep_name in dep_names {
                    graph_str.push_str(&format!("{}\t{}\n", pkg_name, dep_name));
                }
            } else {
                // Print all dependencies on one line.
                graph_str.push_str(&format!("{}\t", pkg_name));
                for (i, dep_name) in dep_names.enumerate() {
                    if i > 0 {
                        graph_str.push_str(&format!(" {}", dep_name));
                    } else {
                        graph_str.push_str(dep_name);
                    }
                }
                graph_str.push('\n');
            }
        }
        let mut tw = TabWriter::new(vec![]);
        write!(&mut tw, "{}", graph_str).unwrap();
        tw.flush().unwrap();
        print!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
    } else {
        let mut version_str = String::from("");
        for pkgs in sess.packages().iter() {
            let pkg_names = pkgs.iter().map(|&id| sess.dependency_name(id));
            let pkg_sources = pkgs.iter().map(|&id| sess.dependency(id));
            if version {
                for pkg_source in pkg_sources {
                    version_str.push_str(&format!(
                        "{}:\t{}\tat {}\t{}\n",
                        pkg_source.name,
                        match pkg_source.version {
                            Some(ref v) => format!("v{}", v),
                            None => "".to_string(),
                        },
                        pkg_source.source,
                        match pkg_source.source {
                            DependencySource::Path { .. } => " as path".to_string(),
                            DependencySource::Git(_) =>
                                format!(" with hash {}", pkg_source.version()),
                            _ => "".to_string(),
                        }
                    ));
                }
            } else if flat {
                // Print one line per package.
                for pkg_name in pkg_names {
                    println!("{}", pkg_name);
                }
            } else {
                // Print all packages per rank on one line.
                for (i, pkg_name) in pkg_names.enumerate() {
                    if i > 0 {
                        print!(" {}", pkg_name);
                    } else {
                        print!("{}", pkg_name);
                    }
                }
                println!();
            }
        }
        if version {
            let mut tw = TabWriter::new(vec![]);
            write!(&mut tw, "{}", version_str).unwrap();
            tw.flush().unwrap();
            print!("{}", String::from_utf8(tw.into_inner().unwrap()).unwrap());
        }
    }
    Ok(())
}

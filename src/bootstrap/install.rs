// Copyright 2016 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation of the install aspects of the compiler.
//!
//! This module is responsible for installing the standard library,
//! compiler, and documentation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf, Component};
use std::process::Command;

use Build;
use dist::{pkgname, sanitize_sh, tmpdir};

pub struct Installer<'a> {
    build: &'a Build,
    prefix: PathBuf,
    sysconfdir: PathBuf,
    docdir: PathBuf,
    bindir: PathBuf,
    libdir: PathBuf,
    mandir: PathBuf,
    empty_dir: PathBuf,
}

impl<'a> Drop for Installer<'a> {
    fn drop(&mut self) {
        t!(fs::remove_dir_all(&self.empty_dir));
    }
}

impl<'a> Installer<'a> {
    pub fn new(build: &'a Build) -> Installer<'a> {
        let prefix_default = PathBuf::from("/usr/local");
        let sysconfdir_default = PathBuf::from("/etc");
        let docdir_default = PathBuf::from("share/doc/rust");
        let bindir_default = PathBuf::from("bin");
        let libdir_default = PathBuf::from("lib");
        let mandir_default = PathBuf::from("share/man");
        let prefix = build.config.prefix.as_ref().unwrap_or(&prefix_default);
        let sysconfdir = build.config.sysconfdir.as_ref().unwrap_or(&sysconfdir_default);
        let docdir = build.config.docdir.as_ref().unwrap_or(&docdir_default);
        let bindir = build.config.bindir.as_ref().unwrap_or(&bindir_default);
        let libdir = build.config.libdir.as_ref().unwrap_or(&libdir_default);
        let mandir = build.config.mandir.as_ref().unwrap_or(&mandir_default);

        let sysconfdir = prefix.join(sysconfdir);
        let docdir = prefix.join(docdir);
        let bindir = prefix.join(bindir);
        let libdir = prefix.join(libdir);
        let mandir = prefix.join(mandir);

        let destdir = env::var_os("DESTDIR").map(PathBuf::from);

        let prefix = add_destdir(&prefix, &destdir);
        let sysconfdir = add_destdir(&sysconfdir, &destdir);
        let docdir = add_destdir(&docdir, &destdir);
        let bindir = add_destdir(&bindir, &destdir);
        let libdir = add_destdir(&libdir, &destdir);
        let mandir = add_destdir(&mandir, &destdir);

        let empty_dir = build.out.join("tmp/empty_dir");

        t!(fs::create_dir_all(&empty_dir));

        Installer {
            build,
            prefix,
            sysconfdir,
            docdir,
            bindir,
            libdir,
            mandir,
            empty_dir,
        }
    }

    pub fn install_docs(&self, stage: u32, host: &str) {
        self.install_sh("docs", "rust-docs", stage, Some(host));
    }

    pub fn install_std(&self, stage: u32) {
        for target in self.build.config.target.iter() {
            self.install_sh("std", "rust-std", stage, Some(target));
        }
    }

    pub fn install_cargo(&self, stage: u32, host: &str) {
        self.install_sh("cargo", "cargo", stage, Some(host));
    }

    pub fn install_rls(&self, stage: u32, host: &str) {
        self.install_sh("rls", "rls", stage, Some(host));
    }

    pub fn install_analysis(&self, stage: u32, host: &str) {
        self.install_sh("analysis", "rust-analysis", stage, Some(host));
    }

    pub fn install_src(&self, stage: u32) {
        self.install_sh("src", "rust-src", stage, None);
    }
    pub fn install_rustc(&self, stage: u32, host: &str) {
        self.install_sh("rustc", "rustc", stage, Some(host));
    }

    fn install_sh(&self, package: &str, name: &str, stage: u32, host: Option<&str>) {
        println!("Install {} stage{} ({:?})", package, stage, host);
        let package_name = if let Some(host) = host {
            format!("{}-{}", pkgname(self.build, name), host)
        } else {
            pkgname(self.build, name)
        };

        let mut cmd = Command::new("sh");
        cmd.current_dir(&self.empty_dir)
           .arg(sanitize_sh(&tmpdir(self.build).join(&package_name).join("install.sh")))
           .arg(format!("--prefix={}", sanitize_sh(&self.prefix)))
           .arg(format!("--sysconfdir={}", sanitize_sh(&self.sysconfdir)))
           .arg(format!("--docdir={}", sanitize_sh(&self.docdir)))
           .arg(format!("--bindir={}", sanitize_sh(&self.bindir)))
           .arg(format!("--libdir={}", sanitize_sh(&self.libdir)))
           .arg(format!("--mandir={}", sanitize_sh(&self.mandir)))
           .arg("--disable-ldconfig");
        self.build.run(&mut cmd);
    }
}

fn add_destdir(path: &Path, destdir: &Option<PathBuf>) -> PathBuf {
    let mut ret = match *destdir {
        Some(ref dest) => dest.clone(),
        None => return path.to_path_buf(),
    };
    for part in path.components() {
        match part {
            Component::Normal(s) => ret.push(s),
            _ => {}
        }
    }
    return ret
}

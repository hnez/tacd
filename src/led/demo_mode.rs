// This file is part of tacd, the LXA TAC system daemon
// Copyright (C) 2023 Pengutronix e.K.
//
// This program is free software; you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation; either version 2 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License along
// with this program; if not, write to the Free Software Foundation, Inc.,
// 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA.

use std::io::{Error, ErrorKind, Result};
use std::path::{Path, PathBuf};
use std::str::{from_utf8, FromStr};

use sysfs_class::{set_trait_method, trait_method};

const FILES_READ: &[(&str, &str)] = &[
    ("tac:green:out0/max_brightness", "1"),
    ("tac:green:out1/max_brightness", "1"),
    ("tac:green:dutpwr/max_brightness", "1"),
    ("rgb:status/max_brightness", "65535"),
    ("rgb:status/multi_index", "red green blue"),
];

pub trait SysClass: Sized {
    fn class() -> &'static str;
    unsafe fn from_path_unchecked(path: PathBuf) -> Self;
    fn path(&self) -> &Path;

    fn new(id: &str) -> Result<Self> {
        let inst = unsafe { Self::from_path_unchecked(id.into()) };
        Ok(inst)
    }

    fn read_file<P: AsRef<Path>>(&self, name: P) -> Result<String> {
        let path = self.path().join(name);
        let path = path.to_str().unwrap();

        FILES_READ
            .iter()
            .find(|(p, _)| p == &path)
            .map(|(_, d)| d.to_string())
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("{path} not found")))
    }

    fn parse_file<F: FromStr, P: AsRef<Path>>(&self, name: P) -> Result<F> {
        self.read_file(name)?
            .parse()
            .map_err(|_| Error::new(ErrorKind::InvalidData, "too bad"))
    }

    fn write_file<P: AsRef<Path>, S: AsRef<[u8]>>(&self, name: P, data: S) -> Result<()> {
        let path = self.path().join(name);
        let path = path.to_str().unwrap();
        let data = from_utf8(data.as_ref()).unwrap();

        log::info!("LED: Write {} to {}", data, path);

        Ok(())
    }
}

pub trait Brightness {
    fn brightness(&self) -> Result<u64>;
    fn max_brightness(&self) -> Result<u64>;
    fn set_brightness(&self, val: u64) -> Result<()>;
}

pub struct Leds {
    path: PathBuf,
}

impl SysClass for Leds {
    fn class() -> &'static str {
        "leds"
    }

    unsafe fn from_path_unchecked(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Brightness for Leds {
    trait_method!(brightness parse_file u64);
    trait_method!(max_brightness parse_file u64);
    set_trait_method!("brightness", set_brightness u64);
}

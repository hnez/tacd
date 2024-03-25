// This file is part of tacd, the LXA TAC system daemon
// Copyright (C) 2022 Pengutronix e.K.
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

use anyhow::{bail, Result};
use async_std::sync::Arc;
use nix::sys::utsname::uname;
use serde::{Deserialize, Serialize};

use crate::broker::{BrokerBuilder, Topic};

#[cfg(feature = "demo_mode")]
mod read_dt_props {
    const DEMO_DATA_STR: &[(&str, &str)] = &[
        (
            "compatible",
            "lxa,stm32mp153c-tac-gen3\0oct,stm32mp15xx-osd32\0st,stm32mp153",
        ),
        ("chosen/barebox-version", "barebox-2022.11.0-20221121-1"),
        (
            "chosen/baseboard-factory-data/pcba-hardware-release",
            "lxatac-S01-R03-B02-C00",
        ),
        (
            "chosen/powerboard-factory-data/pcba-hardware-release",
            "lxatac-S05-R03-V01-C00",
        ),
    ];

    const DEMO_DATA_NUM: &[(&str, u32)] = &[
        ("chosen/baseboard-factory-data/modification", 0),
        (
            "chosen/baseboard-factory-data/factory-timestamp",
            1678086417,
        ),
        ("chosen/powerboard-factory-data/modification", 0),
        (
            "chosen/powerboard-factory-data/factory-timestamp",
            1678086418,
        ),
    ];

    pub fn read_dt_property(path: &str) -> String {
        let (_, content) = DEMO_DATA_STR.iter().find(|(p, _)| *p == path).unwrap();

        content.to_string()
    }

    pub fn read_dt_property_u32(path: &str) -> u32 {
        DEMO_DATA_NUM.iter().find(|(p, _)| *p == path).unwrap().1
    }
}

#[cfg(not(feature = "demo_mode"))]
mod read_dt_props {
    use std::fs::read;
    use std::str::from_utf8;

    const DT_BASE: &str = "/sys/firmware/devicetree/base/";

    pub fn read_dt_property(path: &str) -> String {
        let bytes = read([DT_BASE, path].join("/")).unwrap();
        from_utf8(bytes.strip_suffix(&[0]).unwrap())
            .unwrap()
            .to_string()
    }

    pub fn read_dt_property_u32(path: &str) -> u32 {
        read_dt_property(path).parse().unwrap()
    }
}

use read_dt_props::{read_dt_property, read_dt_property_u32};

#[derive(Serialize, Deserialize)]
pub struct Uname {
    pub sysname: String,
    pub nodename: String,
    pub release: String,
    pub version: String,
    pub machine: String,
}

impl Uname {
    fn get() -> Self {
        let uts = uname().unwrap();

        Self {
            sysname: uts.sysname().to_str().unwrap().to_string(),
            nodename: uts.nodename().to_str().unwrap().to_string(),
            release: uts.release().to_str().unwrap().to_string(),
            version: uts.version().to_str().unwrap().to_string(),
            machine: uts.machine().to_str().unwrap().to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Barebox {
    pub version: String,
    pub baseboard_release: String,
    pub powerboard_release: String,
    pub baseboard_timestamp: u32,
    pub powerboard_timestamp: u32,
}

impl Barebox {
    fn get() -> Self {
        // Get info from devicetree chosen
        Self {
            version: read_dt_property("chosen/barebox-version"),
            baseboard_release: {
                let template =
                    read_dt_property("chosen/baseboard-factory-data/pcba-hardware-release");
                let changeset = read_dt_property_u32("chosen/baseboard-factory-data/modification");

                template.replace("-C??", &format!("-C{changeset:02}"))
            },
            powerboard_release: {
                let template =
                    read_dt_property("chosen/powerboard-factory-data/pcba-hardware-release");
                let changeset = read_dt_property_u32("chosen/powerboard-factory-data/modification");

                template.replace("-C??", &format!("-C{changeset:02}"))
            },
            baseboard_timestamp: {
                read_dt_property_u32("chosen/baseboard-factory-data/factory-timestamp")
            },
            powerboard_timestamp: {
                read_dt_property_u32("chosen/powerboard-factory-data/factory-timestamp")
            },
        }
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum HardwareGeneration {
    Gen1,
    Gen2,
    Gen3,
}

impl HardwareGeneration {
    pub fn get() -> Result<Self> {
        let compatible = read_dt_property("compatible");

        // The compatible property consists of strings separated by NUL bytes.
        // We are interested in the first of these strings.
        let device = compatible.split('\0').next().unwrap_or("<empty>");

        match device {
            "lxa,stm32mp157c-tac-gen1" => Ok(Self::Gen1),
            "lxa,stm32mp157c-tac-gen2" => Ok(Self::Gen2),
            "lxa,stm32mp153c-tac-gen3" => Ok(Self::Gen3),
            gen => bail!("Running on unknown LXA TAC hardware generation \"{gen}\""),
        }
    }
}

pub struct System {
    #[allow(dead_code)]
    pub uname: Arc<Topic<Arc<Uname>>>,
    #[allow(dead_code)]
    pub barebox: Arc<Topic<Arc<Barebox>>>,
    #[allow(dead_code)]
    pub tacd_version: Arc<Topic<String>>,
    #[allow(dead_code)]
    pub hardware_generation: Arc<Topic<HardwareGeneration>>,
}

impl System {
    pub fn new(bb: &mut BrokerBuilder, hardware_generation: HardwareGeneration) -> Self {
        let version = env!("VERSION_STRING").to_string();

        Self {
            uname: bb.topic_ro("/v1/tac/info/uname", Some(Arc::new(Uname::get()))),
            barebox: bb.topic_ro("/v1/tac/info/bootloader", Some(Arc::new(Barebox::get()))),
            tacd_version: bb.topic_ro("/v1/tac/info/tacd/version", Some(version)),
            hardware_generation: bb.topic_ro(
                "/v1/tac/info/hardware_generation",
                Some(hardware_generation),
            ),
        }
    }
}

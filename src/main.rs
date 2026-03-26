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
// with this library; if not, see <https://www.gnu.org/licenses/>.

use std::time::Duration;

use anyhow::{Error, Result};
use async_std::task::sleep;
use log::{error, info};

mod adc;
mod broker;
mod measurement;
mod system;
mod watched_tasks;

use adc::Adc;
use broker::BrokerBuilder;
use system::HardwareGeneration;
use watched_tasks::WatchedTasksBuilder;

#[async_std::main]
async fn main() -> Result<()> {
    env_logger::init();

    // The tacd spawns a couple of async tasks that should run as long as
    // the tacd runs and if any one fails the tacd should stop.
    // These tasks are spawned via the watched task builder.
    let mut wtb = WatchedTasksBuilder::new();

    // The BrokerBuilder collects topics that should be exported via the
    // MQTT/REST APIs.
    // The topics are also used to pass around data inside the tacd.
    let mut bb = BrokerBuilder::new();

    let hardware_generation = HardwareGeneration::get()?;
    let adc = Adc::new(&mut bb, &mut wtb, hardware_generation).await?;

    wtb.spawn_task("print-adc", async move {
        let mut start = None;

        loop {
            let meas = adc
                .usb_host_curr
                .fast
                .get()
                .map_err(|_| Error::msg("Adc Error"))?;

            let ts = meas.ts.as_instant();

            let ms_since_start = ts.duration_since(*start.get_or_insert(ts)).as_millis();

            println!(
                "{:10}ms | {:15}A | {:10}",
                ms_since_start,
                meas.value,
                meas.raw.unwrap_or(0)
            );

            sleep(Duration::from_millis(500)).await;
        }
    })?;

    wtb.watch().await
}

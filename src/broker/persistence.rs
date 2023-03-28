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

use std::fs::{create_dir, rename, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use anyhow::{anyhow, Result};
use async_std::channel::{unbounded, Receiver};
use async_std::prelude::*;
use async_std::sync::Arc;
use async_std::task::spawn;
use log::info;

use super::{AnyTopic, TopicName};

#[cfg(feature = "demo_mode")]
const PERSISTENCE_PATH: &str = "demo_files/srv/tacd/persistent_topics";

#[cfg(not(feature = "demo_mode"))]
const PERSISTENCE_PATH: &str = "/srv/tacd/persistent_topics";

async fn load(topics: &[Arc<dyn AnyTopic>]) -> Result<()> {
    let path = Path::new(PERSISTENCE_PATH);

    if !path.is_file() {
        info!(
            "Persistence file at \"{}\" does not yet exist. Using defaults",
            PERSISTENCE_PATH
        );
        return Ok(());
    }

    let mut fd = BufReader::new(File::open(path)?);

    loop {
        let (topic_name, value) = {
            let mut topic_name = Vec::new();
            let mut value = Vec::new();

            fd.read_until(b' ', &mut topic_name)?;

            if topic_name.is_empty() {
                break Ok(());
            }

            if topic_name.pop() != Some(b' ') {
                break Err(anyhow!("Persistent topic file ended unexpectedly"));
            }

            fd.read_until(b'\n', &mut value)?;

            if value.last() == Some(&b'\n') {
                value.pop();
            }

            (topic_name, value)
        };

        let topic = topics
            .iter()
            .find(|t| t.persistent() && t.path().as_bytes() == topic_name)
            .ok_or_else(|| {
                let topic_name = String::from_utf8_lossy(&topic_name);
                anyhow!("Could not find persistent topic \"{topic_name}\"")
            })?;

        topic.set_from_bytes(&value).await?
    }
}

async fn save_on_change(
    topics: Arc<Vec<Arc<dyn AnyTopic>>>,
    mut change_ev: Receiver<(TopicName, Arc<[u8]>)>,
) -> Result<()> {
    while let Some((topic_name, _)) = change_ev.next().await {
        let topic_name = String::from_utf8_lossy(topic_name.as_bytes());

        info!(
            "Persistent topic \"{}\" has changed. Saving to disk",
            topic_name
        );

        let path = Path::new(PERSISTENCE_PATH);
        let parent = path.parent().unwrap();

        let path_tmp = {
            let mut path_tmp = path.to_owned();
            assert!(path_tmp.set_extension("tmp"));
            path_tmp
        };

        if !parent.exists() {
            create_dir(parent)?;
        }

        {
            let mut fd = File::create(&path_tmp)?;

            for topic in topics.iter().filter(|t| t.persistent()) {
                if let Some(value) = topic.try_get_as_bytes().await {
                    fd.write_all(topic.path().as_bytes())?;
                    fd.write_all(b" ")?;
                    fd.write_all(&value)?;
                    fd.write_all(b"\n")?;
                }
            }

            fd.sync_all()?
        }

        rename(path_tmp, path)?
    }

    Ok(())
}

pub async fn register(topics: Arc<Vec<Arc<dyn AnyTopic>>>) {
    load(&topics).await.unwrap();

    let (tx, rx) = unbounded();

    for topic in topics.iter().filter(|t| t.persistent()).cloned() {
        topic.subscribe_as_bytes(tx.clone(), false).await;
    }

    spawn(async move { save_on_change(topics, rx).await.unwrap() });
}

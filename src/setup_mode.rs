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

use std::fs::{create_dir_all, read, write};
use std::io::ErrorKind;
use std::path::Path;

use async_std::sync::Arc;
use tide::{http::mime, Request, Response, Server};

use crate::broker::{BrokerBuilder, Topic};

#[cfg(feature = "demo_mode")]
const AUTHORIZED_KEYS_PATH: &str = "demo_files/home/root/ssh/authorized_keys";

#[cfg(not(feature = "demo_mode"))]
const AUTHORIZED_KEYS_PATH: &str = "/home/root/.ssh/authorized_keys";

pub struct SetupMode {
    pub setup_mode: Arc<Topic<bool>>,
}

impl SetupMode {
    fn expose_file_conditionally(
        &self,
        server: &mut Server<()>,
        fs_path: &'static str,
        web_path: &str,
    ) {
        let setup_mode_task = self.setup_mode.clone();
        server.at(web_path).put(move |mut req: Request<()>| {
            let setup_mode = setup_mode_task.clone();

            async move {
                let res = if setup_mode.get().await {
                    let fs_path = Path::new(fs_path);
                    let parent = fs_path.parent().unwrap();

                    if !parent.exists() {
                        create_dir_all(parent)?;
                    }

                    let content = req.body_bytes().await?;
                    write(fs_path, content)?;

                    Response::new(204)
                } else {
                    Response::builder(403)
                        .body("This file may only be written in setup mode")
                        .content_type(mime::PLAIN)
                        .build()
                };

                Ok(res)
            }
        });

        let setup_mode_task = self.setup_mode.clone();
        server.at(web_path).get(move |_| {
            let setup_mode = setup_mode_task.clone();

            async move {
                let res = if setup_mode.get().await {
                    match read(fs_path) {
                        Ok(content) => Response::builder(200)
                            .body(content)
                            .content_type(mime::PLAIN)
                            .build(),
                        Err(e) => {
                            let status = match e.kind() {
                                ErrorKind::NotFound => 404,
                                _ => 500,
                            };
                            Response::builder(status)
                                .body("Failed to read file")
                                .content_type(mime::PLAIN)
                                .build()
                        }
                    }
                } else {
                    Response::builder(403)
                        .body("This file may only be read in setup mode")
                        .content_type(mime::PLAIN)
                        .build()
                };

                Ok(res)
            }
        });
    }

    pub fn new(bb: &mut BrokerBuilder, server: &mut Server<()>) -> Self {
        let this = Self {
            setup_mode: bb.topic("/v1/tac/setup_mode", true, false, true, Some(true), 1),
        };

        this.expose_file_conditionally(server, AUTHORIZED_KEYS_PATH, "/v1/tac/ssh/authorized_keys");

        this
    }
}

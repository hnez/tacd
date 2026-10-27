// This file is part of tacd, the LXA TAC system daemon
// Copyright (C) 2026 Pengutronix e.K.
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

use std::time::{Duration, SystemTime};

use super::CapabilitySet;
use serde::Serialize;

mod utils;
use utils::*;

const FALLBACK_VALIDITY: Duration = Duration::from_mins(15);

#[derive(PartialEq, Eq, Debug, Clone, Serialize)]
pub struct Token {
    #[serde(serialize_with = "serialize_system_time")]
    pub generated: SystemTime,
    #[serde(serialize_with = "serialize_system_time")]
    pub expires: SystemTime,
    pub capabilities_read: CapabilitySet,
    pub capabilities_write: CapabilitySet,
}

pub struct TokenStore {}

impl Token {
    pub fn new(validity: Duration) -> Self {
        let now = SystemTime::now();

        Self {
            generated: whole_second(now),
            expires: whole_second(now + validity),
            capabilities_read: CapabilitySet::empty(),
            capabilities_write: CapabilitySet::empty(),
        }
    }
}

impl TokenStore {
    pub fn open() -> std::io::Result<Self> {
        Ok(Self {})
    }

    pub fn fallback_token(&self, is_browser: bool, is_same_origin: bool) -> Token {
        let mut token = Token::new(FALLBACK_VALIDITY);

        if !is_browser || is_same_origin {
            // Set all capabilities for requests that do not come from a browser,
            // or for requests that come from a browser but have the CSRF headers
            // set correctly.
            token.capabilities_read.set_all();
            token.capabilities_write.set_all();
        }

        token
    }
}

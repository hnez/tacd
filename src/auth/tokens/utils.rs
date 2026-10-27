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

use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub(super) fn to_unix_ts(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH).unwrap().as_secs()
}

pub(super) fn from_unix_ts(ts: u64) -> SystemTime {
    UNIX_EPOCH + Duration::from_secs(ts)
}

pub(super) fn whole_second(time: SystemTime) -> SystemTime {
    from_unix_ts(to_unix_ts(time))
}

use std::io::{Error, ErrorKind, Result};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use async_std::future::pending;
use async_std::sync::Weak;
use async_std::task::sleep;

#[cfg(any(test, feature = "stub_out_root"))]
mod sd {
    use std::io::Result;

    pub const STATE_READY: () = ();
    pub const STATE_WATCHDOG: () = ();

    pub fn notify<I>(_: bool, _: I) -> Result<bool> {
        println!("Watchdog tick");
        Ok(true)
    }

    pub fn watchdog_enabled(_: bool) -> Result<u64> {
        Ok(5_000_000)
    }
}

#[cfg(not(any(test, feature = "stub_out_root")))]
mod sd {
    pub use systemd::daemon::*;
}

use sd::{notify, watchdog_enabled, STATE_READY, STATE_WATCHDOG};

pub struct Watchdog {
    dut_power_tick: Weak<AtomicU32>,
}

impl Watchdog {
    pub fn new(dut_power_tick: Weak<AtomicU32>) -> Self {
        Self {
            dut_power_tick: dut_power_tick,
        }
    }

    /// Make sure the following things are still somewhat working:
    ///
    /// - async_std runtime - otherwise the future would not be polled
    /// - dut_pwr thread - otherwise the tick would not be incremented
    /// - adc thread - if the adc values are too old dut_pwr_thread will
    ///   not increment the tick.
    pub async fn keep_fed(self) -> Result<()> {
        let interval = {
            let micros = watchdog_enabled(false).unwrap_or(0);

            if micros == 0 {
                eprintln!("Watchdog not requested. Disabling");

                // Wait forever, as returning from this function terminated the program
                let () = pending().await;
            }

            Duration::from_micros(micros) / 2
        };

        notify(false, [(STATE_READY, "1")].iter())?;

        let mut prev = self
            .dut_power_tick
            .upgrade()
            .map(|v| v.load(Ordering::Relaxed));

        loop {
            sleep(interval).await;

            let curr = self
                .dut_power_tick
                .upgrade()
                .map(|v| v.load(Ordering::Relaxed));

            let (p, c) = prev.zip(curr).unwrap_or((0, 0));

            // Fail if the power thread tick did not increment in the meantime or if
            // the DutPwrThread was dropped (e.g. dut_power_tick could not be upgraded
            // from Weak<_> to Arc<_>).
            if p == c {
                eprintln!("Power Thread has stalled. Will trigger watchdog.");

                notify(false, [(STATE_WATCHDOG, "trigger")].iter())?;

                break Err(Error::new(
                    ErrorKind::TimedOut,
                    "Power Thread stalled for too long",
                ));
            }

            notify(false, [(STATE_WATCHDOG, "1")].iter())?;
            prev = curr;
        }
    }
}

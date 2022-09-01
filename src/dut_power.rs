use std::convert::TryFrom;
use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use async_std::prelude::*;
use async_std::sync::{Arc, Weak};
use async_std::task;
use gpio_cdev::{LineHandle, LineRequestFlags};
use serde::{Deserialize, Serialize};
use thread_priority::*;

use crate::adc::AdcChannel;
use crate::broker::{BrokerBuilder, Topic};
use crate::digital_io::find_line;

const MAX_AGE: Duration = Duration::from_millis(300);
const THREAD_INTERVAL: Duration = Duration::from_millis(100);
const TASK_INTERVAL: Duration = Duration::from_millis(200);
const MAX_CURRENT: f32 = 5.0;
const MAX_VOLTAGE: f32 = 48.0;
const MIN_VOLTAGE: f32 = -1.0;

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum OutputRequest {
    Idle,
    On,
    Off,
    OffDischarge,
}

impl From<u8> for OutputRequest {
    fn from(val: u8) -> Self {
        if val == (OutputRequest::Idle as u8) {
            return OutputRequest::Idle;
        }

        if val == (OutputRequest::On as u8) {
            return OutputRequest::On;
        }

        if val == (OutputRequest::Off as u8) {
            return OutputRequest::Off;
        }

        if val == (OutputRequest::OffDischarge as u8) {
            return OutputRequest::OffDischarge;
        }

        panic!()
    }
}

#[derive(PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum OutputState {
    On,
    Off,
    OffDischarge,
    InvertedPolarity,
    OverCurrent,
    OverVoltage,
    RealtimeViolation,
}

impl From<u8> for OutputState {
    fn from(val: u8) -> Self {
        if val == (OutputState::Off as u8) {
            return OutputState::Off;
        }

        if val == (OutputState::OffDischarge as u8) {
            return OutputState::OffDischarge;
        }

        if val == (OutputState::On as u8) {
            return OutputState::On;
        }

        if val == (OutputState::InvertedPolarity as u8) {
            return OutputState::InvertedPolarity;
        }

        if val == (OutputState::OverCurrent as u8) {
            return OutputState::OverCurrent;
        }

        if val == (OutputState::OverVoltage as u8) {
            return OutputState::OverVoltage;
        }

        if val == (OutputState::RealtimeViolation as u8) {
            return OutputState::RealtimeViolation;
        }

        panic!()
    }
}

pub struct DutPwrThread {
    pub request: Arc<Topic<OutputRequest>>,
    pub state: Arc<Topic<OutputState>>,
    tick: Arc<AtomicU32>,
    join: Option<thread::JoinHandle<()>>,
}

/// Bring the outputs into a fail safe mode
fn fail(
    reason: OutputState,
    pwr_line: &LineHandle,
    discharge_line: &LineHandle,
    fail_state: &AtomicU8,
) {
    pwr_line.set_value(1).unwrap();
    discharge_line.set_value(1).unwrap();
    fail_state.store(reason as u8, Ordering::Relaxed);
}

impl DutPwrThread {
    pub fn new(bb: &mut BrokerBuilder, pwr_volt: AdcChannel, pwr_curr: AdcChannel) -> Self {
        let tick = Arc::new(AtomicU32::new(0));
        let tick_weak = Arc::downgrade(&tick);

        let request = Arc::new(AtomicU8::new(OutputRequest::Idle as u8));
        let state = Arc::new(AtomicU8::new(OutputState::Off as u8));

        // The request and state topic use the same external path, this way one
        // can e.g. publish "On" to the topic and be sure that the output is
        // actually on once a corresponding publish is received from the broker,
        // as it has done the full round trip through the realtime power thread
        // and is not just a copy of the received command.
        let request_topic = bb.topic_wo::<OutputRequest>("/v1/dut/power/status", None);
        let state_topic = bb.topic_ro::<OutputState>("/v1/dut/power/status", None);

        // Requests come from the broker framework and are placed into an atomic
        // request variable read by the thread.
        let request_task = request.clone();
        let request_topic_task = request_topic.clone();
        task::spawn(async move {
            let (mut request_stream, _) = request_topic_task.subscribe_unbounded().await;

            while let Some(req) = request_stream.next().await {
                request_task.store(*req as u8, Ordering::Relaxed);
            }
        });

        // State information comes from the thread in the form of an atomic
        // variable and is forwarded to the broker framework.
        let state_task = state.clone();
        let state_topic_task = state_topic.clone();
        task::spawn(async move {
            let mut prev_state: Option<OutputState> = None;

            loop {
                task::sleep(TASK_INTERVAL).await;

                let state = state_task.load(Ordering::Relaxed).into();

                if prev_state.map(|prev| prev != state).unwrap_or(true) {
                    state_topic_task.set(state).await;
                    prev_state = Some(state);
                }
            }
        });

        // Spawn a high priority thread that handles the power status
        // in a realtimey fashion.
        let join = thread::Builder::new()
            .name("tacd power".into())
            .spawn(move || {
                let pwr_line = find_line("IO0")
                    .unwrap()
                    .request(LineRequestFlags::OUTPUT, 0, "tacd")
                    .unwrap();

                let discharge_line = find_line("IO1")
                    .unwrap()
                    .request(LineRequestFlags::OUTPUT, 0, "tacd")
                    .unwrap();

                set_thread_priority_and_policy(
                    thread_native_id(),
                    ThreadPriority::Crossplatform(ThreadPriorityValue::try_from(10).unwrap()),
                    ThreadSchedulePolicy::Realtime(RealtimeThreadSchedulePolicy::Fifo),
                )
                .unwrap();

                let mut last_ts: Option<Instant> = None;

                // Run as long as there is a strong reference to `tick`.
                // As tick is a private memeber of the struct this is equivalent
                // to running as long as the DutPwrThread was not dropped.
                while let Some(tick) = tick_weak.upgrade() {
                    thread::sleep(THREAD_INTERVAL);

                    // Get new voltage and current readings while making sure
                    // that they are not stale
                    let (volt, curr) = loop {
                        let feedback = pwr_volt
                            .fast
                            .try_get_multiple([&pwr_volt.fast, &pwr_curr.fast]);

                        if let Some((new_ts, _)) = feedback {
                            last_ts = Some(new_ts);
                        }

                        let too_old = last_ts
                            .map(|ts| Instant::now().duration_since(ts) > MAX_AGE)
                            .unwrap_or(false);

                        if too_old {
                            fail(
                                OutputState::RealtimeViolation,
                                &pwr_line,
                                &discharge_line,
                                &state,
                            );
                        } else {
                            // We have a fresh ADC value. Signal "everythin is well"
                            // to the watchdog task.
                            tick.fetch_add(1, Ordering::Relaxed);
                        }

                        if let Some((_, [volt, curr])) = feedback {
                            break (volt, curr);
                        }
                    };

                    // Don't even look at the requests if there is an ongoing
                    // overvoltage condition. Instead turn the output off and
                    // go back to measuring.
                    if volt > MAX_VOLTAGE {
                        fail(OutputState::OverVoltage, &pwr_line, &discharge_line, &state);

                        continue;
                    }

                    // Don't even look at the requests if there is an ongoin
                    // polarity inversion. Turn off, go back to start, do not
                    // collect $200.
                    if volt < MIN_VOLTAGE {
                        fail(
                            OutputState::InvertedPolarity,
                            &pwr_line,
                            &discharge_line,
                            &state,
                        );

                        continue;
                    }

                    // Don't even look at the requests if there is an ongoin
                    // overcurrent condition.
                    if curr > MAX_CURRENT {
                        fail(OutputState::OverCurrent, &pwr_line, &discharge_line, &state);

                        continue;
                    }

                    // There is no ongoing fault condition, so we could e.g. turn
                    // the output on if requested.
                    match request
                        .swap(OutputRequest::Idle as u8, Ordering::Relaxed)
                        .into()
                    {
                        OutputRequest::Idle => {}
                        OutputRequest::On => {
                            discharge_line.set_value(1).unwrap();
                            pwr_line.set_value(0).unwrap();
                            state.store(OutputState::On as u8, Ordering::Relaxed);
                        }
                        OutputRequest::Off => {
                            discharge_line.set_value(1).unwrap();
                            pwr_line.set_value(1).unwrap();
                            state.store(OutputState::Off as u8, Ordering::Relaxed);
                        }
                        OutputRequest::OffDischarge => {
                            discharge_line.set_value(0).unwrap();
                            pwr_line.set_value(1).unwrap();
                            state.store(OutputState::OffDischarge as u8, Ordering::Relaxed);
                        }
                    }
                }

                // Make sure to enter fail safe mode before leaving the thread
                fail(OutputState::Off, &pwr_line, &discharge_line, &state);
            })
            .unwrap();

        Self {
            request: request_topic,
            state: state_topic,
            tick,
            join: Some(join),
        }
    }

    pub fn tick(&self) -> Weak<AtomicU32> {
        Arc::downgrade(&self.tick)
    }
}

impl Drop for DutPwrThread {
    fn drop(&mut self) {
        self.join.take().unwrap().join().unwrap()
    }
}

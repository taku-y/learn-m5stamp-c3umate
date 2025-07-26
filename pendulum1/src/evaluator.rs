use crate::env::PendulumEnv;
use anyhow::Result;
use border_core::{Env, Policy};
use esp_idf_svc::hal::{
    delay::FreeRtos,
    timer::{TimerDriver, TIMER00},
};
use std::sync::atomic::Ordering;

/// Evaluate given policy with PendulumEnv.
///
/// The sampling frequency is 50Hz, which means that the policy is called every 20ms.
pub struct PendulumEvaluator<'d> {
    timer: TimerDriver<'d>,
}

impl PendulumEvaluator<'_> {
    pub fn new(timer: TIMER00) -> Self {
        let config = esp_idf_svc::hal::timer::config::Config::new();
        PendulumEvaluator {
            timer: TimerDriver::new(timer, &config).expect("Failed to create timer driver"),
        }
    }

    pub fn evaluate<'d, P: Policy<PendulumEnv<'d>>>(
        &mut self,
        policy: &mut P,
        env: &mut PendulumEnv<'d>,
        _steps: usize,
    ) -> Result<()> {
        let mut obs = env.reset(None)?;

        loop {
            // Reset timer
            let _ = self.timer.set_counter(0);

            // Proceed with the environment step
            let (step, _) = env.step(&policy.sample(&obs));
            obs = step.obs.clone();

            // Break if the state changes to something other than running an episode
            let state = crate::STATE.load(Ordering::Relaxed);
            if state == crate::IDLE {
                break;
            } else if state ==  crate::MANUAL_POLICY_START {
                crate::set_state(crate::MANUAL_POLICY);
                break;
            }

            // Wait for 20ms to simulate the 50Hz sampling frequency
            let wait_time = 20 - self.timer.counter().unwrap() / 1000; // Convert to milliseconds
            if wait_time > 0 {
                FreeRtos::delay_ms(wait_time as _);
            } else {
                log::warn!("Timer overflow, skipping wait time");
            }
        }

        Ok(())
    }
}

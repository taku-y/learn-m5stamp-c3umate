use crate::env::{PendulumEnv, PendulumEnvAct, PendulumEnvObs};
use border_core::Policy;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::{
    adc::oneshot::{config::AdcChannelConfig, AdcChannelDriver, AdcDriver},
    gpio::ADCPin,
    peripheral::Peripheral,
};
use std::sync::atomic::Ordering;

pub struct ManualPolicy<T>
where
    T: ADCPin,
{
    adc_pin: AdcChannelDriver<'static, T, AdcDriver<'static, T::Adc>>,
    min_limit: u16,
    max_limit: u16,
}

impl<T> ManualPolicy<T>
where
    T: ADCPin,
{
    pub fn new(
        adc: impl Peripheral<P = T::Adc> + 'static,
        pin: impl Peripheral<P = T> + 'static,
    ) -> Self {
        let adc_driver = AdcDriver::new(adc).unwrap();
        let config = AdcChannelConfig {
            attenuation: esp_idf_svc::hal::adc::attenuation::DB_11,
            ..Default::default()
        };
        let adc_pin = AdcChannelDriver::new(adc_driver, pin, &config)
            .expect("Failed to create ADC channel driver");

        ManualPolicy {
            adc_pin,
            min_limit: 0,
            max_limit: 2048,
        }
    }

    pub fn take_potentiometer_value(&mut self, current_state: u8) -> u16 {
        let mut value = 0;

        loop {
            FreeRtos::delay_ms(100);

            match self.adc_pin.read() {
                Ok(value_) => {
                    value = value_;
                    log::info!("Potentiometer value: {}", value);
                }
                Err(e) => {
                    log::error!("Failed to read ADC value: {:?}", e);
                }
            }

            if get_state() != current_state {
                return value; // Return a default value or handle cancellation as needed
            }
        }
    }

    pub fn set_min_limit(&mut self, min_limit: u16) {
        self.min_limit = min_limit;
    }

    pub fn set_max_limit(&mut self, max_limit: u16) {
        self.max_limit = max_limit;
    }
}

impl<'d, T> Policy<PendulumEnv<'d>> for ManualPolicy<T>
where
    T: ADCPin,
{
    fn sample(&mut self, _obs: &PendulumEnvObs) -> PendulumEnvAct {
        let raw = self.adc_pin.read().unwrap();
        map(raw, self.min_limit, self.max_limit).into()
    }
}

fn get_state() -> u8 {
    crate::STATE.load(Ordering::Relaxed)
}

// Return the mapped value, taking between -1.0 and 1.0
//
// min_limit and max_limit are the limits of the raw value.
fn map(raw: u16, min_limit: u16, max_limit: u16) -> f32 {
    if raw < min_limit {
        return -1.0;
    } else if raw > max_limit {
        return 1.0;
    }
    let range = max_limit - min_limit;
    let normalized = (raw - min_limit) as f32 / range as f32;
    normalized * 2.0 - 1.0 // Scale to [-1, 1]
}

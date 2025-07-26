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

        ManualPolicy { adc_pin }
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
}

impl<'d, T> Policy<PendulumEnv<'d>> for ManualPolicy<T>
where
    T: ADCPin,
{
    fn sample(&mut self, _obs: &PendulumEnvObs) -> PendulumEnvAct {
        todo!("Implement manual control logic here");
    }
}

fn get_state() -> u8 {
    crate::STATE.load(Ordering::Relaxed)
}

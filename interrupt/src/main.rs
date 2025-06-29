use std::sync::atomic::{AtomicU8, Ordering};

use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::delay::FreeRtos;
use esp_idf_svc::hal::peripheral::Peripheral;
use esp_idf_svc::hal::peripherals::Peripherals;

static STATE: AtomicU8 = AtomicU8::new(0);

fn gpio_interrupt_handler1() {
    if STATE.load(Ordering::Relaxed) == 0 {
        STATE.store(1, Ordering::Relaxed);
    } else {
        STATE.store(0, Ordering::Relaxed);
    }
}

fn gpio_interrupt_handler2() {
    STATE.store(2, Ordering::Relaxed);
}

fn gpio_interrupt_handler3() {
    STATE.store(3, Ordering::Relaxed);
}

fn gpio_interrupt_handler4() {
    STATE.store(4, Ordering::Relaxed);
}

fn state1() {
    loop {
        // Sleep for 1 second
        FreeRtos::delay_ms(1000);
        log::info!("state1");
        
        if STATE.load(Ordering::Relaxed) != 1 {
            break;
        }
    }
}

fn polling() {
    log::info!("polling: {}", STATE.load(Ordering::Relaxed));
    match STATE.load(Ordering::Relaxed) {
        // Idle
        0 => FreeRtos::delay_ms(50),
        // Run an episode
        1 => state1(),
        // Send episode data to the server
        2 => {
            FreeRtos::delay_ms(1000);
            STATE.store(0, Ordering::Relaxed);
        },
        // Receive model parameters from the server
        3 => {
            FreeRtos::delay_ms(1000);
            STATE.store(0, Ordering::Relaxed);
        },
        // Clear the episode data
        4 => {
            FreeRtos::delay_ms(1000);
            STATE.store(0, Ordering::Relaxed);
        },
        _ => {}
    }
}

/// Initialize a button with an interrupt handler.
fn init_button<T, F>(pin: T, callback: F) -> PinDriver<'static, T::P, Input>
where 
    T: Peripheral + 'static,
    T::P: InputPin + OutputPin,
    F: FnMut() + Send + 'static,
{
    // Configure button pin as input
    let mut button = PinDriver::input(pin).unwrap();

    // Configure button pin with internal pull up
    button.set_pull(Pull::Up).unwrap();

    // Configure button pin to detect interrupts on a positive edge
    button.set_interrupt_type(InterruptType::PosEdge).unwrap();

    // Attach the ISR to the button interrupt
    unsafe { button.subscribe(callback).unwrap() }

    button
}

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    // Take peripherals
    let dp = Peripherals::take().unwrap();

    // Initialize buttons
    let mut button1 = init_button(dp.pins.gpio0, gpio_interrupt_handler1);
    let mut button2 = init_button(dp.pins.gpio1, gpio_interrupt_handler2);
    let mut button3 = init_button(dp.pins.gpio10, gpio_interrupt_handler3);
    let mut button4 = init_button(dp.pins.gpio8, gpio_interrupt_handler4);

    log::info!("Start program");

    loop {
        polling();
        button1.enable_interrupt().unwrap();
        button2.enable_interrupt().unwrap();
        button3.enable_interrupt().unwrap();
        button4.enable_interrupt().unwrap();
    }
}

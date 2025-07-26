/// Handles the buttons.
use anyhow::Result;
use esp_idf_svc::hal::gpio::*;
use esp_idf_svc::hal::peripheral::Peripheral;
use std::sync::atomic::Ordering;

fn get_state() -> u8 {
    crate::STATE.load(Ordering::Relaxed)
}

fn set_state(state: u8) {
    crate::STATE.store(state, Ordering::Relaxed);
}

/// Handlers for the reset button.
fn gpio_interrupt_handler1() {
    match get_state()
    {
        crate::OFFSET_CORRECTION => set_state(crate::OFFSET_CORRECTION_CANCEL),
        crate::RUN_AUTO => set_state(crate::IDLE),
        crate::IDLE => set_state(crate::TERMINATE),
        _ => {}, // do nothing
    }
}

/// Handlers for the offset correction button.
fn gpio_interrupt_handler2() {
    match get_state() {
        crate::IDLE => set_state(crate::OFFSET_CORRECTION),
        crate::OFFSET_CORRECTION => set_state(crate::OFFSET_CORRECTION_END),
        _ => {}, // do nothing
    }
}

// Handlers for the run episode button.
fn gpio_interrupt_handler3() {
    match get_state() {
        crate::IDLE => set_state(crate::RUN_AUTO),
        _ => {}, // do nothing
    }
}

fn gpio_interrupt_handler4() {
    crate::STATE.store(4, Ordering::Relaxed);
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

pub struct Buttons<P1, P2, P3, P4>
where
    P1: Peripheral + 'static,
    P2: Peripheral + 'static,
    P3: Peripheral + 'static,
    P4: Peripheral + 'static,
    P1::P: InputPin + OutputPin,
    P2::P: InputPin + OutputPin,
    P3::P: InputPin + OutputPin,
    P4::P: InputPin + OutputPin,
{
    /// Button to run an episode.
    ///
    /// This button works in the idle state.
    button1: PinDriver<'static, P1::P, Input>,

    /// Button to send episode data to the server.
    button2: PinDriver<'static, P2::P, Input>,

    /// Button to receive model parameters from the server.
    button3: PinDriver<'static, P3::P, Input>,

    /// Button to clear the episode data.
    button4: PinDriver<'static, P4::P, Input>,
}

impl<P1, P2, P3, P4> Buttons<P1, P2, P3, P4>
where
    P1: Peripheral + 'static,
    P2: Peripheral + 'static,
    P3: Peripheral + 'static,
    P4: Peripheral + 'static,
    P1::P: InputPin + OutputPin,
    P2::P: InputPin + OutputPin,
    P3::P: InputPin + OutputPin,
    P4::P: InputPin + OutputPin,
{
    pub fn new(p1: P1, p2: P2, p3: P3, p4: P4) -> Self {
        let button1 = init_button(p1, gpio_interrupt_handler1);
        let button2 = init_button(p2, gpio_interrupt_handler2);
        let button3 = init_button(p3, gpio_interrupt_handler3);
        let button4 = init_button(p4, gpio_interrupt_handler4);

        Self {
            button1,
            button2,
            button3,
            button4,
        }
    }

    /// Enable the interrupt for the buttons.
    pub fn enable_interrupt(&mut self) -> Result<()> {
        self.button1.enable_interrupt()?;
        self.button2.enable_interrupt()?;
        self.button3.enable_interrupt()?;
        self.button4.enable_interrupt()?;
        Ok(())
    }
}

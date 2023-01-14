// Use alias bsp so we can switch boards at a single location
use rp_pico as bsp;

use bsp::hal::{
    gpio::{Output, Pin, PinId, PushPull},
    timer::Instant,
};
use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin};

pub struct Led<I: PinId> {
    pin: Pin<I, Output<PushPull>>,
    pub rate: u64,
    last: Instant,
}

impl<I: PinId> Led<I> {
    pub fn new(pin: Pin<I, Output<PushPull>>) -> Led<I> {
        let rate: u64 = 500;
        Led {
            pin,
            rate,
            last: Instant::from_ticks(0),
        }
    }

    pub fn run(&mut self, now: &Instant) {
        // blink the led
        if self.rate > 0 {
            if (*now - self.last).to_millis() > self.rate {
                self.toggle();
                self.last = *now
            }
        } else {
            self.off();
        }
    }

    fn on(&mut self) {
        self.pin.set_high().unwrap();
    }

    pub fn off(&mut self) {
        self.pin.set_low().unwrap();
    }

    pub fn is_on(&self) -> bool {
        self.pin.is_set_high().unwrap()
    }

    pub fn toggle(&mut self) {
        if self.is_on() {
            self.off()
        } else {
            self.on()
        }
    }
}

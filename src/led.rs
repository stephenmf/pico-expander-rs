use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin};
use hal::timer::Instant;
use rp_pico::hal;

type LedPin = hal::gpio::Pin<hal::gpio::bank0::Gpio25, hal::gpio::Output<hal::gpio::PushPull>>;

pub struct Led {
    pin: LedPin,
    pub rate: u64,
    last: Instant,
}

impl Led {
    pub fn new(pin: LedPin, last: Instant) -> Led {
        let rate: u64 = 500;
        Led { pin, rate, last }
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

    fn is_on(&self) -> bool {
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

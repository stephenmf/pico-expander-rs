//! # Pico USB Serial Example
//!
//! Creates a USB Serial device on a Pico board, with the USB driver running in
//! the main thread.
//!
//! This will create a USB Serial device echoing anything it receives. Incoming
//! ASCII characters are converted to upercase, so you can tell it is working
//! and not just local-echo!
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

mod decoder;
mod led;
mod usb;

// use embedded_hal::digital::v2::{OutputPin, StatefulOutputPin};

// The macro for our start-up function
use rp_pico::entry;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// A shorter alias for the Peripheral Access Crate, which provides low-level
// register access
use rp_pico::hal::pac;

// A shorter alias for the Hardware Abstraction Layer, which provides
// higher-level drivers.
use rp_pico::hal;

use usb_device::{class_prelude::*};

use core::fmt::Write;
use heapless::String;

use decoder::{Commands, DecodeResult, Decoder};
use led::Led;
use usb::Usb;

fn run_command(led: &mut Led, command: Commands, target: u8, value: u16) -> String<64> {
    let mut text: String<64> = String::new();

    if command == Commands::Led {
        led.rate = value as u64;
        writeln!(&mut text, "LA\r").unwrap()
    } else {
        writeln!(
            &mut text,
            "run_command(command: '{}' target: {} value: {})\r",
            command, target, value
        )
        .unwrap()
    }
    text
}

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialised.
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks generate a 125 MHz system clock
    let clocks = hal::clocks::init_clocks_and_plls(
        rp_pico::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // #[cfg(feature = "rp2040-e5")]
    // {
    let sio = hal::Sio::new(pac.SIO);
    let pins = rp_pico::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    // }

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut usb = Usb::new(&usb_bus);
    let mut led = Led::new(pins.led.into_push_pull_output(), timer.get_counter());
    let mut decoder = Decoder::new();

    loop {
        let now = timer.get_counter();
        led.run(&now);
        let mut usb_buffer = [0u8; 64];
        match usb.read(&mut usb_buffer) {
            Ok(0) => {}
            Err(_e) => {}
            Ok(count) => {
                // Decode the input
                for c in usb_buffer.iter().take(count) {
                    match decoder.run(c) {
                        DecodeResult::None => {}
                        DecodeResult::Text(text) => {
                            usb.write(&text);
                        }
                        DecodeResult::Command(command, target, value) => {
                            let text = run_command(&mut led, command, target, value);
                            usb.write(&text);
                        }
                    }
                }
            }
        }
    }
}

// End of file

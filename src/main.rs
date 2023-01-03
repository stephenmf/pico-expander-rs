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

// USB Device support
use usb_device::{class_prelude::*, prelude::*};

// USB Communications Class Device support
use usbd_serial::SerialPort;

use core::fmt::Write;
use heapless::String;

use decoder::{Commands, DecodeResult, Decoder};
use led::Led;

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

    // Configure the clocks
    //
    // The default is to generate a 125 MHz system clock
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

    // Set up the USB Communications Class Device driver
    let mut serial = SerialPort::new(&usb_bus);

    // Create a USB device with a fake VID and PID
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0xcafe, 0x27dd))
        .manufacturer("Field Home I/O")
        .product("Pico I/O Expander")
        .serial_number("00001")
        .device_class(2) // from: https://www.usb.org/defined-class-codes
        .build();

    // get the current timer value
    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS);
    let mut now = timer.get_counter();

    let mut led = Led::new(pins.led.into_push_pull_output());

    let mut decoder = Decoder::new();
    loop {
        // blink the led
        if led.rate > 0 {
            let check = timer.get_counter();
            if (check - now).to_millis() > led.rate {
                led.toggle();
                now = check;
            }
        } else {
            led.off();
        }
        // Check for new usb data
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];
            match serial.read(&mut buf) {
                Ok(0) => {
                    // Do nothing
                }
                Err(_e) => {
                    // Do Nothing
                }
                Ok(count) => {
                    // Decode the input
                    for c in buf.iter().take(count) {
                        match decoder.run(c) {
                            DecodeResult::None => {}
                            DecodeResult::Text(text) => {
                                let bytes = text.as_bytes();
                                if !bytes.is_empty() {
                                    // Send response to the host
                                    let mut out = &bytes[..bytes.len()];
                                    while !out.is_empty() {
                                        match serial.write(out) {
                                            Ok(len) => out = &out[len..],
                                            // On error, just drop unwritten data.
                                            // One possible error is Err(WouldBlock), meaning the USB
                                            // write buffer is full.
                                            Err(_) => break,
                                        }
                                    }
                                }
                            }
                            DecodeResult::Command(command, target, value) => {
                                let mut text: String<64> = String::new();
                                writeln!(
                                    &mut text,
                                    "Run command: '{}' target: {} value: {}\r",
                                    command, target, value
                                )
                                .unwrap();
                                if command == Commands::Led {
                                    led.rate = value as u64
                                }
                                let bytes = text.as_bytes();
                                if !bytes.is_empty() {
                                    // Send response to the host
                                    let mut out = &bytes[..bytes.len()];
                                    while !out.is_empty() {
                                        match serial.write(out) {
                                            Ok(len) => out = &out[len..],
                                            // On error, just drop unwritten data.
                                            // One possible error is Err(WouldBlock), meaning the USB
                                            // write buffer is full.
                                            Err(_) => break,
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// End of file

//! # Pico USB Serial Example
//!
//! Creates a USB Serial device on a Pico board, with the USB driver running in
//! the main thread.
//!
//! This will create a USB Serial device echoing anything it receives. Incoming
//! ASCII characters are converted to uppercase, so you can tell it is working
//! and not just local-echo!
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

mod decoder;
mod led;
mod usb;

// Use alias bsp so we can switch boards at a single location
use rp_pico as bsp;

// The macro for our start-up function
use bsp::entry;

// Ensure we halt the program on panic (if we don't mention this crate it won't
// be linked)
use panic_halt as _;

// Aliases for the Hardware Abstraction Layer, Peripheral Access Crate
// and peripherals.
use bsp::{
    hal::{
        clocks,
        clocks::Clock,
        gpio::{FunctionUart, PinId},
        pac,
        uart::{
            DataBits, Enabled, StopBits, UartConfig, UartDevice, UartPeripheral, ValidUartPinout,
        },
        usb::UsbBus as HalUsbBus,
        Sio, Timer, Watchdog,
    },
    Pins,
};

use fugit::RateExtU32;
use usb_device::class_prelude::*;

use core::fmt::Write;
use heapless::String;

// Local modules.
use decoder::{Commands, DecodeResult, Decoder};
use led::Led;
use usb::Usb;

struct Console<D: UartDevice, P: ValidUartPinout<D>> {
    uart: UartPeripheral<Enabled, D, P>,
}

impl<D: UartDevice, P: ValidUartPinout<D>> Console<D, P> {
    fn new(uart: UartPeripheral<Enabled, D, P>) -> Console<D, P> {
        Console { uart }
    }
}

struct Io<'a, B: UsbBus, LP: PinId, D: UartDevice, P: ValidUartPinout<D>> {
    timer: Timer,
    led: Led<LP>,
    console: Console<D, P>,
    usb: Usb<'a, B>,
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
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Configure the clocks generate a 125 MHz system clock
    let clocks = clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Set up the USB driver
    let usb_bus = UsbBusAllocator::new(HalUsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let uart = UartPeripheral::new(
        pac.UART0,
        (
            // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
            pins.gpio0.into_mode::<FunctionUart>(),
            // UART RX (characters received by RP2040) on pin 2 (GPIO1)
            pins.gpio1.into_mode::<FunctionUart>(),
        ),
        &mut pac.RESETS,
    );
    let uart = uart.enable(
        UartConfig::new(115200.Hz(), DataBits::Eight, None, StopBits::One),
        clocks.peripheral_clock.freq(),
    );

    let io = Io {
        timer: Timer::new(pac.TIMER, &mut pac.RESETS),
        led: Led::new(pins.led.into_push_pull_output()),
        console: Console::new(uart.unwrap()),
        usb: Usb::new(&usb_bus),
    };
    forever(io);
}

fn forever<B: UsbBus, LP: PinId, D: UartDevice, P: ValidUartPinout<D>>(
    mut io: Io<B, LP, D, P>,
) -> ! {
    let mut decoder = Decoder::new();
    let mut usb_buffer = [0u8; 64];
    let mut uart_buffer = [0u8; 16];
    loop {
        let now = io.timer.get_counter();
        io.led.run(&now);
        if let Some(count) = io.usb.read(&mut usb_buffer) {
            // Decode the input
            for c in usb_buffer.iter().take(count) {
                match decoder.run(c) {
                    DecodeResult::None => {}
                    DecodeResult::Text(text) => {
                        io.usb.write(&text);
                    }
                    DecodeResult::Command(cmd, target, value) => {
                        let text = command(&mut io, cmd, target, value);
                        io.usb.write(&text);
                    }
                }
            }
        }
        if io.console.uart.uart_is_readable() {
            match io.console.uart.read_raw(&mut uart_buffer) {
                Ok(0) => {}
                Err(_) => {}
                // Echo the input for now.
                Ok(_count) => if let Ok(_count) = io.console.uart.write_raw(&uart_buffer) {},
            }
        }
    }
}

fn command<B: UsbBus, LP: PinId, D: UartDevice, P: ValidUartPinout<D>>(
    io: &mut Io<B, LP, D, P>,
    cmd: Commands,
    target: u8,
    value: u16,
) -> String<64> {
    let mut text: String<64> = String::new();

    if cmd == Commands::Led {
        io.led.rate = value as u64;
        writeln!(&mut text, "LA\r").unwrap()
    } else if cmd == Commands::Status {
        writeln!(&mut text, "SLv{}r{}\r", io.led.is_on(), io.led.rate).unwrap()
    } else {
        writeln!(
            &mut text,
            "run_command(command: '{}' target: {} value: {})\r",
            cmd, target, value
        )
        .unwrap()
    }
    text
}

// End of file

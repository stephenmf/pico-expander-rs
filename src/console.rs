// Use alias bsp so we can switch boards at a single location
use rp_pico as bsp;

use bsp::hal::uart::{
    DataBits, Disabled, Enabled, StopBits, UartConfig, UartDevice, UartPeripheral, ValidUartPinout,
};
use fugit::{HertzU32, RateExtU32};

struct Buffer {
    rpos: usize,
    wpos: usize,
    buffer: [u8; 64],
}

impl Buffer {
    fn new() -> Buffer {
        Buffer {
            rpos: 0,
            wpos: 0,
            buffer: [0; 64],
        }
    }

    fn put(&mut self, uin: &u8) {
        let npos = (self.wpos + 1) & 63;
        if npos != self.rpos {
            self.buffer[self.wpos] = *uin;
            self.wpos = npos;
        }
    }

    fn get(&mut self) -> Option<u8> {
        let npos = (self.rpos + 1) & 63;
        if self.rpos != self.wpos {
            let out = self.buffer[self.rpos];
            self.rpos = npos;
            return Some(out);
        }
        None
    }

    fn empty(&self) -> bool {
        self.rpos == self.wpos
    }
}

pub struct Console<D: UartDevice, P: ValidUartPinout<D>> {
    uart: UartPeripheral<Enabled, D, P>,
    buffer: Buffer,
}

impl<D: UartDevice, P: ValidUartPinout<D>> Console<D, P> {
    pub fn new(uart: UartPeripheral<Disabled, D, P>, frequency: HertzU32) -> Console<D, P> {
        let uart = uart
            .enable(
                UartConfig::new(115_200.Hz(), DataBits::Eight, None, StopBits::One),
                frequency,
            )
            .unwrap();
        Console {
            uart,
            buffer: Buffer::new(),
        }
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        while self.uart.uart_is_writable() && !self.buffer.empty() {
            if let Some(out) = self.buffer.get() {
                let out = [out];
                if let Ok(_count) = self.uart.write_raw(&out) {}
            }
        }
        if self.uart.uart_is_readable() {
            match self.uart.read_raw(buffer) {
                Ok(0) => {}
                Err(_) => {}
                // Echo the input for now.
                Ok(count) => return Some(count),
            }
        }
        None
    }

    pub fn write(&mut self, buffer: &[u8]) {
        for uin in buffer {
            self.buffer.put(uin)
        }
    }
}

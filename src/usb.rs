use heapless::String;
use usb_device::{class_prelude::*, prelude::*};
use usbd_serial::SerialPort;

pub struct Usb<'a, B: UsbBus> {
    device: UsbDevice<'a, B>,
    serial: SerialPort<'a, B>,
}

impl<'a, B: UsbBus> Usb<'a, B> {
    pub fn new(usb_bus: &'a UsbBusAllocator<B>) -> Usb<'a, B> {
        // Set up the USB Communications Class Device driver
        let serial = SerialPort::new(usb_bus);

        // Create a USB device with a fake VID and PID
        let device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0xcafe, 0x27dd))
            .manufacturer("Field Home I/O")
            .product("Pico I/O Expander")
            .serial_number("00001")
            .device_class(2) // from: https://www.usb.org/defined-class-codes
            .build();

        Usb { device, serial }
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Option<usize> {
        //Result<usize, UsbError> {
        if self.device.poll(&mut [&mut self.serial]) {
            match self.serial.read(buffer) {
                Ok(0) => {}
                Err(_e) => {}
                Ok(count) => return Some(count),
            }
        }
        None
    }

    pub fn write(&mut self, text: &String<64>) {
        let bytes = text.as_bytes();
        if !bytes.is_empty() {
            // Send response to the host
            let mut out = &bytes[..bytes.len()];
            while !out.is_empty() {
                match self.serial.write(out) {
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

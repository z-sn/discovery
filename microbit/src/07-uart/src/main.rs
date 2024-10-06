#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use core::fmt::Write;
use heapless::Vec;

#[cfg(feature = "v1")]
use microbit::{
    hal::prelude::*,
    hal::uart,
    hal::uart::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
use microbit::{
    hal::prelude::*,
    hal::uarte,
    hal::uarte::{Baudrate, Parity},
};

#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let mut serial = {
        uart::Uart::new(
            board.UART0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        )
    };

    #[cfg(feature = "v2")]
    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };
/*
    for byte in b"The quick brown fox jumps over the lazy dog.\r\n".iter() {
        nb::block!(serial.write(*byte)).unwrap();

    }
*/
    write!(serial, "Type some words. I will respond.\r\n").unwrap();

    let mut buffer: Vec<u8, 32> = Vec::new();
    let empty: Vec<u8, 5> = Vec::new();
    loop {
        let byte = nb::block!(serial.read()).unwrap();
        buffer.push(byte);
        if byte == 13 {
            // Clear screen
            for c in empty.iter() {
                nb::block!(serial.write(*c)).unwrap();
            }

            // Echo input
            for c in buffer.iter() {
                nb::block!(serial.write(*c)).unwrap();
            }
            buffer.clear();
            nb::block!(serial.flush()).unwrap();
        }
    }

    loop {}
}

//#![deny(unsafe_code)]
#![no_main]
#![no_std]

use cortex_m_rt::entry;
use rtt_target::{rtt_init_print, rprintln};
use panic_rtt_target as _;
use core::fmt::Write;
use heapless::Vec;
use core::str::from_utf8;

use microbit::hal::prelude::*;

#[cfg(feature = "v1")]
use microbit::{
    hal::twi,
    pac::twi0::frequency::FREQUENCY_A,
};

#[cfg(feature = "v2")]
use microbit::{
    hal::{twim, prelude::*, uarte, uarte::{Baudrate, Parity}},
    pac::twim0::frequency::FREQUENCY_A,
};

use lsm303agr::{
    AccelOutputDataRate, MagOutputDataRate, Lsm303agr,
};

const ACCELEROMETER_ADDR: u8 = 0b0011001;
const MAGNETOMETER_ADDR: u8 = 0b0011110;

const ACCELEROMETER_ID_REG: u8 = 0x0f;
const MAGNETOMETER_ID_REG: u8 = 0x4f;

#[cfg(feature = "v2")]
mod serial_setup;
#[cfg(feature = "v2")]
use serial_setup::UartePort;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    let mut serial = {
        let serial = uarte::Uarte::new(
            board.UARTE0,
            board.uart.into(),
            Parity::EXCLUDED,
            Baudrate::BAUD115200,
        );
        UartePort::new(serial)
    };

    let mut i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz50).unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz50).unwrap();
    
    // Need this, otherwise gets stuck
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    write!(serial, "Command : magnetometer or accelerometer \r\n");
    let mut buffer: Vec<u8, 32> = Vec::new();
    loop{
        loop {
            // Read from serial
            let byte = nb::block!(serial.read()).unwrap();
            buffer.push(byte);
            if byte == 13 {
                let command = from_utf8(&buffer).unwrap().trim();
                if command == "magnetometer" {
                    while !sensor.mag_status().unwrap().xyz_new_data{}
                    let magdata = sensor.mag_data().unwrap();
                    write!(serial, "Magnetometer:x {} y {} z {}\r\n", magdata.x, magdata.y, magdata.z).unwrap();
                }
                else if command == "accelerometer"
                {
                    while !sensor.accel_status().unwrap().xyz_new_data{}
                    let data = sensor.accel_data().unwrap();
                    write!(serial, "Acceleraton:x {} y {} z {}\r\n", data.x, data.y, data.z).unwrap();
                }
                else {
                    write!(serial, "Invalid command: {}\r\n", command).unwrap();
                }
                buffer.clear();
            }
        }
       
    } 
    /*

    let mut acc = [0];
    let mut mag = [0];

    // First write the address + register onto the bus, then read the chip's responses
    i2c.write_read(ACCELEROMETER_ADDR, &[ACCELEROMETER_ID_REG], &mut acc).unwrap();
    i2c.write_read(MAGNETOMETER_ADDR, &[MAGNETOMETER_ID_REG], &mut mag).unwrap();

    rprintln!("The accelerometer chip's id is: {:#b}", acc[0]);
    rprintln!("The magnetometer chip's id is: {:#b}", mag[0]);
*/
    loop {}
}

// Disables the Rust standard library (no operating system environment like Windows/Linux)
#![no_std]
// Tells the compiler that this program has a custom entry point instead of a standard main function
#![no_main]
// Safety rule: errors out if `core::mem::forget` is used, preventing leaked hardware control buffers
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
// Safety rule: errors out if variables consume too much stack space, preventing stack overflows
#![deny(clippy::large_stack_frames)]

// Provides panic handling infrastructure, printing debugging details over the UART connection
use esp_backtrace as _; 
use esp_hal::analog::adc::{Adc, AdcConfig, Attenuation};
use esp_hal::delay::Delay;
use esp_hal::gpio::{DriveMode, Input, InputConfig};
use esp_hal::ledc::channel::ChannelIFace;
use esp_hal::ledc::timer::TimerIFace;
use esp_hal::ledc::{LSGlobalClkSource, Ledc, LowSpeed, channel, timer};
use esp_hal::time::Rate;
// Macro used to define the actual startup execution point of the microcontroller firmware
use esp_hal::{delay, main};
// Enum used to configure the ESP32-S3 processor execution speed
use esp_hal::clock::CpuClock;
// Macro to print text directly to your computer terminal console screen via USB/Serial
use esp_println::println;
// Master I2C communication peripheral driver blocks from Espressif's HAL layer
use esp_hal::i2c::master::{Config, I2c};
// Embedded graphics traits, color styles, and text layout format configurations
use embedded_graphics::{
    mono_font::{ascii::FONT_6X10, MonoTextStyleBuilder},
    pixelcolor::BinaryColor,
    prelude::*,
    text::{Baseline, Text},
};
// Pulls in the modern SH1106 driver canvas structures and communication wrapper blocks
use mini_oled::prelude::*; 

// Creates an app metadata descriptor block inside the final binary file required by the bootloader
esp_bootloader_esp_idf::esp_app_desc!();

// Tells the compiler it is okay to create the display canvas buffer arrays right on the main stack frame
#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
// The actual entry function invoked by the ESP32-S3 hardware bootloader right after powering up
#[main]
fn main() -> ! {
    // 1. Create a configuration profile forcing the ESP32-S3 CPU to its maximum clock speed (usually 240MHz)
    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    
    // 2. Initialize the microcontroller system architecture using the custom configuration parameters
    let peripherals = esp_hal::init(config);

    let led = peripherals.GPIO5;

    let mut ledc = Ledc::new(peripherals.LEDC);

    ledc.set_global_slow_clock(LSGlobalClkSource::APBClk);

    let mut timer0 = ledc.timer::<LowSpeed>(
        timer::Number::Timer0
    );

    timer0
        .configure(timer::config::Config {
            duty: timer::config::Duty::Duty10Bit,
            clock_source: timer::LSClockSource::APBClk,
            frequency: Rate::from_hz(5000),
        })
        .unwrap();

    let mut channel0 = ledc.channel(
        channel::Number::Channel0,
        led,
    );

    channel0
        .configure(channel::config::Config {
            timer: &timer0,
            duty_pct: 0,
            drive_mode: DriveMode::PushPull,
        })
        .unwrap();

    

    loop {
        // Fade from 0% → 100%
        channel0
            .start_duty_fade(0, 100, 2000)
            .unwrap();

        while channel0.is_duty_fade_running() {}

        // Fade from 100% → 0%
        channel0
            .start_duty_fade(100, 0, 2000)
            .unwrap();

        while channel0.is_duty_fade_running() {}
    }
}

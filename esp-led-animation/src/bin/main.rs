#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use core::time;

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig, Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
use esp_println::println;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    // generator version: 1.3.0
    // generator parameters: --chip esp32s3

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);
    let mut led1 = Output::new(
        peripherals.GPIO4,
        Level::Low,
        OutputConfig::default(),
    );
     let mut led2 = Output::new(
        peripherals.GPIO5,
        Level::Low,
        OutputConfig::default(),
    );
    let mut led3 = Output::new(
        peripherals.GPIO6,
        Level::Low,
        OutputConfig::default(),
    );
    let mut led4 = Output::new(
        peripherals.GPIO7,
        Level::Low,
        OutputConfig::default(),
    );




    let button = Input::new(
        peripherals.GPIO20,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let delay = Delay::new();
    let mut count = 0;
    loop {
        if button.is_high() {

            // LED 1: every 100 ms
            if count % 10 == 0 {
                led1.toggle();
            }

            // LED 2: every 200 ms
            if count % 20 == 0 {
                led2.toggle();
            }

            // LED 3: every 500 ms
            if count % 50 == 0 {
                led3.toggle();
            }

            // LED 4: every 1000 ms
            if count % 100 == 0 {
                led4.toggle();
            }

            count += 1;

            // Common time tick = 10 ms
            delay.delay_millis(10);
            
    } else {
            // Reset the count and turn off all LEDs when the button is released
            count = 0;
            led1.set_low();
            led2.set_low();
            led3.set_low();
            led4.set_low();
        }

    }

}

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_hal::clock::CpuClock;
use esp_hal::delay::Delay;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
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
    let mut led = Output::new(
        peripherals.GPIO18,
        Level::Low,
        OutputConfig::default(),
    );

    let mut led2 = Output::new(
        peripherals.GPIO16,
        Level::Low,
        OutputConfig::default(),
    );
    let delay = Delay::new();

    loop {
        led.set_high();
        println!("LED 1 set:HIGH");
        led2.set_low();
        println!("LED 2 set:LOW");
        delay.delay_millis(500);

        println!("LED 1 set:LOW");
        led.set_low();
        println!("LED 2 set:HIGH");
        led2.set_high();
        delay.delay_millis(500);


    }

}

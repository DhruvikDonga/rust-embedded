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
    let mut led = Output::new(
        peripherals.GPIO16,
        Level::Low,
        OutputConfig::default(),
    );

    let button = Input::new(
        peripherals.GPIO18,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let delay = Delay::new();
    let mut timems: i32 =1000; 
    loop {
        if button.is_low() {
            if timems == -100 { //-100 then reset to 1000ms
                timems = 1000;
            } else { //increase speed
                timems -= 100;
            }
            println!("Button pressed speed up: {}ms", timems);
            while button.is_low() {
                delay.delay_millis(10);
            }
        } else {
            if timems == 0 { //if 0 ms then just keep the LED on
                led.set_high();
            } else {
                led.set_high();
                println!("LED set:HIGH");
                delay.delay_millis(timems.try_into().unwrap_or(100));

                led.set_low();
                println!("LED set:LOW");
                delay.delay_millis(timems.try_into().unwrap_or(100));
            }
        }



    }

}

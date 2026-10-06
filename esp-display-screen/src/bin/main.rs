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
use esp_hal::delay::Delay;
use esp_hal::gpio::{Input, InputConfig};
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

    // 3. Take ownership of the I2C0 peripheral block, configure it, and bind the pins to it.
    // .unwrap() immediately crashes safely if the initialization fails.
    let i2c = I2c::new(peripherals.I2C0, Config::default()).unwrap()
        .with_sda(peripherals.GPIO1)   // Maps the Serial Data (SDA) stream to Physical Pin 1
        .with_scl(peripherals.GPIO2);  // Maps the Serial Clock (SCL) stream to Physical Pin 2

    // Logs the successful status of the physical wire configuration out to your computer
    println!("Modern I2C initialized!");

    // 4. Wrap the modern I2C block inside a generic interface layer using the standard address 0x3C
    let i2c_interface = I2cInterface::new(i2c, 0x3C);

    // 5. Instantiate the physical SH1106 driver using the configured communication interface wrapper
    let mut display = Sh1106::new(i2c_interface);
    
    // 6. Transmit the physical hardware power-up registers, clearing out visual pixel artifacts automatically
    display.init().unwrap();
    
    // Logs that the hardware has finished its power up commands sequence successfully
    println!("Modern SH1106 OLED initialized and cleared!");

    // 7. Define the styling parameters for rendering text (maps FONT_6X10 to monochrome glowing pixels)
    let text_style = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(BinaryColor::On) // BinaryColor::On turns the OLED pixel elements on (white/blue)
        .build();

    // 8. Construct a string element pointing at layout coordinates (0, 0) inside the display window area
    Text::with_baseline(
        "Dhruvik",
        Point::new(0, 0),        // Top-left boundary of the grid tracking coordinate matrix
        text_style,
        Baseline::Top,           // Aligns the text string perfectly to the top of the line frame boundary
    )
    .draw(display.get_mut_canvas()) // Renders the text characters straight into the hidden off-screen memory grid
    .unwrap();

    // 9. Send the fully rendered off-screen memory grid payload down the I2C wires to change physical display pixels
    display.flush().unwrap();

     // --- ANIMATION COORDINATE VARIABLES ---
    let mut x: i32 = 0;
    let mut y: i32 = 0;
    
    // Direction modifiers: 1 means moving forward/down, -1 means moving backward/up
    let mut x_direction: i32 = 1;
    let mut y_direction: i32 = 1;

    // Boundary constraints based on screen size (128x64) and roughly accounting for text size
    // FONT_6X10 with "Dhruvik" (7 characters) is roughly 42 pixels wide and 10 pixels tall
    let max_x = 128 - 42;
    let max_y = 64 - 10;

    let delay = Delay::new();

    //Button to modify text when clicked
    let button = Input::new(
        peripherals.GPIO20,
        InputConfig::default().with_pull(esp_hal::gpio::Pull::Up),
    );
    let mut is_button_pressed = false;
    let mut text_to_display = "Dhruvik";
    loop {
        if button.is_low() {
            println!("Button pressed!: {}",is_button_pressed);
            is_button_pressed = !is_button_pressed; // Toggle the button state

            while button.is_low() {
                delay.delay_millis(10);
            }
        }
        if is_button_pressed {
           text_to_display = "Swity";
        } else {
            text_to_display = "Dhruvik";
        }
        // 1. Draw the text at the current dynamic (x, y) coordinates

        Text::with_baseline(
                text_to_display,
                Point::new(x, y), // Dynamic coordinate injection
                text_style,
                Baseline::Top,
            )
            .draw(display.get_mut_canvas())
            .unwrap();
        // 2. Push the canvas updates down the I2C pipeline to update the panel pixels
        display.flush().unwrap();

        // 3. Block execution briefly (approx 30ms = ~33 Frames Per Second animation speed)
        delay.delay_millis(30);

        // 4. Wipe the off-screen canvas completely so the next frame doesn't smear
        display.get_mut_canvas().clear(BinaryColor::Off).unwrap();

        // 5. Calculate the next coordinate positions based on active velocity directions
        x += x_direction;
        y += y_direction;

        // 6. Check Horizontal Edge Bounds (Bounce if text hits left or right border)
        if x >= max_x {
            x_direction = -1; // Reverse direction to move left
        } else if x <= 0 {
            x_direction = 1;  // Reverse direction to move right
        }

        // 7. Check Vertical Edge Bounds (Bounce if text hits top or bottom border)
        if y >= max_y {
            y_direction = -1; // Reverse direction to move up
        } else if y <= 0 {
            y_direction = 1;  // Reverse direction to move down
        }
    }
}

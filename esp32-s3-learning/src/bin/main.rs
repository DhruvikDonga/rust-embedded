#![no_std]
#![no_main]

// We are running directly on the microcontroller without the Rust
// standard library. This is typical for bare-metal embedded systems.
//
// `no_main` means we provide our own embedded entry point using
// `#[esp_hal::main]` instead of the normal Rust `main()` runtime.

use esp_backtrace as _;

use esp_hal::{
    delay::Delay,
    gpio::{Level, Output, OutputConfig},
    rmt::Rmt,
    time::Rate,
};

use esp_hal_smartled::{RmtSmartLeds, Ws2811Timing, buffer_size, color_order};

use esp_println::println;

use smart_leds::{RGB8, SmartLedsWrite};

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal::main]
fn main() -> ! {
    // ============================================================
    // 1. INITIALIZE THE ESP32-S3 PERIPHERALS
    // ============================================================
    //
    // This initializes the ESP32-S3 hardware and gives us access
    // to its peripherals:
    //
    //     GPIO4
    //     GPIO47
    //     GPIO48
    //     RMT
    //     ...
    //
    // We interact with these hardware blocks through esp-hal.
    //
    // Mental model:
    //
    //     ESP32-S3 hardware
    //            ↓
    //         esp-hal
    //            ↓
    //      peripherals.GPIO48
    //
    let peripherals = esp_hal::init(esp_hal::Config::default());

    // Delay lets us pause execution for a specific amount of time.
    //
    // We will use it both for hardware stabilization and for
    // keeping each LED color visible for one second.
    let delay = Delay::new();

    println!("Initializing system power tracks");

    // ============================================================
    // 2. ENABLE THE RGB LED POWER / CONTROL CIRCUIT
    // ============================================================
    //
    // IMPORTANT DISCOVERY:
    //
    // GPIO48 is NOT the only GPIO involved in controlling the
    // onboard RGB LED on this development board.
    //
    // GPIO4 and GPIO47 are used as board-level power/control
    // enables. They need to be HIGH before the RGB LED circuitry
    // becomes operational.
    //
    // This is a good example of why the ESP32-S3 chip pinout alone
    // is not enough to understand a development board.
    //
    // There are multiple layers:
    //
    //     ESP32-S3 chip
    //          ↓
    //     Development board PCB
    //          ↓
    //     Power/control circuitry
    //          ↓
    //     RGB LED
    //
    // So our actual hardware path is approximately:
    //
    //     GPIO4  ──┐
    //             ├──> RGB LED power/control
    //     GPIO47 ──┘
    //
    //     GPIO48 ─────> RGB LED DATA
    //
    // `Output::new()` configures the GPIO as an output and
    // immediately drives it HIGH.
    //
    // The `_` prefix is intentional: we don't need to manipulate
    // these outputs again, but keeping the Output objects alive
    // keeps the pins configured for the lifetime of the program.
    //
    let _rgb_pwr_a = Output::new(peripherals.GPIO4, Level::High, OutputConfig::default());

    let _rgb_pwr_b = Output::new(peripherals.GPIO47, Level::High, OutputConfig::default());

    // Give the board's power/control circuitry some time to
    // stabilize before we start communicating with the LED.
    delay.delay_millis(100u32);

    // ============================================================
    // 3. SELECT GPIO48 AS THE RGB LED DATA PIN
    // ============================================================
    //
    // GPIO48 is physically connected to the data input of the
    // onboard addressable RGB LED.
    //
    // Important distinction:
    //
    //     RMT  = peripheral that generates the precise waveform
    //
    //     GPIO48 = physical MCU pin through which that waveform
    //              reaches the LED
    //
    // We are NOT manually toggling GPIO48 in software.
    //
    // Instead:
    //
    //     CPU
    //      ↓
    //     RMT peripheral
    //      ↓
    //     GPIO48
    //      ↓
    //     electrical waveform
    //      ↓
    //     RGB LED
    //
    let led_pin = peripherals.GPIO48;

    // ============================================================
    // 4. INITIALIZE THE RMT PERIPHERAL
    // ============================================================
    //
    // RMT = Remote Control peripheral.
    //
    // Although originally designed for things such as infrared
    // remote-control signals, it is useful for generating precise
    // digital waveforms.
    //
    // Addressable RGB LEDs such as WS281x-family LEDs require very
    // precise HIGH/LOW pulse durations to distinguish between
    // binary 0 and 1.
    //
    // Instead of the CPU doing:
    //
    //     GPIO HIGH
    //     wait
    //     GPIO LOW
    //     wait
    //     GPIO HIGH
    //     ...
    //
    // we configure the RMT hardware to generate those timings.
    //
    // We configure the RMT with an 80 MHz base clock.
    //
    let rmt = Rmt::new(peripherals.RMT, Rate::from_mhz(80)).expect("Failed RMT init");

    // ============================================================
    // 5. CREATE THE SMART LED DRIVER
    // ============================================================
    //
    // This connects the high-level RGB representation to the
    // low-level RMT waveform generator.
    //
    // Conceptually:
    //
    //     RGB8
    //       ↓
    //     GRB ordering
    //       ↓
    //     binary bits
    //       ↓
    //     WS2811 timing encoding
    //       ↓
    //     RMT
    //       ↓
    //     GPIO48
    //       ↓
    //     RGB LED
    //
    let mut led = RmtSmartLeds::<
        // ----------------------------------------------------
        // BUFFER SIZE
        // ----------------------------------------------------
        //
        // We have one RGB LED.
        //
        // RGB8 contains:
        //
        //     R = 8 bits
        //     G = 8 bits
        //     B = 8 bits
        //
        // The smart-led library calculates the required RMT
        // buffer size for one RGB8 LED.
        //
        { buffer_size::<RGB8>(1) },
        // ----------------------------------------------------
        // MODE
        // ----------------------------------------------------
        //
        // `_` lets Rust infer the appropriate RMT mode.
        //
        _,
        // ----------------------------------------------------
        // COLOR TYPE
        // ----------------------------------------------------
        //
        // RGB8 represents three 8-bit color components:
        //
        //     R = 0..255
        //     G = 0..255
        //     B = 0..255
        //
        RGB8,
        // ----------------------------------------------------
        // COLOR ORDER
        // ----------------------------------------------------
        //
        // Although our Rust structure is RGB:
        //
        //     RGB8 { r, g, b }
        //
        // many addressable RGB LEDs expect the bytes on the
        // physical data line in GRB order:
        //
        //     Green → Red → Blue
        //
        // The library performs this conversion for us.
        //
        color_order::Grb,
        // ----------------------------------------------------
        // LED PROTOCOL TIMING
        // ----------------------------------------------------
        //
        // This tells the driver how to convert binary 0/1
        // values into physical HIGH/LOW pulse durations.
        //
        // The LED does not receive a number such as "30".
        //
        // It receives an electrical waveform.
        //
        // Conceptually:
        //
        //     bit 0 → short HIGH + longer LOW
        //
        //     bit 1 → longer HIGH + shorter LOW
        //
        // The exact timing profile is defined by
        // Ws2811Timing.
        //
        Ws2811Timing,
    >::new(
        // RMT channel used to generate the waveform.
        rmt.channel0,
        // Physical GPIO carrying the generated waveform.
        led_pin,
    )
    .unwrap();

    println!("Power stable! Beginning color loop...");

    // ============================================================
    // 6. CONTINUOUSLY SEND RGB DATA
    // ============================================================
    //
    // The loop demonstrates the complete data path from Rust data
    // structures all the way to an electrical signal on GPIO48.
    //
    loop {
        // --------------------------------------------------------
        // RED
        // --------------------------------------------------------
        //
        // RGB8:
        //
        //     R = 30
        //     G = 0
        //     B = 0
        //
        // `30` is only the logical brightness value here.
        // It is eventually converted into bits and then into
        // precisely timed electrical pulses by the RMT.
        //
        println!("Color Frame: RED");

        let _ = led.write([RGB8 { r: 30, g: 0, b: 0 }].into_iter());

        delay.delay_millis(1000);

        // --------------------------------------------------------
        // GREEN
        // --------------------------------------------------------
        //
        //     R = 0
        //     G = 30
        //     B = 0
        //
        println!("Color Frame: GREEN");

        let _ = led.write([RGB8 { r: 0, g: 30, b: 0 }].into_iter());

        delay.delay_millis(1000);

        // --------------------------------------------------------
        // BLUE
        // --------------------------------------------------------
        //
        //     R = 0
        //     G = 0
        //     B = 30
        //
        println!("Color Frame: BLUE");

        let _ = led.write([RGB8 { r: 0, g: 0, b: 30 }].into_iter());

        delay.delay_millis(1000);
    }
}

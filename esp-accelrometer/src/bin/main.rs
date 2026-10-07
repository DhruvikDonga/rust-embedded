#![no_std]
#![no_main]

use esp_backtrace as _;

use esp_hal::{
    clock::CpuClock,
    i2c::master::{Config, I2c},
    main,
    time::{Duration, Instant, Rate},
};

use esp_println::println;

esp_bootloader_esp_idf::esp_app_desc!();

#[main]
fn main() -> ! {
    let config = esp_hal::Config::default()
        .with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    // GPIO1 = SDA
    // GPIO2 = SCL
    let mut i2c = I2c::new(
        peripherals.I2C0,
        Config::default()
            .with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_scl(peripherals.GPIO1)
    .with_sda(peripherals.GPIO2);

    const MPU_ADDR: u8 = 0x68;

    println!("MPU6050 test");
    println!("I2C address: 0x68");

    // --------------------------------------------------
    // 1. Read WHO_AM_I
    // --------------------------------------------------

    let mut who_am_i = [0u8; 1];

    i2c.write_read(
        MPU_ADDR,
        &[0x75],
        &mut who_am_i,
    )
    .unwrap();

    println!(
        "WHO_AM_I = 0x{:02X}",
        who_am_i[0]
    );

    // --------------------------------------------------
    // 2. Wake up the sensor
    // --------------------------------------------------

    // PWR_MGMT_1 = 0x6B
    // Write 0x00 → wake sensor
    i2c.write(
        MPU_ADDR,
        &[0x6B, 0x00],
    )
    .unwrap();

    println!("Sensor awakened!");

    // Small delay
    let start = Instant::now();

    while start.elapsed() < Duration::from_millis(100) {
        core::hint::spin_loop();
    }

    // --------------------------------------------------
    // 3. Read accelerometer
    // --------------------------------------------------

    // Accelerometer registers:
    //
    // 0x3B = X high
    // 0x3C = X low
    // 0x3D = Y high
    // 0x3E = Y low
    // 0x3F = Z high
    // 0x40 = Z low

    let mut data = [0u8; 6];

    println!("Reading accelerometer...");
    println!("------------------------");

    loop {
        i2c.write_read(
            MPU_ADDR,
            &[0x3B],
            &mut data,
        )
        .unwrap();

        // Combine high + low bytes into signed 16-bit values

        let accel_x =
            i16::from_be_bytes([data[0], data[1]]);

        let accel_y =
            i16::from_be_bytes([data[2], data[3]]);

        let accel_z =
            i16::from_be_bytes([data[4], data[5]]);

        println!(
            "Raw X: {:6} | Y: {:6} | Z: {:6}",
            accel_x,
            accel_y,
            accel_z
        );

        // MPU6050 default accelerometer range = ±2g
        //
        // 16384 LSB = 1g
        //
        let x_g = accel_x as f32 / 16384.0;
        let y_g = accel_y as f32 / 16384.0;
        let z_g = accel_z as f32 / 16384.0;

        println!(
            "X: {:.2} g | Y: {:.2} g | Z: {:.2} g",
            x_g,
            y_g,
            z_g
        );

        println!("------------------------");

        let start = Instant::now();

        while start.elapsed() < Duration::from_millis(500) {
            core::hint::spin_loop();
        }
    }
}
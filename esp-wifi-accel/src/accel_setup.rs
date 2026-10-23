use core::cell::RefCell;

use critical_section::Mutex;
use esp_hal::{Blocking, i2c::master::I2c};
use esp_println::println;

const MPU_ADDR: u8 = 0x68;


// Latest accelerometer reading shared between the sensor and HTTP tasks.
#[derive(Clone, Copy)]
pub struct AccelReading {
    pub connected: bool,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub magnitude: f32,
}

impl AccelReading {
    const fn disconnected() -> Self {
        Self {
            connected: false,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            magnitude: 0.0,
        }
    }
}

// Critical-section protection keeps shared state access safe.
static ACCEL_STATE: Mutex<RefCell<AccelReading>> =
    Mutex::new(RefCell::new(AccelReading::disconnected()));

pub fn set_accel_reading(reading: AccelReading) {
    critical_section::with(|cs| {
        *ACCEL_STATE.borrow(cs).borrow_mut() = reading;
    });
}

pub fn get_accel_reading() -> AccelReading {
    critical_section::with(|cs| {
        *ACCEL_STATE.borrow(cs).borrow()
    })
}

pub fn accel_setup(spawner: embassy_executor::Spawner, mut i2c: I2c<'static, Blocking>,
) {
    spawner
        .spawn(
            accelerometer_task(
                i2c,
            )
            .expect("Could not start accelerometer task"),
        );
}


// Reads the MPU6050 independently from the web server.
#[embassy_executor::task]
async fn accelerometer_task(
   mut i2c: I2c<'static, Blocking>,
) {

    println!("Accelerometer task started");

    let mut initialized = false;

    loop {
        // Reinitialize after a failed I2C transaction.
        if !initialized {
            if i2c.write(MPU_ADDR, &[0x6B, 0x00]).is_err() {
                set_accel_reading(AccelReading::disconnected());
                println!("Accelerometer unavailable; retrying...");
                embassy_time::Timer::after_secs(1).await;
                continue;
            }

            let mut who_am_i = [0u8; 1];

            match i2c.write_read(
                MPU_ADDR,
                &[0x75],
                &mut who_am_i,
            ) {
                Ok(()) => {
                    println!(
                        "Accelerometer detected: WHO_AM_I = 0x{:02X}",
                        who_am_i[0]
                    );
                    initialized = true;
                }
                Err(_) => {
                    set_accel_reading(AccelReading::disconnected());
                    println!("Accelerometer unavailable; retrying...");
                    embassy_time::Timer::after_secs(1).await;
                    continue;
                }
            }
        }

        let mut data = [0u8; 6];

        match i2c.write_read(
            MPU_ADDR,
            &[0x3B],
            &mut data,
        ) {
            Ok(()) => {
                let raw_x =
                    i16::from_be_bytes([data[0], data[1]]);
                let raw_y =
                    i16::from_be_bytes([data[2], data[3]]);
                let raw_z =
                    i16::from_be_bytes([data[4], data[5]]);

                // Assumes the sensor's accelerometer range is ±2g.
                let x = raw_x as f32 / 16384.0;
                let y = raw_y as f32 / 16384.0;
                let z = raw_z as f32 / 16384.0;

                let magnitude =
                    libm::sqrtf(x * x + y * y + z * z);

                set_accel_reading(AccelReading {
                    connected: true,
                    x,
                    y,
                    z,
                    magnitude,
                });
            }
            Err(_) => {
                // Never panic on a sensor read error.
                set_accel_reading(AccelReading::disconnected());
                initialized = false;
                println!("Accelerometer read failed; retrying...");
            }
        }

        embassy_time::Timer::after_millis(500).await;
    }
}

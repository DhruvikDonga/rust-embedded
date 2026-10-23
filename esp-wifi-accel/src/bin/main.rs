#![no_std]
#![no_main]

extern crate alloc;


use embassy_executor::Spawner;
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock, i2c::master::{Config, I2c}, interrupt::software::SoftwareInterruptControl, time::Rate, timer::timg::TimerGroup,
};
use esp_println::println;
use esp_radio::wifi::{
    sta::StationConfig, Config as WifiConfig, ControllerConfig,
};
use esp_wifi_accel::conn;
use esp_wifi_accel::accel_setup;
esp_bootloader_esp_idf::esp_app_desc!();


const WIFI_SSID: &str = "Redmi 12 5G";
const WIFI_PASSWORD: &str = "mpatel98";

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let hal_config =
        esp_hal::Config::default().with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(hal_config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    // Start the ESP RTOS/Embassy runtime.
    let sw_int =
        SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
    let timers = TimerGroup::new(peripherals.TIMG0);

    esp_rtos::start(
        timers.timer0,
        sw_int.software_interrupt0,
    );

    println!("Starting ESP32-S3 web server...");

    // Configure Wi-Fi station mode.
    let wifi_config = WifiConfig::Station(
        StationConfig::default()
            .with_ssid(WIFI_SSID)
            .with_password(WIFI_PASSWORD.into()),
    );

    let (controller, interfaces) = esp_radio::wifi::new(
        peripherals.WIFI,
        ControllerConfig::default()
            .with_initial_config(wifi_config),
    )
    .expect("Wi-Fi initialization failed");


    // Setup the independent accelerometer task.
    // GPIO1 = SCL; GPIO2 = SDA, matching the code you supplied.
    let i2c = I2c::new(
    peripherals.I2C0,
    Config::default()
            .with_frequency(Rate::from_khz(100)),
    )
    .unwrap()
    .with_scl(peripherals.GPIO1)
    .with_sda(peripherals.GPIO2);
    
    accel_setup::accel_setup(spawner, i2c);
    conn::setup_connection(spawner, controller, interfaces).await;

    // The main task does nothing; the other tasks run independently.
    loop {
        embassy_time::Timer::after_secs(60).await;
    }
}
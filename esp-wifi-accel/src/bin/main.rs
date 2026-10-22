
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_net::{
    tcp::TcpSocket, Config as NetConfig, DhcpConfig, Runner, StackResources,
};
use embedded_io_async::{Read, Write};
use esp_backtrace as _;
use esp_hal::{
    clock::CpuClock,
    interrupt::software::SoftwareInterruptControl,
    rng::Rng,
    timer::timg::TimerGroup,
};
use esp_println::println;
use esp_radio::wifi::{
    sta::StationConfig, Config as WifiConfig, ControllerConfig,
    Interface, WifiController,
};
use static_cell::StaticCell;

esp_bootloader_esp_idf::esp_app_desc!();

// Change these to your Wi-Fi credentials.
const WIFI_SSID: &str = "Redmi 12 5G";
const WIFI_PASSWORD: &str = "mpatel98";

const HTML: &str = r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>ESP32-S3</title>
</head>
<body>
    <h1>Hello from ESP32-S3!</h1>
    <p>The ESP32 is serving this page over Wi-Fi.</p>
</body>
</html>
"#;

#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    let config = esp_hal::Config::default()
        .with_cpu_clock(CpuClock::max());

    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(size: 72 * 1024);

    // Start the RTOS scheduler.
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

    let (controller, interfaces) =
        esp_radio::wifi::new(
            peripherals.WIFI,
            ControllerConfig::default()
                .with_initial_config(wifi_config),
        )
        .expect("Wi-Fi initialization failed");

    // Keep the controller alive for the connection task.
    static WIFI_CONTROLLER: StaticCell<
        WifiController<'static>
    > = StaticCell::new();

    let controller = WIFI_CONTROLLER.init(controller);

    // Configure DHCP so the router assigns an IP address.
    let net_config = NetConfig::dhcpv4(DhcpConfig::default());

    static RESOURCES: StaticCell<StackResources<5>> =
        StaticCell::new();

    let resources = RESOURCES.init(StackResources::new());

    let seed = Rng::new().random() as u64;

    let (stack, runner) = embassy_net::new(
        interfaces.station,
        net_config,
        resources,
        seed,
    );

    // Start Wi-Fi connection management and the network stack.
    spawner
        .spawn(connection_task(controller)
        .expect("Could not start Wi-Fi task"));

    spawner
        .spawn(net_task(runner)
        .expect("Could not start network task"));

    // Wait until DHCP provides an IPv4 address.
    println!("Waiting for Wi-Fi connection...");

    loop {
        if stack.is_link_up() {
            if let Some(config) = stack.config_v4() {
                println!("Wi-Fi connected!");
                println!("Open http://{}/ in your browser", config.address);
                break;
            }
        }

        embassy_time::Timer::after_millis(500).await;
    }

    // Serve the HTML page on port 80.
    loop {
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];

        let mut socket =
            TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        println!("Waiting for browser request...");

        if socket.accept(80).await.is_err() {
            continue;
        }

        let mut request = [0u8; 512];

        // Read the browser's HTTP request.
        let _ = socket.read(&mut request).await;

        // Send an HTTP response followed by the HTML page.
        let header =
            "HTTP/1.0 200 OK\r\nContent-Type: text/html; charset=utf-8\r\n\r\n";

        let result = async {
            socket.write_all(header.as_bytes()).await?;
            socket.write_all(HTML.as_bytes()).await?;
            socket.flush().await
        }
        .await;

        if result.is_err() {
            println!("Failed to send HTTP response");
        }

        socket.close();
        socket.abort();

        println!("HTML page served");
    }
}

#[embassy_executor::task]
async fn connection_task(
    controller: &'static mut WifiController<'static>,
) {
    loop {
        if !controller.is_connected() {
            println!("Connecting to Wi-Fi...");

            if let Err(error) = controller.connect_async().await {
                println!("Wi-Fi connection failed: {:?}", error);
                embassy_time::Timer::after_secs(5).await;
            } else {
                println!("Wi-Fi connected");
            }
        } else {
            controller.wait_for_disconnect_async().await.ok();
        }
    }
}

#[embassy_executor::task]
async fn net_task(
    mut runner: Runner<'static, Interface<'static>>,
) {
    runner.run().await;
}

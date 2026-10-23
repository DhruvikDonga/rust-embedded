#![no_std]
#![no_main]
extern crate alloc;

use esp_hal::rng::Rng;
use esp_println::println;
use esp_radio::wifi::{
    Interface, Interfaces, WifiController,
};
use static_cell::StaticCell;
use embassy_net::{
    Config as NetConfig, DhcpConfig, Runner, StackResources,
};
mod ui_view;
mod request_processor;

pub async fn setup_connection(spawner: embassy_executor::Spawner, controller: WifiController<'static>, interfaces: Interfaces<'static> ) {
    // Keep the Wi-Fi controller alive for the connection task.
    static WIFI_CONTROLLER: StaticCell<WifiController<'static>> =
        StaticCell::new();

    let controller = WIFI_CONTROLLER.init(controller);

    // Configure DHCP: the router assigns the ESP32 an IPv4 address.
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

      // Start the two long-running network tasks once.
    spawner
        .spawn(
            connection_task(controller)
                .expect("Could not start Wi-Fi task"),
        );

    spawner
        .spawn(
            net_task(runner)
                .expect("Could not start network task"),
        );
    
    // Wait for DHCP to assign an IPv4 address.
    println!("Waiting for Wi-Fi connection...");

    loop {
        if stack.is_link_up() {
            if let Some(config) = stack.config_v4() {
                println!("Wi-Fi connected!");
                println!(
                    "Open http://{}/ in your browser",
                    config.address
                );
                break;
            }
        }

        embassy_time::Timer::after_millis(500).await;
    }

    // Start the HTTP request processing task.
    spawner
        .spawn(
            request_processor::request_process(stack).expect("Failed to process request")
        );
}

// Continuously manages the Wi-Fi connection.
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
            let _ = controller.wait_for_disconnect_async().await;
        }
    }
}

// Continuously processes network traffic, TCP and DHCP.
#[embassy_executor::task]
async fn net_task(
    mut runner: Runner<'static, Interface<'static>>,
) {
    runner.run().await;
}


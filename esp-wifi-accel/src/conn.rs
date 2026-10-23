#![no_std]
#![no_main]
extern crate alloc;

use embedded_io_async::Write;
use esp_hal::rng::Rng;
use esp_println::println;
use esp_radio::wifi::{
    Interface, Interfaces, WifiController,
};
use static_cell::StaticCell;
use embassy_net::{
    tcp::TcpSocket, Config as NetConfig, DhcpConfig, Runner, StackResources,
};
use alloc::{format, string::String};
mod ui_view;
use ui_view::HTML;
use crate::accel_setup::get_accel_reading;

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

     loop {
        let mut rx_buffer = [0u8; 1024];
        let mut tx_buffer = [0u8; 1024];

        let mut socket =
            TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);

        println!("Waiting for browser request...");

        if socket.accept(80).await.is_err() {
            socket.abort();
            continue;
        }

        let mut request = [0u8; 512];

        let request_len = match socket.read(&mut request).await {
            Ok(n) => n,
            Err(_) => {
                socket.abort();
                continue;
            }
        };

        let request_text =
            core::str::from_utf8(&request[..request_len])
                .unwrap_or("");

        let (status, content_type, body): (&str, &str, String) =
            if request_text.starts_with("GET /accel ") {
                let reading = get_accel_reading();

                if reading.connected {
                    (
                        "200 OK",
                        "application/json",
                        format!(
                            concat!(
                                "{{\"connected\":true,",
                                "\"x\":{:.3},",
                                "\"y\":{:.3},",
                                "\"z\":{:.3},",
                                "\"magnitude\":{:.3}}}"
                            ),
                            reading.x,
                            reading.y,
                            reading.z,
                            reading.magnitude,
                        ),
                    )
                } else {
                    (
                        "200 OK",
                        "application/json",
                        String::from("{\"connected\":false}"),
                    )
                }
            } else if request_text.starts_with("GET / ") {
                (
                    "200 OK",
                    "text/html; charset=utf-8",
                    String::from(HTML),
                )
            } else {
                (
                    "404 Not Found",
                    "text/plain",
                    String::from("Not found"),
                )
            };

        let header = format!(
            concat!(
                "HTTP/1.1 {}\r\n",
                "Content-Type: {}\r\n",
                "Content-Length: {}\r\n",
                "Cache-Control: no-store\r\n",
                "Connection: close\r\n",
                "\r\n"
            ),
            status,
            content_type,
            body.len(),
        );

        let result = async {
            socket.write_all(header.as_bytes()).await?;
            socket.write_all(body.as_bytes()).await?;
            socket.flush().await
        }
        .await;

        if result.is_err() {
            println!("Failed to send HTTP response");
        }

        socket.abort();
    }

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
use embassy_net::{Stack, tcp::TcpSocket};
use embedded_io_async::Write;
use esp_println::println;

use crate::{accel_setup::get_accel_reading, conn::ui_view::HTML};

extern crate alloc;
use alloc::{format, string::String};

// Request process
#[embassy_executor::task]
pub async fn request_process(stack: Stack<'static>) {
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
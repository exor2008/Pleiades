use arrayvec::ArrayString;
use core::str::from_utf8;
use cyw43::{Control, NetDriver};
use defmt::*;
use embassy_net::tcp::TcpSocket;
use embassy_net::Stack;
use embassy_time::Duration;
use embedded_io::asynch::Write;
use heapless::Vec;
use numtoa::NumToA;

pub async fn run_http(mut control: Control<'_>, stack: &'static Stack<NetDriver<'static>>) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));
        // socket.set_keep_alive(None);

        control.gpio_set(0, false).await;
        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    warn!("read EOF");
                    break;
                }
                Ok(n) => {
                    debug!("Message received");
                    n
                }
                Err(e) => {
                    warn!("read error: {:?}", e);
                    break;
                }
            };

            let buf = from_utf8(&buf[..n]).unwrap();

            let http_request: Vec<&str, 4096> = buf
                .lines()
                .map(|result| result)
                .take_while(|line| !line.is_empty())
                .collect();
            defmt::info!("Request: {:#?}", http_request);

            let status = "HTTP/1.1 200 OK\r\n";
            let content = "<!DOCTYPE html>
            <html lang=\"en\">
              <head>
                <meta charset=\"utf-8\">
                <title>Hello!</title>
              </head>
              <body>
                <h1>Hello!</h1>
                <p>Hi from Rust</p>
              </body>
            </html>\r\n";
            let length = content.len();

            // let format_str = "{}\r\nContent-Length: {}\r\n\r\n{}";
            let mut buf = [0u8; 20];

            let mut text = ArrayString::<[_; 1024]>::new();
            text.push_str(status);
            text.push_str(length.numtoa_str(10, &mut buf));
            text.push_str("\r\n\r\n");
            text.push_str(content);

            match socket.write_all(text.as_bytes()).await {
                Ok(()) => {}
                Err(e) => {
                    warn!("write error: {:?}", e);
                    break;
                }
            };

            socket.flush().await.unwrap();
            socket.close();
            break;
        }
    }
}

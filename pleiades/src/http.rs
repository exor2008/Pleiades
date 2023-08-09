use arrayvec::ArrayString;
use core::str::from_utf8;
use cyw43::{Control, NetDriver};
use defmt::*;
use embassy_net::tcp::TcpSocket;
use embassy_net::Stack;
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use heapless::Vec;
use numtoa::NumToA;

pub async fn run_http<'a>(
    mut control: Control<'a>,
    stack: &'static Stack<NetDriver<'static>>,
    firmware: &'static Channel<ThreadModeRawMutex, Vec<u8, 4096>, 1>,
) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(25)));
        socket.set_keep_alive(None);

        control.gpio_set(0, false).await;
        info!("Listening on TCP:1234...");
        if let Err(e) = socket.accept(1234).await {
            warn!("accept error: {:?}", e);
            continue;
        }

        info!("Received connection from {:?}", socket.remote_endpoint());
        control.gpio_set(0, true).await;

        let n = match socket.read(&mut buf).await {
            Ok(0) => {
                warn!("read EOF");
                continue;
            }
            Ok(n) => {
                debug!("Message received, len: {}", n);
                n
            }
            Err(e) => {
                warn!("read error: {:?}", e);
                continue;
            }
        };

        let buf = match from_utf8(&buf[..n]) {
            Ok(buf) => buf,
            Err(_) => {
                warn!("Error while parsing http request");
                continue;
            }
        };

        let http_request: Vec<&str, 4096> = buf
            .lines()
            .map(|result| result)
            .take_while(|line| !line.is_empty())
            .collect();
        defmt::info!("Request: {:#?}", http_request);

        let mut start_update = false;

        let text = match http_request[0] {
            "GET / HTTP/1.1" => index_hanlder(),
            "POST /upload/ HTTP/1.1" => {
                start_update = true;
                upload_hanlder().await
            }
            _ => {
                warn!("Error 404 {}", http_request[0]);
                err404_hanlder().await
            }
        };

        let sent = match socket.write(text.as_bytes()).await {
            Ok(n) => {
                info!("responce sent");
                n
            }
            Err(e) => {
                warn!("write error: {:?}", e);
                continue;
            }
        };

        info!("Sent bytes {}", sent);

        socket.flush().await.unwrap();

        if start_update {
            let content_lenght = http_request
                .iter()
                .find(|header| header.starts_with("Content-Length"))
                .and_then(|header| header.split(":").nth(1))
                .and_then(|length| length.trim().parse::<usize>().ok())
                .unwrap_or(0);

            let mut bytes_received = 0;
            let mut buf = [0; 4096];

            // Signal to start flashing
            defmt::info!("Sending start flash signal");
            firmware.send(Vec::default()).await;
            info!("Firmware size {}", content_lenght);

            //TODO: stop flashing in case of socket error
            while bytes_received < content_lenght {
                match socket.read(&mut buf).await {
                    Ok(0) => {
                        warn!("Connection closed prematurely");
                        break;
                    }
                    Ok(n) => {
                        bytes_received += n;
                        // info!("Got the next chunk");
                        firmware.send(Vec::from_slice(&buf[..n]).unwrap()).await;
                        // info!("bytes_received {} {}", n, bytes_received)
                    }
                    Err(_) => {
                        error!("Error occurred while receiving data");
                        break;
                    }
                }
            }

            // Signal to stop flashing
            defmt::info!("Sending stop flash signal");
            firmware.send(Vec::default()).await;
        }
        socket.close();
    }
}

fn index_hanlder() -> ArrayString<[u8; 1024]> {
    let status = "HTTP/1.1 200 OK\r\n";
    let content = "<!DOCTYPE html>
    <html>
    <head>
        <title>File Upload</title>
    </head>
    <body>
        <h1>File Upload</h1>
        <form action=\"/upload/\" method=\"post\" enctype=\"multipart/form-data\">
            <input type=\"file\" name=\"file\" />
            <button type=\"submit\">Upload</button>
        </form>
    </body>
    </html>
    \r\n";
    let length = content.len();

    // let format_str = "{}\r\nContent-Length: {}\r\n\r\n{}";
    let mut buf = [0u8; 20];

    let mut text = ArrayString::<[_; 1024]>::new();
    text.push_str(status);
    text.push_str("Content-Type: text/html\r\n");
    text.push_str("Content-Length: ");
    text.push_str(length.numtoa_str(10, &mut buf));
    text.push_str("\r\n\r\n");
    text.push_str(content);

    text
}

async fn upload_hanlder<'a>() -> ArrayString<[u8; 1024]> {
    let status = "HTTP/1.1 200 OK\r\n";
    let content = "<!DOCTYPE html>
    <html>
    <head>
        <title>Firmware upload</title>
    </head>
    <body>
        Starting to flash firmware
    </body>
    </html>
    \r\n";

    let length = content.len();
    let mut buf = [0u8; 20];
    let mut text = ArrayString::<[_; 1024]>::new();

    text.push_str(status);
    text.push_str("Content-Type: text/html\r\n");
    text.push_str("Content-Length: ");
    text.push_str(length.numtoa_str(10, &mut buf));
    text.push_str("\r\n\r\n");
    text.push_str(content);

    // let mut buf = [0u8; 1024 * 4];
    info!("Start Reading File");

    text
}

async fn err404_hanlder() -> ArrayString<[u8; 1024]> {
    let status = "HTTP/1.1 200 OK\r\n";
    let content = "<!DOCTYPE html>
    <html>
    <head>
        <title>Firmware upload</title>
    </head>
    <body>
        Starting to flash firmware
    </body>
    </html>
    \r\n";

    let length = content.len();
    let mut buf = [0u8; 20];

    let mut text = ArrayString::<[_; 1024]>::new();
    text.push_str(status);
    text.push_str("Content-Type: text/html\r\n");
    text.push_str("Content-Length: ");
    text.push_str(length.numtoa_str(10, &mut buf));
    text.push_str("\r\n\r\n");
    text.push_str(content);

    text
}

// pub struct TcpSocketIterator<'a> {
//     socket: &'a mut TcpSocket<'a>,
//     content_lenght: usize,
//     downloaded: usize,
//     buf: [u8; 4096],
// }

// impl<'a> TcpSocketIterator<'a> {
//     fn new(socket: &'a mut TcpSocket<'a>, content_lenght: usize) -> Self {
//         TcpSocketIterator {
//             socket,
//             content_lenght,
//             downloaded: Default::default(),
//             buf: [0u8; 4096],
//         }
//     }
// }

// impl<'a> Iterator for TcpSocketIterator<'a> {
//     type Item = Vec<u8, 4096>;

//     async fn next(&mut self) -> Option<Self::Item> {
//         if self.downloaded >= self.content_lenght {
//             return None;
//         }
//         let n = self.socket.read(&mut self.buf).await.unwrap();
//         let vec = Vec::from_slice(&self.buf[..n]).unwrap();
//         self.downloaded += n;

//         Some(vec)
//     }
// }

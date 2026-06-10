use std::{env, net::SocketAddr, process, str::FromStr};

use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() {
    let mut args = env::args();

    println!("args: {:?}", args);
    args.next();
    for arg in args {
        let addr = format!("127.0.0.1:{}", arg);
        let sockaddr = if let Ok(s) = SocketAddr::from_str(&addr) {
            s
        } else {
            println!("Invalid Address: {}", addr);
            process::exit(1);
        };
        println!("Rust Server running on {}", addr);
        tokio::spawn(async move {
            let listener = TcpListener::bind(sockaddr).await.unwrap();
            loop {
                let (mut stream, _) = match listener.accept().await {
                    Ok(s) => s,
                    Err(e) => {
                        println!("Err: {}", e);
                        continue;
                    }
                };
                _ = stream.set_nodelay(true);
                tokio::spawn(async move {
                    let mut scratch = [0u8; 1024];
                    let response: &[u8] = b"HTTP/1.1 200 OK\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 11\r\n\
                        Connection: keep-alive\r\n\
                        \r\n\
                        Hello World";

                    loop {
                        match stream.read(&mut scratch).await {
                            Ok(0) => break,
                            Ok(_) => {
                                _ = stream.write_all(response).await;
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                });
            }
        });
    }
    _ = tokio::signal::ctrl_c().await;
}

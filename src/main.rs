use std::{
    env,
    net::SocketAddr,
    process,
    str::FromStr,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

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
        let ttlcn = Arc::new(AtomicUsize::new(0));
        println!("Rust Server running on {}", addr);
        let ttlcn_clone = Arc::clone(&ttlcn);
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
                let ttlcn_clone = Arc::clone(&ttlcn_clone);
                tokio::spawn(async move {
                    let mut scratch = [0u8; 1024];
                    let response: &[u8] = b"HTTP/1.1 200 OK\r\n\
                        Content-Type: text/plain\r\n\
                        Content-Length: 12\r\n\
                        Connection: keep-alive\r\n\
                        \r\n\
                        Hello World\n";

                    loop {
                        match stream.read(&mut scratch).await {
                            Ok(0) => break,
                            Ok(_) => {
                                ttlcn_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
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
        tokio::spawn(async move {
            let mut last_cn = 0usize;
            loop {
                let curcn = ttlcn.load(Ordering::SeqCst);
                if curcn != 0 && curcn != last_cn {
                    println!("Total Response: {} [+{}]", curcn, curcn - last_cn);
                    last_cn = curcn;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
    }
    _ = tokio::signal::ctrl_c().await;
}

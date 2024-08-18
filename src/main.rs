use dotenv::dotenv;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    let host = env::var("HOST").expect("failed to read the HOST env variavle");
    let port = env::var("PORT").expect("failed to read the PORT env variable");

    let addr = format!("{host}:{port}");

    let listener = TcpListener::bind(addr.clone())
        .await
        .expect("failed to bind to specified address");

    println!("server is listening on {}", addr);

    let (tx, _) = broadcast::channel::<(String, SocketAddr)>(32);

    let mut client_count = 1;

    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("{} client is connected", client_count);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let client_id = client_count;
            loop {
                let mut buffer = [0u8; 1024];
                tokio::select! {
                    Ok(number_of_bytes_read)= socket.read(&mut buffer) => {
                        if number_of_bytes_read == 0 {
                            println!("{} was disconnected",client_id);
                            break;
                        }
                        let message=String::from_utf8_lossy(&buffer);
                        tx.send((message.to_string(),addr)).unwrap();
                    }
                    Ok(result) = rx.recv() => {
                        let (message,other_addr)=result;
                        if addr != other_addr {
                            socket.write_all(message.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });

        client_count += 1;
    }
}

use std::net::{Ipv4Addr, SocketAddr};
use tokio::{net::TcpListener, io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt}};
use tokio::sync::broadcast;
use std::path::Path;
use serde::Deserialize;
use serde_json;

use std::net::TcpStream;


#[derive(Deserialize, Debug)]
pub struct Settings {
    pub ipv4: Ipv4Addr,
    pub port: u32,
}


struct Server {
    ipv4: Ipv4Addr,
    port: u32,
}


#[tokio::main]
async fn main() {
    // TODO make this part of the config.
    let config_path = Path::new("config/init.yml").to_str().unwrap();

    let config: Settings = get_config(config_path).unwrap_or(Settings { ipv4: Ipv4Addr::new(127,0,0,1), port: 7878 });
    let bind_string = format!("{}:{}", config.ipv4, config.port);
    let listener = TcpListener::bind(bind_string).await.unwrap();
    let (tx, _) = broadcast::channel(10);

    loop {
        // Wait for a new connection.
        let (mut socket, address) = listener.accept().await.unwrap();
        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            // The socket is split into a reader and writer
            let (read, mut writer) = socket.split();
            // A smart buffer is generated to help facilitate reading off the socket.
            let mut reader = BufReader::new(read);
            let mut line = String::new();
            loop {

                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        if result.unwrap() == 0 {
                            break;
                        }
                        tx.send((line.clone(), address)).unwrap();
                        line.clear();
                    }
                    result = rx.recv() => {
                        let (msg, other_address) = result.unwrap();
                        if address != other_address {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}

pub fn get_config<'a>(path: &'a str) -> Result<Settings, config::ConfigError> {
    config::Config::builder()
        .add_source(config::File::with_name(path))
        .build()?
        .try_deserialize::<Settings>()
}

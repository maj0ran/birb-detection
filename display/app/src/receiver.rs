use bird_display_lib::error::{EInkError, Result};
use serde::Deserialize;
use smol::io::{AsyncBufReadExt, BufReader};
use smol::net::TcpStream;

#[derive(Debug, Deserialize, Clone)]
pub struct BirdData {
    pub name: String,
}

pub struct Receiver {
    addr: String,
}

impl Receiver {
    pub fn new(addr: &str) -> Self {
        Self {
            addr: addr.to_string(),
        }
    }

    pub async fn connect(&self) -> Result<TcpStream> {
        TcpStream::connect(&self.addr)
            .await
            .map_err(|e| EInkError::Generic(format!("Failed to connect to {}: {}", self.addr, e)))
    }

    pub async fn run<F>(&self, mut on_data: F) -> Result<()>
    where
        F: FnMut(BirdData),
    {
        loop {
            log::info!("Connecting to {}...", self.addr);
            match self.connect().await {
                Ok(stream) => {
                    log::info!("Connected to {}", self.addr);
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    loop {
                        line.clear();
                        match reader.read_line(&mut line).await {
                            Ok(0) => {
                                log::warn!("Connection closed by server");
                                break;
                            }
                            Ok(_) => match serde_json::from_str::<BirdData>(&line) {
                                Ok(data) => on_data(data),
                                Err(e) => log::error!("Failed to deserialize BirdData: {}", e),
                            },
                            Err(e) => {
                                log::error!("Error reading from stream: {}", e);
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Connection error: {}. Retrying in 5 seconds...", e);
                    smol::Timer::after(std::time::Duration::from_secs(5)).await;
                }
            }
        }
    }
}

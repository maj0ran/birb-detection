use bird_display_lib::error::{EInkError, Result};
use smol::io::{AsyncReadExt, BufReader};
use smol::net::TcpStream;

#[derive(Debug, Clone)]
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
                    loop {
                        let mut length_bytes = [0u8; 4];
                        if let Err(e) = reader.read_exact(&mut length_bytes).await {
                            log::warn!("Failed to read bird name length: {}", e);
                            break;
                        }

                        let length = u32::from_be_bytes(length_bytes) as usize;
                        let mut name_bytes = vec![0u8; length];
                        if let Err(e) = reader.read_exact(&mut name_bytes).await {
                            log::warn!("Failed to read bird name: {}", e);
                            break;
                        }

                        match String::from_utf8(name_bytes) {
                            Ok(name) => on_data(BirdData { name }),
                            Err(e) => {
                                log::error!("Invalid UTF-8 in bird name: {}", e);
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

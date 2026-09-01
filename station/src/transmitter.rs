use crate::bird_detection::BirdName;
use crate::error::Result;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};

pub struct Transmitter {
    tx: UnboundedSender<BirdName>,
}

const PORT: u16 = 8128;

impl Transmitter {
    pub async fn new() -> Result<Self> {
        let (tx, rx) = unbounded_channel::<BirdName>();
        let clients = Arc::new(Mutex::new(Vec::new()));

        log::info!("start listening on port {}.", PORT);
        let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;

        let clients_server = clients.clone();
        tokio::spawn(async move {
            // Spawn a task to accept incoming connections.
            // This task needs access to the clients-vector, so we give it its own mutex'd handle.
            let clients_accept = clients_server.clone();
            tokio::spawn(async move {
                loop {
                    match listener.accept().await {
                        Ok((socket, addr)) => {
                            log::info!("New client connected! (IP: {})", addr);
                            clients_accept.lock().await.push(socket);
                        }
                        Err(e) => {
                            log::error!("Error accepting connection from IP {}", e);
                        }
                    }
                }
            });

            // Main loop to broadcast data received from the channel
            let mut rx = rx;
            while let Some(bird_data) = rx.recv().await {
                let name = bird_data.as_bytes();
                let length = match u32::try_from(name.len()) {
                    Ok(length) => length,
                    Err(_) => {
                        log::error!("Bird name is too long to transmit");
                        continue;
                    }
                };

                let mut payload = Vec::with_capacity(size_of::<u32>() + name.len());
                payload.extend_from_slice(&length.to_be_bytes());
                payload.extend_from_slice(name);

                let mut clients = clients_server.lock().await;
                let mut disconnected = Vec::new();

                for (i, client) in clients.iter_mut().enumerate() {
                    if let Err(e) = client.write_all(&payload).await {
                        // We consider failed writes a disconnected client. We cannot remove clients
                        // from the vec while iterating over it, so track their indexes first.
                        log::warn!("Failed to send data to client: {}", e);
                        disconnected.push(i);
                    }
                }

                for i in disconnected.into_iter().rev() {
                    clients.remove(i);
                }
            }

            log::info!("Transmitter channel closed; server stopped");
        });

        Ok(Self { tx })
    }

    pub fn send(&self, data: BirdName) -> Result<()> {
        self.tx.send(data)?;
        Ok(())
    }
}

use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::sync::Mutex;

use crate::bird_parser::BirdData;
use crate::error::Result;

pub struct Transmitter {
    clients: Arc<Mutex<Vec<TcpStream>>>,
    rx: UnboundedReceiver<BirdData>,
}

const PORT: u16 = 8128;

impl Transmitter {
    pub fn new() -> (Self, UnboundedSender<BirdData>) {
        let (tx, rx) = unbounded_channel();
        (
            Self {
                clients: Arc::new(Mutex::new(Vec::new())),
                rx,
            },
            tx,
        )
    }

    pub async fn run(mut self) -> Result<()> {
        log::info!("start listening on port {}.", PORT);

        let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT)).await?;

        // Spawn a task to accept incoming connections.
        // This task needs access to the clients-vector, so we give it its own mutex'd handle.
        let clients_accept = self.clients.clone();
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((socket, addr)) => {
                        log::info!("New client connected: {}", addr);
                        let mut clients = clients_accept.lock().await;
                        clients.push(socket);
                    }
                    Err(e) => {
                        log::error!("Error accepting connection: {}", e);
                    }
                }
            }
        });

        // Main loop to broadcast data received from the channel
        loop {
            let bird_data =
                self.rx.recv().await.ok_or_else(|| {
                    crate::error::BirdError::Generic("Channel closed".to_string())
                })?;

            let payload = serde_json::to_vec(&bird_data).map_err(|e| {
                crate::error::BirdError::Generic(format!("Serialization error: {}", e))
            })?;

            let mut payload_with_newline = payload;
            payload_with_newline.push(b'\n');

            let mut clients = self.clients.lock().await;
            let mut disconnected = Vec::new();

            for (i, client) in clients.iter_mut().enumerate() {
                if let Err(e) = client.write_all(&payload_with_newline).await {
                    // we consider failed writes a disconnected client. We cannot remove clients from the ved
                    // while iterating over it, so we keep track of those in a separate vector, and after
                    // iterating, we can go through the disconnected clients and remove them from our list.
                    log::warn!("Failed to send data to client: {}", e);
                    disconnected.push(i);
                }
            }

            for i in disconnected.into_iter() {
                clients.remove(i);
            }
        }
    }
}

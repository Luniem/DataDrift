use std::sync::Arc;

use async_std::{net::TcpStream, sync::Mutex};
use async_tungstenite::{async_std::connect_async, tungstenite::Message, WebSocketStream};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use futures::{stream::SplitSink, SinkExt, StreamExt};
use shared::models::network_message::NetworkMessage;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::BACKEND_WEBSOCKET_URL;

#[derive(Resource)]
pub struct UnboundedReceiverResource {
    pub receiver: UnboundedReceiver<NetworkMessage>,
}

#[derive(Resource)]
pub struct NetworkClient {
    socket_url: String,
    write_socket: Arc<Mutex<Option<SplitSink<WebSocketStream<TcpStream>, Message>>>>,
    unbounded_sender: Arc<UnboundedSender<NetworkMessage>>,
}

impl NetworkClient {
    /// Creating a new network-client with the given URL.
    pub fn new(websocket_url: String, unbounded_sender: UnboundedSender<NetworkMessage>) -> Self {
        NetworkClient {
            socket_url: websocket_url,
            write_socket: Arc::new(Mutex::new(None)),
            unbounded_sender: Arc::new(unbounded_sender),
        }
    }

    /// Will connect to the websocket.
    /// If it is not able to connect, we will panic for the sake of simplicity.
    pub fn connect(&mut self) -> () {
        // spawn a new thread to create the websocket-connection and handle incoming messages
        // save the write-stream in the network-client
        let cloned_socket_url = self.socket_url.clone();
        let cloned_write_socket = Arc::clone(&self.write_socket);
        let cloned_sender = Arc::clone(&self.unbounded_sender);

        let task_pool = AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                let (ws_stream, _) = connect_async(cloned_socket_url)
                    .await
                    .expect("Failed to connect to the Websocket!");

                let (write_stream, mut read_stream) = ws_stream.split();

                // save the write-stream in the network-client and drop the lock
                {
                    let mut write_socket = cloned_write_socket.lock().await;
                    *write_socket = Some(write_stream);
                }

                // handle incoming messages and send them to the unbounded-channel
                loop {
                    let msg: Message = read_stream.next().await.unwrap().unwrap();

                    // let msg = read.stream.next().await.unwrap().unwrap();
                    let deserialized: NetworkMessage =
                        serde_json::from_str(msg.to_text().unwrap()).unwrap();
                    cloned_sender.send(deserialized).unwrap();
                }
            })
            .detach();
    }

    pub fn send_message(&self, message: NetworkMessage) -> () {
        // spawn an async call and don't wait on it to send the message
        let cloned_write_socket = Arc::clone(&self.write_socket);
        let task_pool = AsyncComputeTaskPool::get();

        task_pool
            .spawn(async move {
                let mut write_socket_lock = cloned_write_socket.lock().await;
                if let Some(write) = &mut *write_socket_lock {
                    write
                        .send(Message::text(serde_json::to_string(&message).unwrap()))
                        .await
                        .unwrap();
                }
            })
            .detach();
    }

    pub fn disconnect(&mut self) -> () {
        //TODO: Implement disconnect
        // if let Some(socket) = self.socket.as_mut() {
        //     let mut locked_socket = socket.lock().unwrap();
        //     let _ = locked_socket.close(None);
        // }
    }
}

/// Here we setup all the necessary stuff for properly connect to the websocket.
/// We open up an unbounded-channel, so the network-client is able to notify the methods inside the bevy-loop about new websocket-messages from the backend.
/// We are creating the network-client, give him the sender of the unbounded-channel and connect to the websocket.
/// Then we are registering the client as well as the receiver of the unbounded-channel as bevy-resource.
pub fn setup_network_client(mut commands: Commands) {
    // create the mpsc-channel
    let (sender, receiver) = unbounded_channel::<NetworkMessage>();

    // create network-client and put it into resources
    let mut client = NetworkClient::new(BACKEND_WEBSOCKET_URL.to_string(), sender);
    client.connect();

    commands.insert_resource(client);
    commands.insert_resource(UnboundedReceiverResource { receiver });
}

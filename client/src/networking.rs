use std::{
    net::TcpStream,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use async_tungstenite::tungstenite::{client, connect, stream::MaybeTlsStream, Message, WebSocket};
use bevy::prelude::*;
use bevy::tasks::AsyncComputeTaskPool;
use shared::models::network_message::NetworkMessage;
use tokio::{
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
    time::interval,
};

use crate::BACKEND_WEBSOCKET_URL;

#[derive(Resource)]
pub struct UnboundedReceiverResource {
    pub receiver: UnboundedReceiver<NetworkMessage>,
}

#[derive(Resource)]
pub struct NetworkClient {
    socket_url: String,
    socket: Option<Arc<Mutex<WebSocket<mio::net::TcpStream>>>>,
    unbounded_sender: Arc<UnboundedSender<NetworkMessage>>,
}

impl NetworkClient {
    /// Creating a new network-client with the given URL.
    pub fn new(websocket_url: String, unbounded_sender: UnboundedSender<NetworkMessage>) -> Self {
        NetworkClient {
            socket_url: websocket_url,
            socket: None,
            unbounded_sender: Arc::new(unbounded_sender),
        }
    }

    /// Will connect to the websocket.
    /// If it is not able to connect, we will panic for the sake of simplicity.
    pub fn connect(&mut self) -> () {
        // don't connect if we already have a socket open
        if self.socket.is_some() {
            return;
        }

        let noraml_tc = TcpStream::connect("127.0.0.1:9001").unwrap();
        let tcp_stream = mio::net::TcpStream::from_std(noraml_tc);
        let (socket, _) = client(self.socket_url.clone(), tcp_stream).unwrap();
        // let (socket, _) = connect(&self.socket_url).expect("Failed to connect to the Websocket!");

        let rc_socket = Arc::new(Mutex::new(socket));

        let cloned_socket = Arc::clone(&rc_socket);
        let cloned_unbounded_sender = Arc::clone(&self.unbounded_sender);
        let task_pool = AsyncComputeTaskPool::get();
        task_pool
            .spawn(async move {
                loop {
                    // let the thread sleep for some time so it does not hug the lock on the socket
                    async_std::task::sleep(Duration::from_millis(20)).await;
                    {
                        let mut locked_socket = cloned_socket.lock().unwrap();
                        println!("Test");
                        let msg = locked_socket.read_message();
                        println!("Test2");

                        match msg {
                            Ok(websocket_message) => {
                                let deserialized: NetworkMessage =
                                    serde_json::from_str(websocket_message.to_text().unwrap())
                                        .unwrap();
                                println!("Ship it!");
                                cloned_unbounded_sender.send(deserialized).unwrap();
                            }
                            Err(_) => break, // leave the loop and kill the thread
                        };
                    }
                }
            })
            .detach();

        self.socket = Some(rc_socket);
    }

    pub fn send_message(&mut self, message: NetworkMessage) -> () {
        if let Some(socket) = &self.socket {
            let mut locked_socket = socket.lock().unwrap();
            let _ = locked_socket
                .write_message(Message::text(serde_json::to_string(&message).unwrap()));
        }
    }

    pub fn disconnect(&mut self) -> () {
        if let Some(socket) = self.socket.as_mut() {
            let mut locked_socket = socket.lock().unwrap();
            let _ = locked_socket.close(None);
        }
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

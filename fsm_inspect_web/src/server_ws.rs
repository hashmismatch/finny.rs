extern crate websocket;

use std::thread;
use websocket::{Message, OwnedMessage};
use websocket::sync::Server;

use server_state::*;

#[derive(Clone)]
pub enum WsMessage {
	WsData(OwnedMessage),
	FsmData(MessageToBrowserClient)
}

pub fn spawn_ws(data: InspectShared) {
	let server = Server::bind("0.0.0.0:9002").unwrap();

	for connection in server.filter_map(Result::ok) {
		let data = data.clone();

		thread::spawn(move || {
			let client = connection.accept().unwrap();

			use std::sync::mpsc::channel;

			let (mut ws_receiver, mut ws_sender) = client.split().unwrap();
			let (msg_sender, msg_receiver) = channel::<WsMessage>();

			data.add_ws_client(msg_sender.clone());

			// ws sender thread
			{				
				thread::spawn(move || {
					for msg in msg_receiver.iter() {
						let send_status = match msg {
							WsMessage::WsData(owned) => {
								ws_sender.send_message(&owned)
							},
							WsMessage::FsmData(fsm) => {
								let json = fsm.to_ws_json();
								let msg = OwnedMessage::Text(json);
								ws_sender.send_message(&msg)
							}
						};

						if send_status.is_err() {
							return;
						}
					}
				});
			}

			// ws receiver thread
			for message in ws_receiver.incoming_messages() {
				/*
				let message = match message {
					Ok(message) => message,
					Err(e) => {
						println!("{:?}", e);
						let _ = sender.send_message(&Message::close());
						return;
					}
				};
				*/

				let message = match message {
					Ok(message) => message,
					Err(e) => {
						println!("error ws receive {:?}", e);
						//let _ = sender.send_message(&Message::close());
						return;
					}
				};

				match message {
					/*
					OwnedMessage::Text(txt) => {
						sender.send_message(&OwnedMessage::Text(txt)).unwrap()
					}
					OwnedMessage::Binary(bin) => {
						sender.send_message(&OwnedMessage::Binary(bin)).unwrap()
					}
					*/
					OwnedMessage::Close(_) => {
						let msg = WsMessage::WsData(OwnedMessage::Close(None));
						msg_sender.send(msg);
						return;
					}
					OwnedMessage::Ping(data) => {
						//sender.send_message(&OwnedMessage::Pong(data)).unwrap();
						let msg = WsMessage::WsData(OwnedMessage::Pong(data));
						msg_sender.send(msg);
						return;
					}
					_ => (),
				}
			}
		});
	}
}
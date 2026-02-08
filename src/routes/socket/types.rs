use axum::extract::ws::Message;
use serde::{Deserialize, de::Error};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct SocketMessage {
    pub from_email: String,
    pub from_token: String,
    pub from_device: String, // what if this is spoofed with to device id
    pub to_email: String,
    pub to_device: String,
    pub event: String,
    pub message_type: String,
    pub payload: Value,
}

impl SocketMessage {
    pub fn parse_message(message: Message) -> Result<Self, serde_json::Error> {
        match message {
            Message::Text(text) => {
                let socket_msg: SocketMessage = serde_json::from_str(&text)?;
                Ok(socket_msg)
            }
            Message::Binary(bin) => {
                let socket_msg: SocketMessage = serde_json::from_slice(&bin)?;
                Ok(socket_msg)
            }

            _ => Err(serde_json::Error::custom("invalid event type")),
        }
    }
}

// user story:
// user enters into socket now user will give a device id -yes , and a token <- thats it
// // this we will call register event
// then users checks who all are in the system
// // check event
// then user clicks a system wwhich he want to access
// // try_connect event
// then users gets a to_device to connect to
// // sdp sharing event
// // followed by a adp answer and ice share event
// now system is completed

// here is the idea
// once we have a user before we where haing a user_email -> socket
// now issue is that once we have multiple pod now we will have to have redis pub sub to send
// messages to different pods and ask if target_user's email is here if yes then give this message
// to that user
// once if that user is found a message will be sent to it
//
// on system level
// mapping 1 : {"user_email@mail.com": { device_id: { socket_id: "49fd1ed5-0024-410c-99a5-f60163d83f1b",device_info } } }
// mapping 2 : { socket_id: socket }
//
// now on redis level
// 1) if some message comes from redis check that message from redis pub sub
// 2) check email on that pod if found check if device id found if yes now send the message to that
//    socket

use axum::extract::ws::Message;
use serde::{Deserialize, Serialize, de::Error};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocketMessage {
    pub from_email: String,
    pub from_token: String,
    pub from_device: String,
    pub to_email: String,
    pub to_device: String,
    pub event: String,
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

    pub fn validate(&self) -> Result<(), String> {
        match self.event.as_str() {
            "register" => {
                if self.from_email.is_empty() {
                    return Err("from_email is required for register".to_string());
                }
                if self.from_token.is_empty() {
                    return Err("from_token is required for register".to_string());
                }
                if self.from_device.is_empty() {
                    return Err("from_device is required for register".to_string());
                }
            }
            "check" => {
                if self.from_email.is_empty() {
                    return Err("from_email is required for check".to_string());
                }
            }
            "connect" => {
                if self.from_email.is_empty() {
                    return Err("from_email is required for connect".to_string());
                }
            }
            "try_connect" | "sdp_offer" | "sdp_answer" | "ice_candidate" => {
                if self.from_email.is_empty() {
                    return Err("from_email is required".to_string());
                }
                if self.to_email.is_empty() {
                    return Err("to_email is required".to_string());
                }
                if self.to_device.is_empty() {
                    return Err("to_device is required".to_string());
                }
                if self.from_device.is_empty() {
                    return Err("from_device is required".to_string());
                }
            }
            "ping" | "pong" | "disconnect" => {
                // No validation needed
            }
            _ => {
                return Err(format!("Unknown event type: {}", self.event));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub socket_id: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisMessage {
    pub target_email: String,
    pub target_device: String,
    pub socket_message: SocketMessage,
    pub sender_pod: Option<String>,
    pub timestamp: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDevicesResponse {
    pub email: String,
    pub devices: Vec<DeviceInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub event: String,
    pub error: String,
    pub target_email: Option<String>,
    pub target_device: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub event: String,
    pub timestamp: u64,
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

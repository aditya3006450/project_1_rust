# WebSocket API Documentation

## Overview
This WebSocket API enables real-time communication between devices using a multi-pod architecture with Redis pub/sub for cross-pod messaging.

## Connection

### Endpoint
```
ws://host:3000/socket/
```

### Connection Lifecycle
1. Client opens WebSocket connection
2. Server generates unique `socket_id` for the connection
3. Client sends `register` event to authenticate
4. Server validates and stores device presence
5. Client can now send/receive messages
6. On disconnect, server automatically cleans up all mappings

## Events

### 1. Register Event
Registers a device after connection.

**Purpose**: Authenticate the user and register the device

**When to send**: Immediately after WebSocket connection is established

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "uuid-token-from-login",
  "from_device": "device-unique-id",
  "to_email": "",
  "to_device": "",
  "event": "register",
  "payload": {
    "device_name": "My Laptop",
    "device_type": "desktop"
  }
}
```

**Field Descriptions**:
- `from_email`: User's email (must match token)
- `from_token`: UUID token received during login
- `from_device`: Unique device identifier (e.g., browser fingerprint)
- `payload.device_name`: Human-readable device name (optional)
- `payload.device_type`: Device type - "desktop", "mobile", "tablet" (optional)

**Success Response**:
```json
{
  "event": "register",
  "status": "ok",
  "socket_id": "49fd1ed5-0024-410c-99a5-f60163d83f1b"
}
```

**Error Response**:
```json
{
  "event": "register",
  "status": "error",
  "error": "Invalid or expired token"
}
```

**Edge Cases Handled**:
- Same device reconnecting: Old socket is cleaned up automatically
- Invalid token: Connection rejected
- Email mismatch: Connection rejected
- Redis unavailable: Falls back to local-only mode

---

### 2. Check Event
Retrieves all online users this user is connected to.

**Purpose**: Get list of available devices to connect to

**When to send**: After registration, or periodically to refresh device list

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "",
  "to_email": "",
  "to_device": "",
  "event": "check",
  "payload": {}
}
```

**Success Response**:
```json
[
  {
    "email": "friend@example.com",
    "devices": [
      {
        "socket_id": "49fd1ed5-0024-410c-99a5-f60163d83f1b",
        "device_name": "Work Laptop",
        "device_type": "desktop"
      },
      {
        "socket_id": "abc123",
        "device_name": "iPhone",
        "device_type": "mobile"
      }
    ]
  }
]
```

**Error Response**: Returns empty array `[]` if no users found or error

**Notes**:
- Only returns users who have accepted connection requests (from UserConnection table)
- Includes devices from all pods via Redis
- Falls back to local data if Redis unavailable

---

### 3. Connect Event
Simple connection acknowledgment.

**Purpose**: Verify socket is alive and ready

**When to send**: After registration to confirm connection

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "",
  "to_email": "",
  "to_device": "",
  "event": "connect",
  "payload": {}
}
```

**Success Response**:
```json
{
  "event": "connected",
  "status": "ok"
}
```

---

### 4. Ping Event (Heartbeat)
Keeps connection alive and detects disconnections.

**Purpose**: Prevent timeout and detect zombie connections

**When to send**: Every 30 seconds

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "",
  "to_email": "",
  "to_device": "",
  "event": "ping",
  "payload": {}
}
```

**Success Response**:
```json
{
  "event": "pong",
  "timestamp": 1707417600
}
```

**Notes**:
- Client should send ping every 30 seconds
- If no ping received for 60+ seconds, server may disconnect
- Timestamp is Unix epoch in seconds

---

### 5. Try Connect Event
Initiates connection request to another device.

**Purpose**: Request to start WebRTC connection with target device

**When to send**: User clicks on a device to connect

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "my-device-id",
  "to_email": "friend@example.com",
  "to_device": "friend-device-id",
  "event": "try_connect",
  "payload": {
    "request_id": "unique-request-id"
  }
}
```

**Routing**:
1. Server checks local connections first
2. If not found locally, publishes to Redis for other pods
3. Target pod receives and forwards to target device
4. If target not found within 5 seconds, sends error to sender

**Success**: Message forwarded to target device (no direct response to sender)

**Error Response** (if target not found):
```json
{
  "event": "target_not_found",
  "error": "User friend@example.com with device friend-device-id is not online",
  "target_email": "friend@example.com",
  "target_device": "friend-device-id"
}
```

**Notes**:
- Target device receives the exact message
- Target can accept/reject and respond accordingly
- Timeout for cross-pod lookup: 5 seconds

---

### 6. SDP Offer Event
Sends WebRTC session description offer.

**Purpose**: Part of WebRTC handshake - send connection offer

**When to send**: After `try_connect` is accepted

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "my-device-id",
  "to_email": "friend@example.com",
  "to_device": "friend-device-id",
  "event": "sdp_offer",
  "payload": {
    "sdp": "v=0\no=- 1234567890 2 IN IP4 127.0.0.1\n...",
    "type": "offer"
  }
}
```

**Routing**: Same as `try_connect`

---

### 7. SDP Answer Event
Sends WebRTC session description answer.

**Purpose**: Part of WebRTC handshake - respond to offer

**When to send**: After receiving and processing `sdp_offer`

**Input**:
```json
{
  "from_email": "friend@example.com",
  "from_token": "",
  "from_device": "friend-device-id",
  "to_email": "user@example.com",
  "to_device": "my-device-id",
  "event": "sdp_answer",
  "payload": {
    "sdp": "v=0\no=- 0987654321 2 IN IP4 127.0.0.1\n...",
    "type": "answer"
  }
}
```

**Routing**: Same as `try_connect`

---

### 8. ICE Candidate Event
Exchanges ICE candidates for NAT traversal.

**Purpose**: Enable peer-to-peer connection through firewalls/NAT

**When to send**: Continuously during WebRTC negotiation

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "my-device-id",
  "to_email": "friend@example.com",
  "to_device": "friend-device-id",
  "event": "ice_candidate",
  "payload": {
    "candidate": "candidate:1234567890 1 udp 2122260223 192.168.1.100 54321 typ host",
    "sdpMid": "0",
    "sdpMLineIndex": 0
  }
}
```

**Routing**: Same as `try_connect`

---

### 9. Disconnect Event
Gracefully closes the connection.

**Purpose**: Clean disconnect

**When to send**: User logs out or closes app

**Input**:
```json
{
  "from_email": "user@example.com",
  "from_token": "",
  "from_device": "my-device-id",
  "to_email": "",
  "to_device": "",
  "event": "disconnect",
  "payload": {}
}
```

**Notes**:
- Server cleans up all mappings
- Broadcasts `user_left` to other pods via Redis
- Connection is closed immediately

---

## Server-Initiated Events

### User Joined
Broadcast when a user/device comes online.

**Received when**: Another user in your network connects

```json
{
  "from_email": "friend@example.com",
  "from_token": "",
  "from_device": "friend-device-id",
  "to_email": "",
  "to_device": "",
  "event": "user_joined",
  "payload": {
    "email": "friend@example.com",
    "device_id": "friend-device-id"
  }
}
```

### User Left
Broadcast when a user/device goes offline.

**Received when**: Another user in your network disconnects

```json
{
  "from_email": "friend@example.com",
  "from_token": "",
  "from_device": "friend-device-id",
  "to_email": "",
  "to_device": "",
  "event": "user_left",
  "payload": {
    "email": "friend@example.com",
    "device_id": "friend-device-id"
  }
}
```

---

## Error Responses

### Validation Error
```json
{
  "event": "error",
  "error": "from_email is required for register"
}
```

### Parse Error
```json
{
  "event": "error",
  "error": "Failed to parse message: ..."
}
```

### Unknown Event
```json
{
  "event": "error",
  "error": "Unknown event: invalid_event"
}
```

### Routing Error
```json
{
  "event": "error",
  "error": "Failed to route message - Redis unavailable",
  "target_email": "friend@example.com",
  "target_device": "friend-device-id"
}
```

---

## Data Structures

### SocketMessage
```rust
{
  from_email: String,    // Sender's email
  from_token: String,    // Auth token (only for register)
  from_device: String,   // Sender's device ID
  to_email: String,      // Target's email (for routing events)
  to_device: String,     // Target's device ID (for routing events)
  event: String,         // Event type
  payload: Value,        // Event-specific data
}
```

### DeviceInfo
```rust
{
  socket_id: String,     // Server-generated socket ID
  device_name: Option<String>,
  device_type: Option<String>,
}
```

---

## Multi-Pod Architecture

### Overview
Multiple pods can run behind a load balancer. Redis pub/sub ensures messages reach the correct pod.

### Flow
1. User connects to Pod A
2. Pod A stores presence in Redis
3. User B connects to Pod B
4. User A sends message to User B
5. Pod A checks locally - not found
6. Pod A publishes to Redis `socket:messages` channel
7. Pod B receives via Redis subscriber
8. Pod B forwards to User B's socket

### Redis Schema
- **Presence Key**: `socket:presence:{email}:{device_id}` → DeviceInfo
- **User Devices Key**: `socket:user_devices:{email}` → Hash of device_id → socket_id
- **Pub/Sub Channel**: `socket:messages`

### Graceful Degradation
If Redis is unavailable:
- Registration continues locally
- Cross-pod messaging fails
- Local pod messaging still works
- System logs errors but continues operating

---

## Client Implementation Guidelines

### Reconnection Strategy
1. On disconnect, wait 1-5 seconds (exponential backoff)
2. Reconnect WebSocket
3. Re-register with same device_id
4. Server automatically cleans up old socket

### Heartbeat
```javascript
setInterval(() => {
  socket.send(JSON.stringify({
    from_email: userEmail,
    event: "ping",
    ...
  }));
}, 30000);
```

### Message Validation
Always check `event` field in responses to handle:
- `error` - something went wrong
- `target_not_found` - recipient offline
- `pong` - heartbeat response
- Custom events from other users

### Error Handling
- Log all errors for debugging
- On `target_not_found`, show "User offline" UI
- On parse errors, check message format
- On connection errors, implement retry logic

---

## Testing Examples

### Basic Flow
```javascript
// 1. Connect
const ws = new WebSocket('ws://localhost:3000/socket/');

// 2. Register
ws.send(JSON.stringify({
  from_email: "test@example.com",
  from_token: "uuid-token",
  from_device: "device-123",
  to_email: "",
  to_device: "",
  event: "register",
  payload: { device_name: "Test Device" }
}));

// 3. Check online users
ws.send(JSON.stringify({
  from_email: "test@example.com",
  event: "check",
  ...
}));

// 4. Start heartbeat
setInterval(() => {
  ws.send(JSON.stringify({
    from_email: "test@example.com",
    event: "ping",
    ...
  }));
}, 30000);

// 5. Try to connect to another device
ws.send(JSON.stringify({
  from_email: "test@example.com",
  from_device: "device-123",
  to_email: "friend@example.com",
  to_device: "friend-device",
  event: "try_connect",
  payload: {}
}));
```

---

## Security Considerations

1. **Token Validation**: All `register` events validate JWT tokens
2. **Email Verification**: Tokens must match the provided email
3. **Rate Limiting**: Currently not implemented (MVP)
4. **Input Validation**: All messages validated before processing
5. **No Message Persistence**: Messages are not stored, only routed

## Troubleshooting

### Connection Issues
- Check WebSocket endpoint URL
- Verify token is valid and not expired
- Ensure email matches token

### Message Not Received
- Check if target is online (send `check` event)
- Verify `to_email` and `to_device` are correct
- Wait for 5-second timeout on cross-pod messages

### Frequent Disconnects
- Ensure ping is sent every 30 seconds
- Check network stability
- Verify no proxy/firewall blocking WebSocket

### Redis Errors
- System falls back to local-only mode
- Cross-pod messaging will fail
- Check Redis connection in logs

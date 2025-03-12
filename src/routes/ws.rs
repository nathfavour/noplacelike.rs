use actix::{Actor, ActorContext, AsyncContext, Handler, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse, Scope};
use actix_web_actors::ws;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// Static counter for connected clients
static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

// Connection ID type
type ClientId = usize;

// Shared state for all WebSocket connections
#[derive(Clone, Debug)]
pub struct ClipboardState {
    content: Arc<Mutex<String>>,
    clients: Arc<Mutex<HashSet<ClientId>>>,
}

impl ClipboardState {
    pub fn new(initial_content: String) -> Self {
        Self {
            content: Arc::new(Mutex::new(initial_content)),
            clients: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn update_content(&self, content: &str) -> Vec<ClientId> {
        let mut content_guard = self.content.lock().unwrap();
        *content_guard = content.to_string();

        // Return all connected clients (to broadcast to them)
        let clients = self.clients.lock().unwrap();
        clients.iter().copied().collect()
    }

    pub fn register_client(&self, client_id: ClientId) -> String {
        // Add client to connected clients
        {
            let mut clients = self.clients.lock().unwrap();
            clients.insert(client_id);
        }

        // Return current clipboard content
        let content = self.content.lock().unwrap();
        content.clone()
    }

    pub fn unregister_client(&self, client_id: ClientId) {
        let mut clients = self.clients.lock().unwrap();
        clients.remove(&client_id);
    }

    pub fn get_current_content(&self) -> String {
        let content = self.content.lock().unwrap();
        content.clone()
    }
}

// Websocket connection is an actor
struct WsClipboardSession {
    id: ClientId,
    last_heartbeat: Instant,
    clipboard_state: ClipboardState,
}

// Message types for WebSocket communication
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum WsMessage {
    Connect,
    Clipboard(String),
    Sync,
    Heartbeat,
}

impl Actor for WsClipboardSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Schedule regular heartbeat checks
        self.heartbeat(ctx);

        // Register this client
        let current_content = self.clipboard_state.register_client(self.id);

        // Send current clipboard content
        ctx.text(
            serde_json::to_string(&WsMessage::Clipboard(current_content))
                .unwrap_or_else(|_| String::from("{\"type\":\"error\"}")),
        );
    }

    fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
        // Unregister on disconnect
        self.clipboard_state.unregister_client(self.id);
        actix::Running::Stop
    }
}

impl WsClipboardSession {
    fn new(clipboard_state: ClipboardState) -> Self {
        Self {
            id: NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed),
            last_heartbeat: Instant::now(),
            clipboard_state,
        }
    }

    // Heartbeat to keep connection alive and detect disconnects
    fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        // Schedule a heartbeat check every 15 seconds
        ctx.run_interval(Duration::from_secs(15), |act, ctx| {
            // Check if we've received a heartbeat recently
            if Instant::now().duration_since(act.last_heartbeat) > Duration::from_secs(30) {
                // No recent heartbeat, disconnect
                act.clipboard_state.unregister_client(act.id);
                ctx.stop();
                return;
            }

            // Send heartbeat ping
            ctx.ping(b"");
        });
    }
}

// Handler for WebSocket messages
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsClipboardSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.last_heartbeat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.last_heartbeat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                // Try to parse the message
                match serde_json::from_str::<WsMessage>(&text) {
                    Ok(WsMessage::Connect) => {
                        // Client just connected, update last heartbeat
                        self.last_heartbeat = Instant::now();
                    }
                    Ok(WsMessage::Clipboard(content)) => {
                        // Client sent new clipboard content
                        // Update state and broadcast to all other clients
                        let clients = self.clipboard_state.update_content(&content);
                        let msg = serde_json::to_string(&WsMessage::Clipboard(content.clone()))
                            .unwrap_or_default();

                        // Broadcast to all clients including this one
                        ctx.text(msg);
                    }
                    Ok(WsMessage::Sync) => {
                        // Client requests current clipboard content
                        let current = self.clipboard_state.get_current_content();
                        ctx.text(
                            serde_json::to_string(&WsMessage::Clipboard(current))
                                .unwrap_or_default(),
                        );
                    }
                    Ok(WsMessage::Heartbeat) => {
                        // Client heartbeat, update timestamp
                        self.last_heartbeat = Instant::now();
                    }
                    Err(_) => {
                        // Invalid message format
                        ctx.text(r#"{"type":"error","data":"Invalid message format"}"#);
                    }
                }
            }
            Ok(ws::Message::Binary(_)) => {
                // Binary messages not supported
                ctx.text(r#"{"type":"error","data":"Binary messages not supported"}"#);
            }
            Ok(ws::Message::Close(reason)) => {
                // Client disconnected
                self.clipboard_state.unregister_client(self.id);
                ctx.close(reason);
                ctx.stop();
            }
            _ => (),
        }
    }
}

// WebSocket route handler
pub async fn clipboard_ws(
    req: HttpRequest,
    stream: web::Payload,
    clipboard_state: web::Data<ClipboardState>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsClipboardSession::new(clipboard_state.get_ref().clone()),
        &req,
        stream,
    )
}

// Create WebSocket scope
pub fn ws_scope() -> Scope {
    web::scope("/ws").route("/clipboard", web::get().to(clipboard_ws))
}

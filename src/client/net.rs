use std::{
    io::{BufRead, BufReader, Write},
    net::TcpStream,
    thread,
    time::Duration,
};

use bevy::prelude::*;

use super::state::{
    ClientAppState,
    ConnectionEvent,
    ConnectionMailbox,
    ConnectionPhase,
    ConnectionStatus,
};
use crate::protocol::{ClientHello, ServerMessage, ShipSnapshot};

pub(crate) fn begin_connection_attempt(
    server_addr: &str,
    status: &mut ConnectionStatus,
    mailbox: &ConnectionMailbox,
) {
    if matches!(status.phase, ConnectionPhase::Connecting) {
        return;
    }

    status.phase = ConnectionPhase::Connecting;
    status.active_snapshot = None;
    clear_mailbox(mailbox);

    let server_addr = server_addr.to_string();
    let mailbox = mailbox.clone();
    thread::spawn(move || {
        let event = match connect_to_host(&server_addr) {
            Ok(snapshot) => ConnectionEvent::Connected(snapshot),
            Err(message) => ConnectionEvent::Failed(message),
        };

        if let Ok(mut events) = mailbox.events.lock() {
            events.push(event);
        }
    });
}

pub(crate) fn poll_connection_events(
    mailbox: Res<ConnectionMailbox>,
    mut status: ResMut<ConnectionStatus>,
    mut next_state: ResMut<NextState<ClientAppState>>,
) {
    let Ok(mut events) = mailbox.events.lock() else {
        return;
    };

    for event in events.drain(..) {
        match event {
            ConnectionEvent::Connected(snapshot) => {
                status.phase = ConnectionPhase::Connected;
                status.active_snapshot = Some(snapshot);
                next_state.set(ClientAppState::Editing);
            }
            ConnectionEvent::Failed(message) => {
                status.phase = ConnectionPhase::Failed(message);
            }
        }
    }
}

fn clear_mailbox(mailbox: &ConnectionMailbox) {
    if let Ok(mut events) = mailbox.events.lock() {
        events.clear();
    }
}

fn connect_to_host(server_addr: &str) -> Result<ShipSnapshot, String> {
    let mut stream = TcpStream::connect(server_addr)
        .map_err(|error| format!("failed to connect to {server_addr}: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("failed to set read timeout: {error}"))?;
    stream
        .set_write_timeout(Some(Duration::from_secs(5)))
        .map_err(|error| format!("failed to set write timeout: {error}"))?;

    let hello = ClientHello::new("ship_editor");
    let encoded = serde_json::to_string(&hello)
        .map_err(|error| format!("failed to encode client hello: {error}"))?;
    stream
        .write_all(encoded.as_bytes())
        .and_then(|_| stream.write_all(b"\n"))
        .map_err(|error| format!("failed to send client hello: {error}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| format!("failed to read host response: {error}"))?;

    if line.trim().is_empty() {
        return Err("host closed the connection without sending a snapshot".to_string());
    }

    let message: ServerMessage = serde_json::from_str(&line)
        .map_err(|error| format!("failed to decode host response: {error}"))?;

    match message {
        ServerMessage::ShipSnapshot(snapshot) => Ok(snapshot),
        ServerMessage::Error { message } => Err(message),
    }
}

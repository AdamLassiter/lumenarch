use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

use crate::{
    protocol::{ClientHello, ServerMessage, ShipSnapshot},
    ship::{ModuleKind, ShipModule, storage::load_default_ship},
};

const DEFAULT_HOST_ADDR: &str = "127.0.0.1:5000";

pub fn run_host() -> Result<(), String> {
    let listener = TcpListener::bind(DEFAULT_HOST_ADDR)
        .map_err(|error| format!("failed to bind host on {DEFAULT_HOST_ADDR}: {error}"))?;

    println!("host: listening on {DEFAULT_HOST_ADDR}");
    println!("host: waiting for an initial client connection");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let peer = stream
                    .peer_addr()
                    .map(|addr| addr.to_string())
                    .unwrap_or_else(|_| "<unknown>".to_string());
                println!("host: accepted connection from {peer}");

                if let Err(error) = handle_client(stream) {
                    eprintln!("host: failed to serve client {peer}: {error}");
                } else {
                    println!("host: served initial ship snapshot to {peer}");
                }
            }
            Err(error) => {
                eprintln!("host: failed to accept incoming connection: {error}");
            }
        }
    }

    Ok(())
}

fn handle_client(stream: TcpStream) -> Result<(), String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|error| format!("failed to read client hello: {error}"))?;

    if line.trim().is_empty() {
        return Err("client disconnected before sending a hello".to_string());
    }

    let hello: ClientHello = serde_json::from_str(&line)
        .map_err(|error| format!("failed to decode client hello: {error}"))?;
    println!("host: client role is `{}`", hello.client_role);

    let response = ServerMessage::ShipSnapshot(load_host_ship_snapshot());
    let encoded = serde_json::to_string(&response)
        .map_err(|error| format!("failed to encode ship snapshot: {error}"))?;

    reader
        .get_mut()
        .write_all(encoded.as_bytes())
        .and_then(|_| reader.get_mut().write_all(b"\n"))
        .map_err(|error| format!("failed to send ship snapshot: {error}"))?;

    Ok(())
}

fn load_host_ship_snapshot() -> ShipSnapshot {
    match load_default_ship() {
        Ok(Some(ship)) => ship,
        Ok(None) => sample_ship_snapshot(),
        Err(error) => {
            eprintln!("host: failed to load saved ship, using sample snapshot: {error}");
            sample_ship_snapshot()
        }
    }
}

fn sample_ship_snapshot() -> ShipSnapshot {
    let mut snapshot = ShipSnapshot::empty("Untitled Knot");
    snapshot.modules = vec![
        ShipModule::new(1, ModuleKind::HullInnerCorner, -2, -2, 0),
        ShipModule::new(2, ModuleKind::Cockpit, -1, -2, 0),
        ShipModule::new(3, ModuleKind::Hull, 0, -2, 0),
        ShipModule::new(4, ModuleKind::HullInnerCorner, 1, -2, 1),
        ShipModule::new(5, ModuleKind::Hull, -2, -1, 3),
        ShipModule::new(6, ModuleKind::Airlock, -1, -1, 3),
        ShipModule::new(7, ModuleKind::Turret, 0, -1, 0),
        ShipModule::new(8, ModuleKind::Hull, 1, -1, 1),
        ShipModule::new(9, ModuleKind::Hull, -2, 0, 3),
        ShipModule::new(10, ModuleKind::Core, -1, 0, 0),
        ShipModule::new(11, ModuleKind::Reactor, 0, 0, 0),
        ShipModule::new(12, ModuleKind::Hull, 1, 0, 1),
        ShipModule::new(13, ModuleKind::HullInnerCorner, -2, 1, 3),
        ShipModule::new(14, ModuleKind::Cargo, -1, 1, 0),
        ShipModule::new(15, ModuleKind::Processor, 0, 1, 0),
        ShipModule::new(16, ModuleKind::HullInnerCorner, 1, 1, 2),
        ShipModule::new(17, ModuleKind::Computer, 1, 1, 0),
        ShipModule::new(18, ModuleKind::Engine, -1, 2, 2),
        ShipModule::new(19, ModuleKind::Engine, 0, 2, 2),
    ];
    snapshot
}

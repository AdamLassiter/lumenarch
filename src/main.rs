mod client;
mod host;
mod protocol;
mod session;
mod ship;

fn main() {
    let wants_host = std::env::args().skip(1).any(|arg| arg == "--host");

    if wants_host {
        if let Err(error) = host::run_host() {
            eprintln!("{error}");
            std::process::exit(1);
        }
    } else {
        client::run_client();
    }
}

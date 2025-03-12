use std::io;

mod config;
mod routes;
mod server;
mod services;
mod templates;

#[actix_web::main]
async fn main() -> io::Result<()> {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();
    let mut host = "0.0.0.0".to_string();
    let mut port = 8000;

    // Very simple argument parsing (could use clap for more robust parsing)
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                if i + 1 < args.len() {
                    host = args[i + 1].clone();
                    i += 1;
                }
            }
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse().unwrap_or(8000);
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Initialize config
    let config = config::load_config();
    
    // Start server
    println!("Starting noplacelike server...");
    server::run_server(host, port, config).await
}

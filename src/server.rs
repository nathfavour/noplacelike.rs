use actix_web::{web, App, HttpServer};
use local_ip_address::local_ip;
use qr2term;
use std::io;
use std::net::IpAddr;

use crate::config::Config;
use crate::routes;

pub async fn run_server(host: String, port: u16, config: Config) -> io::Result<()> {
    // Prepare shared data
    let shared_config = web::Data::new(config);
    
    // Create shared clipboard state
    let clipboard_data = web::Data::new(std::sync::Mutex::new(String::new()));
    
    // Print server URLs and QR codes
    print_server_info(port);
    
    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(shared_config.clone())
            .app_data(clipboard_data.clone())
            // Register API routes
            .service(routes::api::api_scope())
            // Register UI routes
            .service(routes::ui::ui_scope())
            // Register streaming routes
            .service(routes::streaming::stream_scope())
            // Register admin routes
            .service(routes::admin::admin_scope())
            // Add default route to redirect to UI
            .default_service(web::get().to(routes::ui::redirect_to_ui))
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}

fn print_server_info(port: u16) {
    // Get all available IP addresses
    let ips = get_all_ips();
    
    println!("\nServer accessible at:");
    
    for ip in ips {
        let url = format!("http://{}:{}", ip, port);
        
        if ip.is_loopback() {
            println!("\n=== Localhost Access ===");
        } else if ip.to_string().starts_with("192.168.") || ip.to_string().starts_with("10.") {
            println!("\n=== Local Network Access (preferred) ===");
        } else {
            println!("\n=== Other Network Access ===");
        }
        
        println!("URL: {}", url);
        
        // Generate QR code
        match qr2term::print_qr(&url) {
            Ok(_) => {},
            Err(e) => eprintln!("Failed to generate QR code: {}", e),
        }
        
        println!("{}", "-".repeat(50));
    }
}

fn get_all_ips() -> Vec<IpAddr> {
    let mut ips = Vec::new();
    
    // Add localhost
    ips.push("127.0.0.1".parse::<IpAddr>().unwrap());
    
    // Try to get local network IP
    match local_ip() {
        Ok(ip) => {
            if !ip.is_loopback() {
                ips.push(ip);
            }
        },
        Err(e) => eprintln!("Failed to get local IP: {}", e),
    }
    
    // Sort IPs with local network addresses first
    ips.sort_by_key(|ip| {
        let ip_str = ip.to_string();
        if ip_str.starts_with("192.168.") {
            0
        } else if ip_str.starts_with("10.") {
            1
        } else if ip_str.starts_with("172.") {
            2
        } else {
            3
        }
    });
    
    ips
}

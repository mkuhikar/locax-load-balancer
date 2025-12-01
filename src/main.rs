use std::sync::{Arc, Mutex};
use tokio::io;
use tokio::net::{TcpListener, TcpStream};

struct LoadBalancer {
    // List of backend servers (e.g., "127.0.0.1:8081")
    backends: Vec<String>,
    // Current index to support Round Robin selection
    current_index: Mutex<usize>,
}

impl LoadBalancer {
    fn new(backends: Vec<String>) -> Self {
        LoadBalancer {
            backends,
            current_index: Mutex::new(0),
        }
    }

    // Round Robin selection logic
    fn next_backend(&self) -> String {
        let mut idx = self.current_index.lock().unwrap();
        let backend = self.backends[*idx].clone();
        
        // Increment and wrap around
        *idx = (*idx + 1) % self.backends.len();
        backend
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    // 1. Define the ports we want to load balance between
    // We will assume you have servers running on 8081 and 8082
    let backend_addrs = vec![
        "127.0.0.1:9001".to_string(),
        "127.0.0.1:9002".to_string(),
        "127.0.0.1:9003".to_string(),
    ];

    // 2. Initialize the Load Balancer shared state
    let lb = Arc::new(LoadBalancer::new(backend_addrs));

    // 3. Listen on localhost:8080
    let listener_addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(listener_addr).await?;
    println!("Load Balancer listening on {}...", listener_addr);

    loop {
        // 4. Accept incoming client connection
        let (mut client_socket, client_addr) = listener.accept().await?;
        println!("Accepted connection from: {}", client_addr);

        let lb_clone = lb.clone();

        // 5. Spawn a lightweight async task for this connection
        tokio::spawn(async move {
            // Select the next backend server
            let backend_addr = lb_clone.next_backend();
            println!("Forwarding {} to backend: {}", client_addr, backend_addr);

            // Attempt to connect to the backend
            match TcpStream::connect(&backend_addr).await {
                Ok(mut server_socket) => {
                    // 6. Proxy data bidirectionally (Client <-> LB <-> Backend)
                    // copy_bidirectional handles reading/writing efficiently
                    let _ = io::copy_bidirectional(&mut client_socket, &mut server_socket).await;
                }
                Err(e) => {
                    eprintln!("Failed to connect to backend {}: {}", backend_addr, e);
                }
            }
        });
    }
}
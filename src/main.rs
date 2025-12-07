use std::io::BufReader;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::Deserialize;
use tokio::{io, time};
use tokio::net::{TcpListener, TcpStream};
use std::fs::File;
struct Backend{
    addr : String,
    is_healthy: bool

}

struct LoadBalancer {
    backends: Mutex<Vec<Backend>>,
    // Current index to support Round Robin selection
    current_index: Mutex<usize>,
}
#[derive(Debug,Deserialize)]
struct UpstreamConfig {
    ip: String,
    port: u16

}

impl LoadBalancer {
    fn new(addrs: Vec<String>) -> Self {
        let backends = addrs
        .into_iter()
        .map(|addr| Backend {
            addr,
            is_healthy : true,
        })
        .collect();

        LoadBalancer {
            backends:Mutex::new(backends),
            current_index: Mutex::new(0),
        }
    }

    // Round Robin selection logic
    fn next_backend(&self) -> Option<String> {
        let backends = self.backends.lock().unwrap();
        let mut idx = self.current_index.lock().unwrap();
        let pool_size = backends.len();
        for _ in 0..pool_size{
            *idx = (*idx +1)%pool_size;
            if backends[*idx].is_healthy {
                return Some(backends[*idx].addr.clone());
            }
        }
        None
    
    }
    fn set_health(&self,index:usize,healthy:bool){
        let mut backends = self.backends.lock().unwrap();
        if backends[index].is_healthy !=healthy{
            if healthy{

                println!("Server {} is back UP",backends[index].addr)
            }else{
                println!("Server {} is DOWN",backends[index].addr)
            }
            backends[index].is_healthy = healthy;
        }
    }
}

async fn health_checker(lb:Arc<LoadBalancer>){
    loop {
        time::sleep(Duration::from_secs(3)).await;
        let num_backends = lb.backends.lock().unwrap().len();
        for i in 0..num_backends{
            let addr = lb.backends.lock().unwrap()[i].addr.clone();
            let is_alive = match time::timeout(Duration::from_secs(1), TcpStream::connect(&addr)).await{
                Ok(Ok(_))=>true,
                _=>false,
            };
            lb.set_health(i,is_alive);
        }
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let file = File::open("upstream.json").expect("Failed to open upstream.json");
    let reader = BufReader::new(file);
    let configs:Vec<UpstreamConfig> = serde_json::from_reader(reader).expect("Failed to parse JSON");


    let backend_addrs:Vec<String> = configs.into_iter().map(|c| format!("{}:{}",c.ip,c.port)).collect();
    if backend_addrs.is_empty() {
        eprintln!("Error: No upstream servers found in upstream.json");
        return Ok(());
    }

    println!("Loaded backends: {:?}", backend_addrs);

    let lb = Arc::new(LoadBalancer::new(backend_addrs));
    let lb_monitor = lb.clone();
    tokio::spawn(async move{
        health_checker(lb_monitor).await;
    });

    let listener_addr = "127.0.0.1:8080";
    let listener = TcpListener::bind(listener_addr).await?;
    println!("Load Balancer with health check listening on {}...", listener_addr);

    loop {
        let (mut client_socket, client_addr) = listener.accept().await?;
        println!("Accepted connection from: {}", client_addr);
        let lb_clone = lb.clone();

        

        tokio::spawn(async move {
            // Select the next backend server
            match lb_clone.next_backend(){
                Some(backend_addr)=>{
                    println!("Forwarding {} -> {}", client_addr, backend_addr);
                    match TcpStream::connect(&backend_addr).await {
                    Ok(mut server_socket) => {
                        // Proxy data bidirectionally (Client <-> LB <-> Backend)
                        let _ = io::copy_bidirectional(&mut client_socket, &mut server_socket).await;
                    }
                    Err(e) => {
                        eprintln!("Failed to connect to backend {}: {}", backend_addr, e);
                    }
            }
                }
                None=>{
                    eprint!("WARNING: All Backends are down! Dropping connection from {}",client_addr)
                }

            };

            
        });
    }
}
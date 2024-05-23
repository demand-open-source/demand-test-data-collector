use std::net::{SocketAddr, ToSocketAddrs};

use demand_easy_sv2::{PoolMessages, ProxyBuilder, Remote};
use dotenv;
use tokio::{net::{TcpListener, TcpStream}, sync::mpsc::Receiver};


#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let client = wait_for_client().await;
    let server = connect_to_server().await;
    let mut builder = ProxyBuilder::new();
    builder
        .try_add_client(client)
        .await
        .expect("Impossible to add client to the ProxyBuilder")
        .try_add_server(server)
        .await
        .expect("Impossible to add server to the ProxyBuilder");
    add_handlers_for_down(&mut builder);
    add_handlers_for_up(&mut builder);
    let err = builder
        .try_build()
        .expect("Impossible to build the Proxy")
        .start().await;
    eprintln!("{err:?}");
}

async fn wait_for_client() -> TcpStream {
    let client: SocketAddr = std::env::var("CLIENT")
        .expect("Client address must be set: set it via the CLIENT env var")
        .to_socket_addrs()
        .expect("Client address must be a valid address")
        .next().unwrap();
    let listner = TcpListener::bind(client).await.expect("Impossible to listen on given address");
    if let Ok((stream, _)) = listner.accept().await {
        stream
    } else {
        panic!("Impossible to accept dowsntream connetion")
    }
}

async fn connect_to_server() -> TcpStream {
    let server: SocketAddr = std::env::var("SERVER")
        .expect("Server address must be set: set it via the SERVER env var")
        .to_socket_addrs()
        .expect("Client address must be a valid address")
        .next().unwrap();
    TcpStream::connect(server).await.expect("Impossible to connect to server")
}

fn add_handlers_for_down(builder: &mut ProxyBuilder) {
    for i in 0..44 {
        add_printer(builder.add_handler(Remote::Client, i))
    }
}
fn add_handlers_for_up(builder: &mut ProxyBuilder) {
    for i in 0..44 {
        add_printer(builder.add_handler(Remote::Server, i))
    }
}

fn add_printer(mut messages: Receiver<PoolMessages<'static>>) {
    tokio::spawn(async move {
        while let Some(m) = messages.recv().await {
            if let Ok(as_json) = serde_json::to_string(&m) {
                println!("{as_json}");
            } else {
                eprintln!("Impossible to serialize message: {m:?}");
                std::process::exit(1);
            }
        }
    });
}

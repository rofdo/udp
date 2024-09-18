use clap::Parser;
use serde::{Deserialize, Serialize};
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    KeepAlive(String),
    Join(String),
    Leave(String),
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: SocketAddr,
    #[arg(short, long)]
    name: String,
}

fn spawn_keep_alive(target: SocketAddr, socket: Arc<Mutex<UdpSocket>>, playerid: String) {
    thread::spawn(
        move || {
        loop {
            let lock = socket.lock().unwrap();
            let message = Message::KeepAlive(playerid.to_string());
            lock.send_to(&bincode::serialize(&message).unwrap(), target)
                .expect("Couldn't send data");
            println!("Send Heartbeat");
            drop(lock);
            thread::sleep(Duration::from_secs(1));
        }
    },
    );
}

fn main() {
    let args = Args::parse();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind socket");
    socket.set_nonblocking(true).unwrap();
    let socket = Arc::new(Mutex::new(socket));
    spawn_keep_alive(args.ip.clone(), socket.clone(), args.name.clone());

    let mut buf = [0u8; 1024];
    loop {
        while let Ok((size, src_addr)) = socket.lock().unwrap().recv_from(&mut buf) {
            println!("size: {}", size);
            println!("src: {}", src_addr);

            let bytes = buf[..size].to_vec();
            let message_response: Message = bincode::deserialize(&bytes).unwrap();
            println!("got: {:?}", message_response);
        }
    }
}

use clap::{Parser, ValueEnum};
use log::{debug, info, trace, LevelFilter};
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
    #[arg(short, long, default_value = "info")]
    log_level: LogLevel,
}

#[derive(Debug, ValueEnum, Clone, Copy)]
enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Into<LevelFilter> for LogLevel {
    fn into(self) -> LevelFilter {
        match self {
            LogLevel::Trace => LevelFilter::Trace,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Error => LevelFilter::Error,
        }
    }
}

fn spawn_keep_alive(target: SocketAddr, socket: Arc<Mutex<UdpSocket>>, playerid: String) {
    thread::spawn(move || loop {
        let lock = socket.lock().unwrap();
        let message = Message::KeepAlive(playerid.to_string());
        lock.send_to(&bincode::serialize(&message).unwrap(), target)
            .expect("Couldn't send data");
        debug!("Send Heartbeat");
        drop(lock);
        thread::sleep(Duration::from_secs(1));
    });
}

fn join_game(target: SocketAddr, socket: Arc<Mutex<UdpSocket>>, playerid: String) {
    let lock = socket.lock().unwrap();
    let message = Message::Join(playerid.to_string());
    lock.send_to(&bincode::serialize(&message).unwrap(), target)
        .expect("Couldn't send data");
    debug!("Send Join");
}

fn main() {
    let args = Args::parse();
    env_logger::builder()
        .filter_level(args.log_level.into())
        .init();
    let socket = UdpSocket::bind("0.0.0.0:0").expect("couldn't bind socket");
    socket.set_nonblocking(true).unwrap();
    let socket = Arc::new(Mutex::new(socket));
    spawn_keep_alive(args.ip.clone(), socket.clone(), args.name.clone());

    join_game(args.ip.clone(), socket.clone(), args.name.clone());

    let mut buf = [0u8; 1024];
    loop {
        while let Ok((size, src_addr)) = socket.lock().unwrap().recv_from(&mut buf) {
            trace!("size: {}", size);
            trace!("src: {}", src_addr);

            let bytes = buf[..size].to_vec();
            let message_response: Message = bincode::deserialize(&bytes).unwrap();
            match message_response {
                Message::KeepAlive(playerid) => {
                    debug!("KeepAlive from {}", playerid);
                }
                Message::Join(playerid) => {
                    info!("Join from {}", playerid);
                }
                Message::Leave(playerid) => {
                    info!("Leave from {}", playerid);
                }
            }
        }
    }
}

use clap::Parser;
use dhust::Chord;
use std::{net::Ipv4Addr, sync::Arc};
use log::{trace, debug, info, warn, error, };

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Opts {
    #[clap(short, long, required = true)]
    address: Ipv4Addr,
    #[clap(short, long, required = true)]
    port: u16,
    #[clap(long, requires = "join_port")]
    join_address: Option<Ipv4Addr>,
    #[clap(long, requires = "join_address")]
    join_port: Option<u16>,
    #[clap(long, required = true)]
    ts: u64,
    #[clap(long, required = true)]
    tff: u64,
    #[clap(long, required = true)]
    tcp: u64,
    #[clap(short, long, required = false)]
    successors: u32,
}

fn start_reading_stdin_lines(
    sender: tokio::sync::mpsc::Sender<String>,
    runtime: tokio::runtime::Handle,
) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line_buf = String::new();
        while let Ok(_) = stdin.read_line(&mut line_buf) {
            let line = line_buf.trim_end().to_string();
            line_buf.clear();
            let sender2 = sender.clone();

            runtime.spawn(async move {
                let result = sender2.send(line).await;
                if let Err(error) = result {
                    println!("start_reading_stdin_lines send error: {:?}", error);
                }
            });
        }
    });
}

async fn read_input(
    mut line_receiver: tokio::sync::mpsc::Receiver<String>,
    mut _watch_receiver: tokio::sync::watch::Receiver<bool>,
) {
    let chord = Chord::new().await;

    loop {
        let c = Arc::clone(&chord);

        if let Some(line) = line_receiver.recv().await {
            // process the input
            match line.as_str() {
                "exit" => {
                    println!("exiting manually...");
                    break;
                }
                input => {
                    println!("{:?}", Chord::lookup(c, &input).await);
                }
            }
        } else {
            println!("debug");
        }    
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Warn).unwrap();

    let opts: Opts = Opts::parse();

    // Use the parsed arguments as needed
    debug!("IP address: {}", opts.address);
    debug!("Port: {}", opts.port);
    debug!("Time between stabilize: {} ms", opts.ts);
    debug!("Time between fix fingers: {} ms", opts.tff);
    debug!("Time between check predecessor: {} ms", opts.tcp);
    debug!("Number of successors: {}", opts.successors);

    if let Some(ja) = opts.join_address {
        debug!("Join IP address: {}", ja);
    }

    if let Some(jp) = opts.join_port {
        debug!("Join port: {}", jp);
    }

    let (line_sender, line_receiver) = tokio::sync::mpsc::channel(1);

    start_reading_stdin_lines(line_sender, tokio::runtime::Handle::current());
    read_input(line_receiver, tokio::sync::watch::channel(false).1).await;

    trace!("Exiting...");
}

#![forbid(unsafe_code)]
#![forbid(unused_must_use)]
#![warn(unused_crate_dependencies)]

use std::{
    env::args,
    error::Error,
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

fn main() -> Result<(), &'static str> {
    let mut args = args().skip(1);

    let backup_dir = args.next().ok_or("Please provide a backup directory")?;
    let backup_dir = Path::new(&backup_dir);

    if !backup_dir.is_dir() {
        return Err("The provided backup directory does not exist");
    }

    let port = args
        .next()
        .ok_or("Please provide a port number to listen to")?
        .parse::<u16>()
        .map_err(|_| "Please provide a VALID port number to listen to")?;

    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).unwrap();

    println!("> Waiting for incoming connections...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(err) = handle_connection(backup_dir, stream) {
                    eprintln!("| Connection handling failed: {err}");
                }
            }

            Err(err) => {
                eprintln!("| Failed to get TcpStream: {err}");
                continue;
            }
        };

        println!("> Waiting for another connection...");
    }

    println!("> Server exited.");

    Ok(())
}

fn handle_connection(backup_dir: &Path, mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    println!("> Got a new connection");

    let mut buf_reader = BufReader::new(&mut stream);

    println!("| Waiting for slot name...");

    let slot_name = read_line(&mut buf_reader)?;

    println!("| Slot name is: \"{slot_name}\"");
    println!("| Waiting for slot size...");

    let slot_size = read_line(&mut buf_reader)?;

    let slot_size = slot_size
        .parse::<usize>()
        .map_err(|err| format!("Got invalid slot size '{slot_size}': {err}"))?;

    println!("| Fetching data...");

    let mut slot_content = vec![0u8; slot_size];
    buf_reader.read_exact(&mut slot_content)?;

    if slot_content.len() != slot_size {
        return Err(format!(
            "Expected {} bytes, got {}",
            slot_size,
            slot_content.len()
        ))?;
    }

    println!("| Writing {} bytes to slot...", slot_content.len());

    fs::write(backup_dir.join(&slot_name), slot_content)?;

    println!("| Sending confirmation to client...");

    stream.write_all("OK".as_bytes())?;

    println!("| Done!");

    Ok(())
}

fn read_line(buf_reader: &mut BufReader<&mut TcpStream>) -> Result<String, Box<dyn Error>> {
    let mut string = String::new();

    buf_reader.read_line(&mut string)?;

    if !string.ends_with('\n') {
        return Err("Retrieved line does not end with a carriage return symbol")?;
    }

    string.pop();

    Ok(string)
}

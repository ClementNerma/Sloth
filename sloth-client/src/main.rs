#![forbid(unsafe_code)]
#![forbid(unused_must_use)]
#![warn(unused_crate_dependencies)]

use std::{
    env::args,
    error::Error,
    fs,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = args().skip(1);

    let server_url = args.next().ok_or("Please provide a server URL")?;
    let server_secret = args.next().ok_or("Please provide the server's secret")?;
    let slot_name = args.next().ok_or("Please provide a slot name")?;
    let slot_file = args.next().ok_or("Please provide a file to backup")?;

    let slot_file = Path::new(&slot_file);

    if !slot_file.is_file() {
        return Err("Provided file does not exist")?;
    }

    println!("| Reading file...");

    let file_content = fs::read(slot_file).map_err(|err| {
        eprintln!("{err}");
        "Failed to read file"
    })?;

    println!("| Size: {} bytes", file_content.len());
    println!("| Connecting to server...");

    let mut stream = TcpStream::connect(server_url)?;

    println!("| Sending server's secret...");

    stream.write_all(format!("{server_secret}\n").as_bytes())?;

    println!("| Sending slot name...");

    stream.write_all(format!("{slot_name}\n").as_bytes())?;

    println!("| Sending slot size...");

    stream.write_all(format!("{}\n", file_content.len()).as_bytes())?;

    println!("| Sending file content...");

    stream.write_all(&file_content)?;

    println!("| Waiting for confirmation from server...");

    let mut confirm = vec![];
    stream.read_to_end(&mut confirm)?;

    if confirm != "OK".as_bytes() {
        return Err("Did not receive valid confirmation from server")?;
    }

    println!("| Success!");

    Ok(())
}

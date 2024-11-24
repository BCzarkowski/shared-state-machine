use std::io::{Read, Write};
use std::net::TcpListener;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:7878")?;
    println!("Server is listening on 127.0.0.1:7878.");

    for stream in listener.incoming() {
        let mut stream = stream?;
        println!("Connection opened.");

        let mut buffer = [0; 512];
        let bytes_read = stream.read(&mut buffer)?;
        let message = String::from_utf8_lossy(&buffer[..bytes_read]);
        println!("Received: {}", message);

        let response = format!("[echo]: {}", message);
        stream.write_all(response.as_bytes())?;
        println!("Sent: {}", response);
    }

    Ok(())
}

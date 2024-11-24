use std::io::{Read, Write};
use std::net::TcpStream;

pub struct Client {
    stream: TcpStream,
}

impl Client {
    pub fn new(address: &str) -> std::io::Result<Self> {
        let stream = TcpStream::connect(address)?;
        println!("Successfully connected to {}.", address);

        Ok(Client { stream })
    }

    pub fn send(&mut self, message: &str) -> std::io::Result<()> {
        self.stream.write_all(message.as_bytes())?;
        println!("Sent: {}", message);

        Ok(())
    }

    pub fn receive(&mut self) -> std::io::Result<String> {
        let mut buffer = [0; 512];
        let bytes_read = self.stream.read(&mut buffer)?;
        let response = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();

        println!("Received: {}", response);
        Ok(response)
    }   
}

fn main() -> std::io::Result<()> {
    let mut client = Client::new("127.0.0.1:7878")?;

    client.send("I'm a client.")?;
    let _response = client.receive()?;
    Ok(())
}

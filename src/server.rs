use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

async fn run_server() {
    let listener = TcpListener::bind("127.0.0.1:7878").await.unwrap();
    println!("Server is listening on 127.0.0.1:7878.");

    let (tx, _rx) = broadcast::channel(10);

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();
        let rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, writer) = socket.split();

            handle_connection(reader, writer, addr, tx, rx)
                .await
                .expect("Failed to handle connection.");
        });
    }
}

pub async fn handle_connection<Reader, Writer>(
    reader: Reader,
    mut writer: Writer,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<(String, std::net::SocketAddr)>,
    mut rx: broadcast::Receiver<(String, std::net::SocketAddr)>,
) -> std::io::Result<()>
where
    Reader: AsyncRead + Unpin,
    Writer: AsyncWrite + Unpin,
{
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        tokio::select! {
            bytes_read = reader.read_line(&mut line) => {
                if bytes_read.unwrap() == 0 {
                    break Ok(());
                }
                println!("Received: {}", line);
                tx.send((line.clone(), addr)).unwrap();
                line.clear();
            }
            message = rx.recv() => {
                let (message, _sender) = message.unwrap();

                println!("Sending: {}", message);
                writer.write_all(message.as_bytes()).await.unwrap();
            }
        }
    }
}

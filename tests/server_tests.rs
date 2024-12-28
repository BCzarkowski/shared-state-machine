use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio_test::io::Builder;

use shared_state_machine::server::handle_connection;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn echo_server() {
        let reader = tokio_test::io::Builder::new()
            .read(b"Hi there\r\n")
            .read(b"How are you doing?\r\n")
            .build();
        let writer = tokio_test::io::Builder::new()
            .write(b"Hi there\r\n")
            .write(b"How are you doing?\r\n")
            .build();

        let (tx, _rx) = broadcast::channel(10);
        let rx = tx.subscribe();
        let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 7878);

        let _result = handle_connection(reader, writer, addr, tx, rx).await;
    }
}

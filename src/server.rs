use std::net::SocketAddrV4;

use tokio::{io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader}, net::{TcpListener, TcpStream}, signal};

pub const ENDING_BYTE : u8 = 0x07;

pub struct BuboCoreServer {
    pub ip : String,
    pub port : u16,
}

async fn process_client(mut socket : TcpStream) -> io::Result<()> {
    let mut buff = Vec::new();
    let mut res = Vec::new();
    loop {
        let mut buf_reader = BufReader::new(socket);
        let n = buf_reader.read_until(ENDING_BYTE, &mut buff).await?;
        socket = buf_reader.into_inner();
        if n == 0 {
            return Ok(());
        }
        println!("Received : {}", String::from_utf8(buff.clone()).unwrap());
        buff.clear();
        socket.write_all(&res).await?;
    }
}

impl BuboCoreServer {

    pub async fn start(&mut self) -> io::Result<()> {
        println!("[↕] Starting server");
        let addr = SocketAddrV4::new(self.ip.parse().unwrap(), self.port);
        let listener = TcpListener::bind(addr).await?;

        loop {
            let (socket, c_addr) = tokio::select! {
                _ = signal::ctrl_c() => return Ok(()),
                res = listener.accept() => res.unwrap()
            };
            println!("[🎺] New client connected {}", c_addr);
            tokio::spawn(async move {
                let _ = process_client(socket).await;
                println!("[👋] Client disconnected {}", c_addr);
            });
        }
    }

}

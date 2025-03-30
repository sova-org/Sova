use super::{ENDING_BYTE, ServerMessage};
use crate::schedule::SchedulerMessage;
use serde::{Deserialize, Serialize};
use std::net::SocketAddrV4;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{TcpSocket, TcpStream},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    SchedulerControl(SchedulerMessage),
    SetTempo(f64),
    SetName(String),
    GetPattern,
    GetClock,
}

pub struct BuboCoreClient {
    pub ip: String,
    pub port: u16,
    pub stream: Option<TcpStream>,
    pub connected: bool,
}

impl BuboCoreClient {
    pub fn new(ip: String, port: u16) -> Self {
        BuboCoreClient {
            ip,
            port,
            stream: None,
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> io::Result<()> {
        let addr = SocketAddrV4::new(self.ip.parse().unwrap(), self.port);
        let socket = TcpSocket::new_v4()?;
        self.stream = Some(socket.connect(addr.into()).await?);
        self.connected = true;
        Ok(())
    }

    pub async fn send(&mut self, message: ClientMessage) -> io::Result<()> {
        let mut msg = serde_json::to_vec(&message).unwrap();
        msg.push(ENDING_BYTE);
        let socket = self.mut_socket()?;
        let res = socket.write_all(&msg).await;
        if res.is_err() {
            self.connected = false;
        }
        return res;
    }

    pub fn mut_socket(&mut self) -> io::Result<&mut TcpStream> {
        match &mut self.stream {
            Some(x) => Ok(x),
            None => Err(io::ErrorKind::NotConnected.into()),
        }
    }

    pub fn socket(&self) -> io::Result<&TcpStream> {
        match &self.stream {
            Some(x) => Ok(x),
            None => Err(io::ErrorKind::NotConnected.into()),
        }
    }

    /// Waits until some data is available, or the socket has been disconnected.
    /// Returns true if some data is available, false if the socket is disconnected.
    pub async fn ready(&mut self) -> bool {
        let mut buf = [0];
        let Ok(socket) = self.socket() else {
            return false;
        };
        let n = socket.peek(&mut buf).await;
        if n.is_err() || n.unwrap() == 0 {
            self.connected = false;
        }
        self.connected
    }

    pub async fn read(&mut self) -> io::Result<ServerMessage> {
        if !self.connected {
            return Err(io::ErrorKind::NotConnected.into());
        }
        let mut buff = Vec::new();
        let socket = self.mut_socket()?;
        let mut buf_reader = BufReader::new(socket);
        let n = buf_reader.read_until(ENDING_BYTE, &mut buff).await?;
        if n == 0 {
            self.connected = false;
            return Err(io::ErrorKind::NotConnected.into());
        }
        buff.pop();
        if let Ok(msg) = serde_json::from_slice::<ServerMessage>(&buff) {
            Ok(msg)
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

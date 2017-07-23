use std::io::{BufReader, BufWriter, Result, Error, ErrorKind};
use std::io::prelude::*;
use std::net::TcpStream;
use std::sync::Mutex;

pub trait Connection {
    fn send(&self, msg: &str) -> Result<()>;
    fn recv(&self) -> Result<String>;
    fn reconnect(&self) -> Result<()>;
    fn add_reconnect_attempt(&self) -> Result<()>;
    fn get_reconnect_attempts(&self) -> u8;
}

pub struct NetConnection {
    host: String,
    port: u16,
    reconnect_attempts: Mutex<u8>,
    reader: Mutex<BufReader<TcpStream>>,
    writer: Mutex<BufWriter<TcpStream>>,
}

impl NetConnection {
    fn new(
        host: &str,
        port: u16,
        reader: BufReader<TcpStream>,
        writer: BufWriter<TcpStream>,
    ) -> Self {
        NetConnection {
            host: host.to_owned(),
            port: port,
            reconnect_attempts: Mutex::new(0),
            reader: Mutex::new(reader),
            writer: Mutex::new(writer),
        }
    }

    pub fn connect(host: &str, port: u16) -> Result<Self> {
        let socket = TcpStream::connect(&format!("{}:{}", host, port))?;
        Ok(NetConnection::new(
            host,
            port,
            BufReader::new(socket.try_clone()?),
            BufWriter::new(socket),
        ))
    }
}

impl Connection for NetConnection {
    fn send(&self, msg: &str) -> Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.write_all(msg.as_bytes())?;
        writer.flush()
    }

    fn recv(&self) -> Result<String> {
        let mut ret = String::new();
        self.reader.lock().unwrap().read_line(&mut ret)?;
        if ret.is_empty() {
            Err(Error::new(ErrorKind::Other, "EOF"))
        } else {
            Ok(ret)
        }
    }

    fn reconnect(&self) -> Result<()> {
        let socket = TcpStream::connect(&format!("{}:{}", self.host, self.port))?;
        *self.reader.lock().unwrap() = BufReader::new(socket.try_clone()?);
        *self.writer.lock().unwrap() = BufWriter::new(socket);
        *self.reconnect_attempts.lock().unwrap() = 0;
        println!("Reconnect success!");
        Ok(())
    }

    fn add_reconnect_attempt(&self) -> Result<()> {
        (*self.reconnect_attempts.lock().unwrap()) += 1;
        Ok(())
    }

    fn get_reconnect_attempts(&self) -> u8 {
        *self.reconnect_attempts.lock().unwrap()
    }
}

use std::io::{Result, Error, ErrorKind};
use std::error::Error as StdError;

use connection::{Connection, NetConnection};

pub struct Client {
    pass: String,
    user_name: String,
    conn: Box<Connection>,
}

impl Client {
    pub fn new(user_name: &str, pass: &str, host: &str, port: u16) -> Self {
        let conn = NetConnection::connect(host, port).unwrap();
        Client {
            pass: pass.to_string(),
            user_name: user_name.to_string(),
            conn: Box::new(conn),
        }
    }

    pub fn iter<'a>(&'a self) -> Box<Iterator<Item = Result<String>> + 'a> {
        Box::new(ConIter::new(self))
    }

    pub fn conn(&self) -> &Box<Connection> {
        &self.conn
    }

    pub fn send(&self, msg: &str) -> Result<()> {
        let mut send = msg.to_string();
        send.push_str("\r\n");
        self.conn.send(&send)
    }

    pub fn reconnect(&self) -> Result<()> {
        match self.conn.reconnect() {
            Ok(_) => Ok(()),
            Err(err) => {
                self.conn.add_reconnect_attempt()?;
                return Err(err);
            }
        }
    }

    pub fn login(&self) -> Result<()> {
        self.send(&format!("PASS :{}", self.pass))?;
        self.send(&format!("NICK :{}", self.user_name))?;
        self.send(&format!("USER {0} 8 * :{0}", self.user_name))?;
        self.send(&format!("JOIN #c_midknight"))?;
        Ok(())
    }
}

pub struct ConIter<'a> {
    client: &'a Client,
}

impl<'a> ConIter<'a> {
    pub fn new(client: &'a Client) -> Self {
        ConIter { client: client }
    }

    fn get_next_line(&self) -> Result<String> {
        self.client.conn().recv()
    }
}

impl<'a> Iterator for ConIter<'a> {
    type Item = Result<String>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.get_next_line() {
                Ok(msg) => {
                    // message received
                    return Some(Ok(msg));
                }
                Err(ref err) if err.description() == "EOF" => {
                    // empty message
                    return None;
                }
                Err(_) => {
                    // disconnect
                    let attempts = self.client.conn().get_reconnect_attempts();
                    if attempts < 5 {
                        println!(
                            "Lost Connection to server, attempting to reconnect... Attempt: {} / 5",
                            attempts + 1
                        );
                        let _ = self.client.reconnect().and_then(|_| self.client.login());
                    } else {
                        return Some(Err(Error::new(ErrorKind::NotConnected, "Disconnected"))); // Disconnect and kill thread
                    }
                }
            }
        }
    }
}

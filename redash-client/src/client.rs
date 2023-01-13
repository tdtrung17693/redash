use std::{
    cell::RefCell,
    io::{self, Write},
    net::TcpStream,
};

use parser::{Data, Parser};

pub mod errors;
pub mod parser;

use errors::RedashError;

pub struct Client {
    url: String,
    stream: RefCell<Option<TcpStream>>,
}

impl Client {
    pub fn new(host: &str, port: u16) -> Self {
        return Client {
            url: format!("{host}:{}", port),
            stream: RefCell::new(None),
        };
    }

    pub fn connect(&mut self) -> Result<(), io::Error> {
        match TcpStream::connect(&self.url) {
            Ok(stream) => {
                self.stream = RefCell::new(Some(stream));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    pub fn send_command(&self, command: &str) -> Result<Data, RedashError> {
        if self.stream.borrow().is_none() {
            return Err(RedashError::OperationError(String::from("no_connection")));
        }

        let msg = format!("{command}\r\n");
        let msg = msg.as_bytes();
        let stream = self.stream.borrow_mut();
        let mut stream = stream.as_ref().unwrap();
        let reader = Parser::new(stream);

        stream.write(msg).unwrap();

        return reader.next();
    }
}

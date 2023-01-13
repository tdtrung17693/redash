use std::{
    cell::RefCell,
    fmt::Display,
    io::{BufReader, Read},
    str::from_utf8,
};

use super::errors::RedashError;

pub struct Parser<T: Read> {
    source: RefCell<BufReader<T>>,
}

#[derive(Debug)]
pub enum Data {
    String(String),
    Integer(i64),
    Array(Vec<Box<Data>>),
    Null,
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            &Data::Array(arr) => {
                for i in arr {
                    write!(f, "{i}\n")?
                }
            }
            &Data::Integer(i) => write!(f, "{i}")?,
            &Data::Null => write!(f, "null")?,
            &Data::String(s) => write!(f, "{s}")?,
        };
        Ok(())
    }
}

impl<T: Read> Parser<T> {
    pub fn new(source: T) -> Self {
        let buf_reader = BufReader::new(source);
        return Parser {
            source: RefCell::new(buf_reader),
        };
    }

    pub fn next(&self) -> Result<Data, RedashError> {
        let type_indicator = self.type_indicator()?;

        let data = match type_indicator {
            b'-' => {
                if let Err(err) = self.data_error() {
                    Err(err)
                } else {
                    Ok(Data::Null)
                }
            }
            b'+' => self.data_simple_string(),
            b':' => self.data_integer(),
            b'$' => self.data_bulk_string(),
            b'*' => self.data_array(),
            u => Err(RedashError::ServerError(
                String::from("invalid_server_data_type"),
                u,
            )),
        }?;

        return Ok(data);
    }

    fn char(&self) -> Result<u8, RedashError> {
        let mut buf = [0 as u8; 1];
        match self.source.borrow_mut().read_exact(&mut buf) {
            Ok(_) => Ok(buf[0]),
            Err(err) => Err(RedashError::IOError(err)),
        }
    }

    fn line(&self) -> Result<Vec<u8>, RedashError> {
        let mut bytes: Vec<u8> = Vec::new();
        let mut about_stop = false;

        loop {
            let ch = self.char()?;
            if about_stop {
                break;
            }
            if ch == b'\r' {
                about_stop = true;
                continue;
            }
            bytes.push(ch);
        }

        return Ok(bytes);
    }

    fn type_indicator(&self) -> Result<u8, RedashError> {
        let ch = self.char()?;
        return Ok(ch);
    }

    fn data_error(&self) -> Result<(), RedashError> {
        let line = self.line()?;
        match from_utf8(&line[..]) {
            Ok(err_str) => Err(RedashError::DataError(String::from(err_str))),
            Err(err) => Err(RedashError::UnknownError(Box::new(err))),
        }
    }

    fn data_integer(&self) -> Result<Data, RedashError> {
        let line = self.line()?;
        let n_str = match from_utf8(&line[..]) {
            Ok(n) => n,
            Err(err) => return Err(RedashError::UnknownError(Box::new(err))),
        };

        match str::parse(n_str.trim()) {
            Ok(number) => Ok(Data::Integer(number)),
            Err(err) => Err(RedashError::UnknownError(Box::new(err))),
        }
    }

    fn data_simple_string(&self) -> Result<Data, RedashError> {
        let line = self.line()?;
        match from_utf8(&line[..]) {
            Ok(s_str) => Ok(Data::String(String::from(s_str))),
            Err(err) => return Err(RedashError::UnknownError(Box::new(err))),
        }
    }

    fn data_bulk_string(&self) -> Result<Data, RedashError> {
        let total_chars = if let Data::Integer(n) = self.data_integer()? {
            n
        } else {
            -1
        };

        if total_chars == -1 {
            return Ok(Data::Null);
        }

        let mut bytes: Vec<u8> = Vec::with_capacity(total_chars as usize);
        for _ in 0..total_chars {
            bytes.push(self.char()?);
        }
        // swallow CRLF
        for _ in 0..2 {
            self.char()?;
        }

        match from_utf8(&bytes[..]) {
            Ok(s_str) => Ok(Data::String(String::from(s_str))),
            Err(err) => return Err(RedashError::UnknownError(Box::new(err))),
        }
    }

    fn data_array(&self) -> Result<Data, RedashError> {
        let total_items = if let Data::Integer(n) = self.data_integer()? {
            n
        } else {
            -1
        };

        if total_items == -1 {
            return Ok(Data::Null);
        }

        let mut items: Vec<Box<Data>> = Vec::with_capacity(total_items as usize);
        for _ in 0..total_items {
            let item = self.next()?;
            items.push(Box::new(item));
        }

        return Ok(Data::Array(items));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeSource {
        cursor: u16,
        subject: Vec<u8>,
    }

    impl FakeSource {
        fn new() -> Self {
            return FakeSource {
                cursor: 0,
                subject: Vec::from(""),
            };
        }
    }

    impl Read for FakeSource {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            for i in 0..self.subject.len() {
                buf[i] = self.subject[i];
            }
            self.cursor += 1;
            return Ok(buf.len());
        }
    }

    #[test]
    fn test_read_char() {
        let mut source = FakeSource::new();
        source.subject = Vec::from("abc");
        let parser = Parser::new(source);
        let ch = parser.char().unwrap();
        assert_eq!(ch, b'a');
    }
}

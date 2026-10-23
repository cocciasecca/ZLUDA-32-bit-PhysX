//! Versioned framing for the existing CUDA wire structs. Trusted, authenticated peers only.
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
pub const MAX_PAYLOAD: usize = 256 * 1024 * 1024;
const MAGIC: &[u8; 8] = b"ZLUDARP1";
pub const AUTH: u32 = u32::MAX - 1;
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
pub struct Frame {
    pub header: u32,
    pub sequence: u64,
    pub body: Vec<u8>,
}
impl Frame {
    pub fn read(input: &mut impl Read) -> io::Result<Self> {
        let mut h = [0; 24];
        input.read_exact(&mut h)?;
        if &h[..8] != MAGIC {
            return Err(invalid("Native protocol mismatch"));
        }
        let length = u32::from_le_bytes(h[12..16].try_into().unwrap()) as usize;
        if length > MAX_PAYLOAD {
            return Err(invalid("Oversized CUDA message"));
        }
        let mut body = Vec::new();
        body.try_reserve_exact(length).map_err(io::Error::other)?;
        body.resize(length, 0);
        input.read_exact(&mut body)?;
        Ok(Self {
            header: u32::from_le_bytes(h[8..12].try_into().unwrap()),
            sequence: u64::from_le_bytes(h[16..24].try_into().unwrap()),
            body,
        })
    }
    pub fn write(&self, output: &mut impl Write) -> io::Result<()> {
        if self.body.len() > MAX_PAYLOAD {
            return Err(invalid("Oversized CUDA message"));
        }
        let mut h = [0; 24];
        h[..8].copy_from_slice(MAGIC);
        h[8..12].copy_from_slice(&self.header.to_le_bytes());
        h[12..16].copy_from_slice(&(self.body.len() as u32).to_le_bytes());
        h[16..].copy_from_slice(&self.sequence.to_le_bytes());
        output.write_all(&h)?;
        output.write_all(&self.body)
    }
}
pub struct Connection {
    stream: TcpStream,
    sequence: u64,
    broken: bool,
}
impl Connection {
    pub fn new(stream: TcpStream) -> io::Result<Self> {
        stream.set_nodelay(true)?;
        stream.set_read_timeout(Some(Duration::from_secs(300)))?;
        stream.set_write_timeout(Some(Duration::from_secs(300)))?;
        Ok(Self {
            stream,
            sequence: 0,
            broken: false,
        })
    }
    pub fn authenticate_client(&mut self, token: &str, revision: &str) -> io::Result<Frame> {
        self.send(AUTH, format!("{token}\n{revision}").as_bytes())?;
        self.receive_same()
    }
    pub fn authenticate_server(&mut self, token: &str, revision: &str) -> io::Result<()> {
        let frame = self.receive_same()?;
        if frame.header != AUTH || frame.body != format!("{token}\n{revision}").as_bytes() {
            return Err(invalid("Authentication/revision mismatch"));
        }
        Ok(())
    }
    pub fn send(&mut self, header: u32, body: &[u8]) -> io::Result<()> {
        if self.broken {
            return Err(invalid("Disconnected native server"));
        }
        let result = Frame {
            header,
            sequence: self.sequence,
            body: body.to_vec(),
        }
        .write(&mut self.stream);
        if result.is_err() {
            self.broken = true;
        }
        result
    }
    fn receive_same(&mut self) -> io::Result<Frame> {
        if self.broken {
            return Err(invalid("Disconnected native server"));
        }
        let result = Frame::read(&mut self.stream).and_then(|frame| {
            if frame.sequence != self.sequence {
                Err(invalid("CUDA sequence mismatch"))
            } else {
                Ok(frame)
            }
        });
        if result.is_err() {
            self.broken = true;
        }
        result
    }
    pub fn receive_request(&mut self) -> io::Result<Frame> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| invalid("Sequence exhausted"))?;
        self.receive_same()
    }
    pub fn exchange(&mut self, header: u32, body: &[u8]) -> io::Result<Frame> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .ok_or_else(|| invalid("Sequence exhausted"))?;
        self.send(header, body)?;
        self.receive_same()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bounded_framing_and_truncation() {
        let f = Frame {
            header: 17,
            sequence: 4,
            body: vec![5, 6, 7],
        };
        let mut bytes = vec![];
        f.write(&mut bytes).unwrap();
        for n in 0..bytes.len() {
            assert!(Frame::read(&mut &bytes[..n]).is_err());
        }
        let r = Frame::read(&mut bytes.as_slice()).unwrap();
        assert_eq!(r.body, f.body);
        assert_eq!(r.header, 17);
        bytes[12..16].copy_from_slice(&((MAX_PAYLOAD + 1) as u32).to_le_bytes());
        assert!(Frame::read(&mut bytes.as_slice()).is_err());
    }
    #[test]
    fn authenticated_exchange() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = std::thread::spawn(move || {
            let mut c = Connection::new(listener.accept().unwrap().0).unwrap();
            c.authenticate_server("token", "sha").unwrap();
            c.send(9, b"sha").unwrap();
            let r = c.receive_request().unwrap();
            assert_eq!(r.header, 7);
            c.send(0, &r.body).unwrap();
        });
        let mut c = Connection::new(TcpStream::connect(address).unwrap()).unwrap();
        assert_eq!(c.authenticate_client("token", "sha").unwrap().body, b"sha");
        assert_eq!(c.exchange(7, b"payload").unwrap().body, b"payload");
        server.join().unwrap();
    }
}

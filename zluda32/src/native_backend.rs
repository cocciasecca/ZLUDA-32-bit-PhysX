//! Linux transport for the same complete CUDA dispatch used by the Windows server.
use cuda_types::cuda::CUerror;
use rkyv::rancor::{Failure, Strategy};
use rkyv::{Archive, Deserialize, Portable, Serialize};
use std::net::{SocketAddr, TcpStream};
use std::num::NonZeroU32;
use std::sync::OnceLock;
use std::time::Duration;
use zluda_server_common::{
    message_buffer::MessageBuffer, native_rpc::Connection, Opcode, Serializer,
};
pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("ZLUDA_NATIVE_BACKEND").as_deref() == Ok("1"))
}
pub struct Client {
    connection: Connection,
    request: MessageBuffer,
    response: MessageBuffer,
}
impl Client {
    pub fn connect() -> std::io::Result<Self> {
        let bad = || std::io::Error::other("Missing/invalid native server address or token");
        let address: SocketAddr = std::env::var("ZLUDA_NATIVE_ADDR")
            .map_err(|_| bad())?
            .parse()
            .map_err(|_| bad())?;
        if !address.ip().is_loopback() {
            return Err(bad());
        }
        let token = std::env::var("ZLUDA_NATIVE_TOKEN").map_err(|_| bad())?;
        if token.len() != 64 || !token.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err(bad());
        }
        let mut connection = Connection::new(TcpStream::connect_timeout(
            &address,
            Duration::from_secs(10),
        )?)?;
        let startup = connection.authenticate_client(&token, env!("VERGEN_GIT_SHA"))?;
        if startup.header != Opcode::Startup as u32
            || startup.body != env!("VERGEN_GIT_SHA").as_bytes()
        {
            return Err(std::io::Error::other("Native server revision mismatch"));
        }
        if std::env::var_os("ZLUDA_NATIVE_SHM_DIR").is_some() && !connection.uses_shared_memory() {
            return Err(std::io::Error::other("Shared-memory negotiation failed"));
        }
        Ok(Self {
            connection,
            request: MessageBuffer::new(),
            response: MessageBuffer::new(),
        })
    }
    fn invoke(&mut self, opcode: Opcode) -> Result<(), CUerror> {
        let frame = self
            .connection
            .exchange(opcode as u32, self.request.read_buffer())
            .map_err(|_| CUerror::UNKNOWN)?;
        if let Some(code) = NonZeroU32::new(frame.header) {
            return Err(CUerror(code));
        }
        self.response.replace_message(0, &frame.body)
    }
    pub fn fixed<Out: Portable + Clone>(
        &mut self,
        opcode: Opcode,
        data: impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Out, CUerror> {
        self.request.write_body(&data)?;
        self.invoke(opcode)?;
        if self.response.read_buffer().len() != std::mem::size_of::<Out>() {
            return Err(CUerror::UNKNOWN);
        }
        Ok(self.response.read_body())
    }
    pub fn framed_in<Out: Portable + Clone>(
        &mut self,
        opcode: Opcode,
        data: impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
        arena: &mut stumpalo::Arena,
    ) -> Result<Out, CUerror> {
        self.request.serialize_body(arena, &data)?;
        self.invoke(opcode)?;
        if self.response.read_buffer().len() != std::mem::size_of::<Out>() {
            return Err(CUerror::UNKNOWN);
        }
        Ok(self.response.read_body())
    }
    pub fn framed_out<Out: Archive>(
        &mut self,
        opcode: Opcode,
        data: impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Out, CUerror>
    where
        Out::Archived: Deserialize<Out, Strategy<(), Failure>>,
    {
        self.request.write_body(&data)?;
        self.invoke(opcode)?;
        self.response.deserialize_body()
    }
}

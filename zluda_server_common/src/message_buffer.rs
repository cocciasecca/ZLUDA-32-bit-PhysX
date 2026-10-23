//! Aligned, bounded in-process backing for the original wire message representation.
use super::*;
pub struct MessageBuffer {
    storage: Vec<u128>,
    pub name: String,
}
impl MessageBuffer {
    const OFFSET: usize = 16;
    pub fn new() -> Self {
        Self {
            storage: vec![0; 65536],
            name: String::new(),
        }
    }
    fn bytes(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.storage.as_ptr().cast(), self.storage.len() * 16) }
    }
    fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self.storage.as_mut_ptr().cast(), self.storage.len() * 16)
        }
    }
    fn reserve(&mut self, body: usize) -> Result<(), CUerror> {
        if body > native_rpc::MAX_PAYLOAD {
            return Err(CUerror::OUT_OF_MEMORY);
        }
        let count = (body + Self::OFFSET).div_ceil(16);
        if count > self.storage.len() {
            self.storage
                .try_reserve(count - self.storage.len())
                .map_err(|_| CUerror::OUT_OF_MEMORY)?;
            self.storage.resize(count, 0);
        }
        Ok(())
    }
    pub fn write_header(&mut self, value: u32) {
        self.bytes_mut()[..4].copy_from_slice(&value.to_le_bytes());
    }
    pub fn read_header(&self) -> u32 {
        u32::from_le_bytes(self.bytes()[..4].try_into().unwrap())
    }
    pub fn write_size(&mut self, value: u32) {
        self.bytes_mut()[4..8].copy_from_slice(&value.to_le_bytes());
    }
    pub fn read_size(&self) -> u32 {
        u32::from_le_bytes(self.bytes()[4..8].try_into().unwrap())
    }
    // Serialize fields rather than copying potentially uninitialized struct padding.
    pub fn write_body<T: for<'a, 'b> Serialize<Serializer<'a, 'b>>>(
        &mut self, body: &T,
    ) -> Result<(), CUerror> {
        self.serialize_body(&mut stumpalo::Arena::new(), body).map(|_| ())
    }
    pub fn read_body<T>(&self) -> T {
        assert!(mem::size_of::<T>() <= self.read_size() as usize);
        unsafe { ptr::read_unaligned(self.bytes().as_ptr().add(Self::OFFSET).cast()) }
    }
    pub fn write_buffer(&mut self, body: &[u8]) {
        assert!(body.len() <= self.storage.len() * 16 - Self::OFFSET);
        self.bytes_mut()[Self::OFFSET..Self::OFFSET + body.len()].copy_from_slice(body);
        self.write_size(body.len() as u32);
    }
    pub fn read_buffer(&self) -> &[u8] {
        &self.bytes()[Self::OFFSET..Self::OFFSET + self.read_size() as usize]
    }
    pub fn replace_message(&mut self, header: u32, body: &[u8]) -> Result<(), CUerror> {
        self.reserve(body.len())?;
        self.write_header(header);
        self.write_buffer(body);
        Ok(())
    }
    pub fn deserialize_body<Out: Archive>(&self) -> Result<Out, CUerror>
    where
        Out::Archived: Deserialize<Out, Strategy<(), Failure>>,
    {
        unsafe { rkyv::api::low::from_bytes_unchecked(self.read_buffer()) }
            .map_err(|_| CUerror::UNKNOWN)
    }
    pub fn deserialize_body2<Out: Archive>(&self) -> &Out::Archived
    where
        Out::Archived: Portable,
    {
        unsafe { rkyv::access_unchecked::<Out::Archived>(self.read_buffer()) }
    }
    pub fn serialize_body(
        &mut self,
        arena: &mut stumpalo::Arena,
        body: &impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Option<Self>, CUerror> {
        loop {
            let mut serializer = rkyv::ser::Serializer::new(
                SliceWriter {
                    offset: Self::OFFSET,
                    slice: self.bytes_mut(),
                },
                ScopedArena { arena },
                (),
            );
            let result =
                rkyv::api::serialize_using(body, Strategy::<_, AllocError>::wrap(&mut serializer));
            let length = serializer.writer.pos() - Self::OFFSET;
            drop(serializer);
            match result {
                Ok(_) => {
                    self.write_size(length as u32);
                    return Ok(None);
                }
                Err(AllocError::Stumpalo) => return Err(CUerror::OUT_OF_MEMORY),
                Err(AllocError::NotEnoughMemory { more_bytes }) => {
                    let capacity = self.storage.len() * 16;
                    let desired = capacity
                        .checked_add(more_bytes)
                        .and_then(usize::checked_next_power_of_two)
                        .ok_or(CUerror::OUT_OF_MEMORY)?;
                    self.reserve(desired - Self::OFFSET)?;
                }
            }
        }
    }
}
impl Default for MessageBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_messages_have_size_and_alignment() {
        let mut buffer = MessageBuffer::new();
        buffer.write_header(12);
        buffer.write_body(&[1u32, 2, 3, 4]).unwrap();
        assert_eq!(buffer.read_header(), 12);
        assert_eq!(buffer.read_size(), 16);
        assert_eq!((buffer.read_buffer().as_ptr() as usize) % 16, 0);
        assert_eq!(buffer.read_body::<[u32; 4]>(), [1, 2, 3, 4]);
    }
    #[test]
    fn framed_cuda_copy_survives_growth_and_reuse() {
        let mut buffer = MessageBuffer::new();
        let mut arena = stumpalo::Arena::new();
        for length in [1024 * 1024 + 137, 3, 0] {
            let input = crate::cuMemcpyHtoD_v2In {
                dst_device: 256u32.into(),
                src_host: vec![37; length],
            };
            assert!(buffer.serialize_body(&mut arena, &input).unwrap().is_none());
            let output = buffer
                .deserialize_body::<crate::cuMemcpyHtoD_v2In>()
                .unwrap();
            assert_eq!(output.dst_device.to_native(), 256);
            assert_eq!(output.src_host, input.src_host);
        }
    }
    #[test]
    fn reserve_is_bounded_and_replacement_preserves_header() {
        let mut buffer = MessageBuffer::new();
        assert!(buffer.reserve(native_rpc::MAX_PAYLOAD + 1).is_err());
        buffer.replace_message(42, b"abc").unwrap();
        assert_eq!(buffer.read_header(), 42);
        assert_eq!(buffer.read_buffer(), b"abc");
    }
}

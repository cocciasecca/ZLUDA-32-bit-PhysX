//! Windows events and named shared mappings for the serialized 32/64-bit bridge.
//! Both peers must be built from the same revision. A mapping-growth reply is
//! consumed before decoding its payload; no archived reference may survive reuse.
use super::*;
use rand::distr::{Alphanumeric, SampleString};
use windows::core::{Owned, PCSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::Memory::*;
use windows::Win32::System::Threading::*;
pub struct OwnedView(MEMORY_MAPPED_VIEW_ADDRESS);

impl OwnedView {
    fn new(shmem: HANDLE, size: usize) -> windows::core::Result<Self> {
        let view = unsafe { MapViewOfFile(shmem, FILE_MAP_ALL_ACCESS, 0, 0, size) };
        if view.Value.is_null() {
            return Err(windows::core::Error::empty());
        }
        Ok(OwnedView(view))
    }
}

impl Drop for OwnedView {
    fn drop(&mut self) {
        let _ = unsafe { UnmapViewOfFile(self.0) };
    }
}

pub struct Endpoint {
    pub event: Owned<HANDLE>,
    pub event_name: String,
    pub shared_memory: SharedMemory,
}

unsafe impl Send for Endpoint {}
unsafe impl Sync for Endpoint {}

impl Endpoint {
    pub unsafe fn new() -> windows::core::Result<Self> {
        let shared_memory = SharedMemory::new()?;
        let mut event_name = random_global_name();
        let event = Owned::new(CreateEventA(
            None,
            false,
            false,
            PCSTR(event_name.as_ptr()),
        )?);
        event_name.pop();
        Ok(Endpoint {
            event,
            event_name,
            shared_memory,
        })
    }

    pub unsafe fn open(
        mut event_name: String,
        shared_memory_name: String,
    ) -> windows::core::Result<Self> {
        event_name.push('\0');
        let event =
            Owned::new(unsafe { OpenEventA(EVENT_ALL_ACCESS, false, PCSTR(event_name.as_ptr())) }?);
        event_name.pop();
        Ok(Self {
            event,
            event_name,
            shared_memory: SharedMemory::open(
                shared_memory_name,
                Some(SharedMemory::INITIAL_SHARED_MEMORY_SIZE),
            )?,
        })
    }
}

pub struct SharedMemory {
    pub view: OwnedView,
    pub handle: Owned<HANDLE>,
    pub size: usize,
    pub name: String,
}

impl SharedMemory {
    const INITIAL_SHARED_MEMORY_SIZE: usize = 1024 * 1024;
    const OFFSET_HEADER: usize = 0;
    const OFFSET_SIZE: usize = 4;
    const OFFSET_BODY: usize = 16;

    pub unsafe fn new() -> windows::core::Result<Self> {
        let mut shared_memory_name = random_global_name();
        let shared_memory = Owned::new(CreateFileMappingA(
            INVALID_HANDLE_VALUE,
            None,
            PAGE_READWRITE,
            0,
            Self::INITIAL_SHARED_MEMORY_SIZE as u32,
            PCSTR(shared_memory_name.as_ptr()),
        )?);
        shared_memory_name.pop();
        let view = OwnedView::new(*shared_memory, Self::INITIAL_SHARED_MEMORY_SIZE)?;
        Ok(SharedMemory {
            name: shared_memory_name,
            handle: shared_memory,
            view,
            size: Self::INITIAL_SHARED_MEMORY_SIZE,
        })
    }

    pub unsafe fn open(
        mut shared_memory_name: String,
        size: Option<usize>,
    ) -> windows::core::Result<Self> {
        shared_memory_name.push('\0');
        let shared_memory = Owned::new(unsafe {
            OpenFileMappingA(
                FILE_MAP_ALL_ACCESS.0,
                false,
                PCSTR(shared_memory_name.as_ptr()),
            )
        }?);
        shared_memory_name.pop();
        let view = OwnedView::new(*shared_memory, size.unwrap_or(0))?;
        let size = match size {
            Some(s) => s,
            None => {
                let mut query = mem::zeroed();
                if 0 == VirtualQuery(Some(view.0.Value), &mut query, mem::size_of_val(&query)) {
                    return Err(windows::core::Error::empty());
                }
                query.RegionSize
            }
        };
        Ok(SharedMemory {
            name: shared_memory_name,
            handle: shared_memory,
            view,
            size,
        })
    }

    fn write<T>(&mut self, offset: usize, data: &T) {
        unsafe {
            ptr::copy_nonoverlapping(data, self.view.0.Value.wrapping_byte_add(offset).cast(), 1)
        };
    }

    fn read<T>(&self, offset: usize) -> T {
        let mut body = std::mem::MaybeUninit::<T>::uninit();
        unsafe {
            ptr::copy_nonoverlapping(
                self.view.0.Value.wrapping_byte_add(offset).cast(),
                body.as_mut_ptr(),
                1,
            )
        };
        unsafe { body.assume_init() }
    }

    pub fn write_header(&mut self, header: u32) {
        self.write(Self::OFFSET_HEADER, &header);
    }

    pub fn read_header(&self) -> u32 {
        self.read(Self::OFFSET_HEADER)
    }

    pub fn write_size(&mut self, size: u32) {
        self.write(Self::OFFSET_SIZE, &size);
    }

    pub fn read_size(&self) -> u32 {
        self.read(Self::OFFSET_SIZE)
    }

    pub fn write_body<T>(&mut self, body: &T) {
        self.write(Self::OFFSET_BODY, body);
        self.write_size(mem::size_of::<T>() as u32);
    }

    pub fn write_buffer(&mut self, body: &[u8]) {
        self.write_size(body.len() as u32);
        let output = unsafe {
            std::slice::from_raw_parts_mut(
                self.view
                    .0
                    .Value
                    .wrapping_byte_add(Self::OFFSET_BODY)
                    .cast(),
                body.len(),
            )
        };
        output.copy_from_slice(body);
    }

    pub fn read_buffer<'a>(&'a self) -> &'a [u8] {
        let size = self.read_size();
        unsafe {
            std::slice::from_raw_parts(
                self.view
                    .0
                    .Value
                    .wrapping_byte_add(Self::OFFSET_BODY)
                    .cast(),
                size as usize,
            )
        }
    }

    pub fn read_body<T>(&self) -> T {
        self.read(Self::OFFSET_BODY)
    }

    pub fn deserialize_body<Out: Archive>(&self) -> Result<Out, CUerror>
    where
        <Out as Archive>::Archived: Deserialize<Out, Strategy<(), Failure>>,
    {
        let size = self.read_size();
        let slice = unsafe {
            slice::from_raw_parts(
                self.view
                    .0
                    .Value
                    .wrapping_byte_add(Self::OFFSET_BODY)
                    .cast(),
                size as usize,
            )
        };
        unsafe { rkyv::api::low::from_bytes_unchecked(slice) }.map_err(|_| CUerror::UNKNOWN)
    }

    pub fn deserialize_body2<Out: Archive>(&self) -> &Out::Archived
    where
        <Out as Archive>::Archived: Portable,
    {
        let size = self.read_size();
        let slice = unsafe {
            slice::from_raw_parts(
                self.view
                    .0
                    .Value
                    .wrapping_byte_add(Self::OFFSET_BODY)
                    .cast(),
                size as usize,
            )
        };
        unsafe { rkyv::access_unchecked::<Out::Archived>(slice) }
    }

    pub fn serialize_body(
        &mut self,
        arena: &mut stumpalo::Arena,
        body: &impl for<'a, 'b> Serialize<Serializer<'a, 'b>>,
    ) -> Result<Option<SharedMemory>, CUerror> {
        let mut dropped_shmem = None;
        let length = loop {
            let mut serializer_base = rkyv::ser::Serializer::new(
                SliceWriter {
                    offset: 16,
                    slice: unsafe {
                        slice::from_raw_parts_mut(self.view.0.Value.cast(), self.size)
                    },
                },
                ScopedArena { arena },
                (),
            );
            let serializer = Strategy::<_, AllocError>::wrap(&mut serializer_base);
            match rkyv::api::serialize_using(body, serializer) {
                Ok(_) => break serializer_base.writer.pos() - Self::OFFSET_BODY,
                Err(AllocError::Stumpalo) => {
                    return Err(CUerror::OUT_OF_MEMORY);
                }
                Err(AllocError::NotEnoughMemory { more_bytes }) => {
                    let new_size = (self.size + more_bytes).next_power_of_two();
                    let new_shared_memory_name = random_global_name();
                    let new_shared_memory = unsafe {
                        Owned::new(
                            CreateFileMappingA(
                                INVALID_HANDLE_VALUE,
                                None,
                                PAGE_READWRITE,
                                0,
                                new_size as u32,
                                PCSTR(new_shared_memory_name.as_ptr()),
                            )
                            .map_err(|_| CUerror::MAP_FAILED)?,
                        )
                    };
                    let new_view = OwnedView::new(*new_shared_memory, new_size)
                        .map_err(|_| CUerror::MAP_FAILED)?;
                    let new_shmem = SharedMemory {
                        handle: new_shared_memory,
                        view: new_view,
                        size: new_size,
                        name: new_shared_memory_name,
                    };
                    let old_shmem = mem::replace(self, new_shmem);
                    dropped_shmem.get_or_insert(old_shmem);
                }
            }
        };
        self.write_size(length as u32);
        Ok(dropped_shmem)
    }
}

fn random_global_name() -> String {
    let name = Alphanumeric.sample_string(&mut rand::rng(), 32);
    format!("Local\\zluda-{name}\0")
}

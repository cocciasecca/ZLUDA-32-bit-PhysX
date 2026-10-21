//! Experimental fixed-capacity descriptor pool. Host texrefs NEVER own pooled objects.
use std::collections::{BTreeMap, BTreeSet};
const SIZE: usize = 88;
const OBJECT: usize = 72;
const LIMIT: usize = 64;
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    pointer: usize,
    bytes: usize,
    config: [u8; SIZE],
}
#[derive(Clone)]
struct Binding {
    global: usize,
    host: usize,
    pointer: usize,
    bytes: usize,
    key: Key,
}
#[derive(Default)]
pub struct Pool {
    objects: BTreeMap<Key, u64>,
    validated: BTreeSet<(usize, usize)>,
    active: BTreeMap<u32, Binding>,
    disabled: bool,
    pub hits: u64,
    pub misses: u64,
    pub fallbacks: u64,
}
fn u32_at(b: &[u8; SIZE], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
// Deliberately limited to observed linear integer/float descriptors. Everything else is native.
fn supported(b: &[u8; SIZE]) -> bool {
    u32_at(b, 0) == 0
        && u32_at(b, 4) <= 1
        && u32_at(b, 8) == 0
        && [12, 16, 20].iter().all(|o| u32_at(b, *o) <= 3)
        && u32_at(b, 44) == 0
        && u32_at(b, 48) <= 1
        && u32_at(b, 52) == 0
        && [56, 60, 64].iter().all(|o| u32_at(b, *o) == 0)
        && matches!(u32_at(b, 80), 1 | 2 | 4)
        && matches!(u32_at(b, 84), 3 | 32)
}
#[cfg(any(windows, test))]
mod native {
    #[cfg(test)]
    use super::mock::*;
    use super::*;
    #[cfg(not(test))]
    use crate::*;
    #[cfg(not(test))]
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn ReadProcessMemory(
            p: *mut std::ffi::c_void,
            a: *const std::ffi::c_void,
            b: *mut std::ffi::c_void,
            n: usize,
            r: *mut usize,
        ) -> i32;
    }
    fn read(host: usize) -> Option<[u8; SIZE]> {
        let mut b = [0; SIZE];
        let mut n = 0;
        let ok = unsafe {
            ReadProcessMemory(
                (-1isize) as *mut _,
                host as *const _,
                b.as_mut_ptr().cast(),
                SIZE,
                &mut n,
            )
        };
        (ok != 0 && n == SIZE).then_some(b)
    }
    fn object(b: &[u8; SIZE]) -> u64 {
        u64::from_le_bytes(b[OBJECT..OBJECT + 8].try_into().unwrap())
    }
    fn write(global: usize, b: &[u8; SIZE], obj: u64) -> Result<(), CUerror> {
        let mut copy = *b;
        copy[OBJECT..OBJECT + 8].copy_from_slice(&obj.to_le_bytes());
        unsafe { cuMemcpyHtoD_v2(CUdeviceptr_v2(global as *mut _), copy.as_ptr().cast(), SIZE) }
    }
    fn bind_native(host: usize, pointer: usize, bytes: usize) -> Result<usize, CUerror> {
        let mut offset = 0;
        unsafe {
            cuTexRefSetAddress_v2(
                &mut offset,
                host as CUtexref,
                CUdeviceptr_v2(pointer as *mut _),
                bytes,
            )
        }?;
        Ok(offset)
    }
    fn create(k: &Key) -> Result<u64, CUerror> {
        let mut resource: CUDA_RESOURCE_DESC = unsafe { std::mem::zeroed() };
        resource.resType = CUresourcetype::CU_RESOURCE_TYPE_LINEAR;
        resource.res.linear = CUDA_RESOURCE_DESC_st__bindgen_ty_1__bindgen_ty_3 {
            devPtr: CUdeviceptr_v2(k.pointer as *mut _),
            format: CUarray_format(u32_at(&k.config, 84)),
            numChannels: u32_at(&k.config, 80),
            sizeInBytes: k.bytes,
        };
        let mut texture: CUDA_TEXTURE_DESC = unsafe { std::mem::zeroed() };
        for (i, o) in [12, 16, 20].iter().enumerate() {
            texture.addressMode[i] = CUaddress_mode(u32_at(&k.config, *o));
        }
        texture.filterMode = CUfilter_mode(0);
        texture.flags = if u32_at(&k.config, 4) == 0 { 1 } else { 0 };
        texture.maxAnisotropy = u32_at(&k.config, 48);
        let mut obj = 0;
        unsafe { cuTexObjectCreate(&mut obj, &resource, &texture, std::ptr::null()) }?;
        Ok(obj)
    }
    impl Pool {
        // Restore native ownership before destroying pool objects. All mutations remain serialized.
        pub fn drain(&mut self) -> Result<(), CUerror> {
            if self.objects.is_empty() && self.active.is_empty() {
                self.validated.clear();
                return Ok(());
            }
            unsafe { cuCtxSynchronize() }?;
            let handles: Vec<_> = self.active.keys().copied().collect();
            for h in handles {
                let b = self.active.get(&h).unwrap().clone();
                // Native host copy is null, so it cannot destroy the pool object.
                bind_native(b.host, b.pointer, b.bytes)?;
                self.active.remove(&h);
            }
            self.validated.clear();
            let keys: Vec<_> = self.objects.keys().cloned().collect();
            for key in keys {
                let obj = self.objects.remove(&key).unwrap();
                unsafe { cuTexObjectDestroy(obj) }?;
            }
            Ok(())
        }
        pub fn configuration_changed(&mut self, h: u32) -> Result<(), CUerror> {
            if let Some(b) = self.active.get(&h).cloned() {
                if let Some(mut c) = read(b.host) {
                    c[OBJECT..OBJECT + 8].fill(0);
                    if c == b.key.config {
                        return Ok(());
                    }
                }
                if self
                    .address(h, b.host, Some(b.global), b.pointer, b.bytes)?
                    .is_none()
                {
                    bind_native(b.host, b.pointer, b.bytes)?;
                }
            }
            Ok(())
        }
        // None means no publication was made: caller can safely use the native path.
        pub fn address(
            &mut self,
            h: u32,
            host: usize,
            global: Option<usize>,
            pointer: usize,
            bytes: usize,
        ) -> Result<Option<usize>, CUerror> {
            if self.disabled {
                self.drain()?;
                return Ok(None);
            }
            if pointer == 0 {
                if let Some(b) = self.active.get(&h).cloned() {
                    unsafe { cuCtxSynchronize() }?;
                    write(b.global, &b.key.config, 0)?;
                    self.active.remove(&h);
                    return Ok(Some(0));
                }
                return Ok(None);
            }
            let Some(mut config) = read(host) else {
                self.drain()?;
                self.disabled = true;
                self.fallbacks += 1;
                return Ok(None);
            };
            if !supported(&config) || object(&config) != 0 || global.is_none() || pointer % 256 != 0
            {
                self.drain()?;
                self.fallbacks += 1;
                return Ok(None);
            }
            config[OBJECT..OBJECT + 8].fill(0);
            let key = Key {
                pointer,
                bytes,
                config,
            };
            let global = global.unwrap();
            if let Some(obj) = self
                .objects
                .get(&key)
                .copied()
                .filter(|_| self.validated.contains(&(host, global)))
            {
                unsafe { cuCtxSynchronize() }?;
                write(global, &key.config, obj)?;
                self.active.insert(
                    h,
                    Binding {
                        global,
                        host,
                        pointer,
                        bytes,
                        key,
                    },
                );
                self.hits += 1;
                return Ok(Some(0));
            }
            if self.objects.len() >= LIMIT && !self.objects.contains_key(&key) {
                // First prototype never evicts pinned objects. Saturation disables pooling for this context.
                self.drain()?;
                self.disabled = true;
                self.fallbacks += 1;
                return Ok(None);
            }
            // Validate the native offset and device/host object association for this miss.
            unsafe { cuCtxSynchronize() }?;
            let offset = bind_native(host, pointer, bytes)?;
            // Native bind superseded any previous device binding, without owning that pooled object.
            self.active.remove(&h);
            let Some(bound) = read(host) else {
                self.disabled = true;
                return Ok(Some(offset));
            };
            let mut device = [0u8; SIZE];
            if offset != 0
                || unsafe {
                    cuMemcpyDtoH_v2(
                        device.as_mut_ptr().cast(),
                        CUdeviceptr_v2(global as *mut _),
                        SIZE,
                    )
                }
                .is_err()
                || object(&bound) == 0
                || object(&bound) != object(&device)
            {
                self.fallbacks += 1;
                return Ok(Some(offset));
            }
            // Create independent ownership. If this fails the native binding is still valid.
            self.validated.insert((host, global));
            let obj = match self
                .objects
                .get(&key)
                .copied()
                .map(Ok)
                .unwrap_or_else(|| create(&key))
            {
                Ok(obj) => obj,
                Err(_) => {
                    self.fallbacks += 1;
                    return Ok(Some(offset));
                }
            };
            if let Err(e) = bind_native(host, 0, 0) {
                if !self.objects.contains_key(&key) {
                    let _ = unsafe { cuTexObjectDestroy(obj) };
                }
                return Err(e);
            }
            self.objects.insert(key.clone(), obj);
            // Pin before publication; if copy fails, cleanup must restore native state before freeing.
            self.active.insert(
                h,
                Binding {
                    global,
                    host,
                    pointer,
                    bytes,
                    key: key.clone(),
                },
            );
            write(global, &key.config, obj)?;
            self.misses += 1;
            Ok(Some(0))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn config() -> [u8; SIZE] {
        let mut b = [0; SIZE];
        b[80..84].copy_from_slice(&4u32.to_le_bytes());
        b[84..88].copy_from_slice(&32u32.to_le_bytes());
        b
    }
    #[test]
    fn descriptor_whitelist() {
        let mut b = config();
        assert!(supported(&b));
        b[0] = 1;
        assert!(!supported(&b));
        b[0] = 0;
        b[8] = 1;
        assert!(!supported(&b));
        b[8] = 0;
        b[84] = 1;
        assert!(!supported(&b));
    }
    #[test]
    fn keys_distinguish_config_range_and_not_handle() {
        let k = Key {
            pointer: 256,
            bytes: 64,
            config: config(),
        };
        let mut changed = k.clone();
        changed.bytes = 65;
        assert!(k != changed);
        changed = k.clone();
        changed.config[4] = 1;
        assert!(k != changed);
        changed = k.clone();
        changed.pointer = 512;
        assert!(k != changed);
    }
}

// Portable mock executes the SAME pool control flow with independently owned native/pool objects.
#[cfg(test)]
#[allow(non_snake_case, non_camel_case_types, dead_code)]
mod mock {
    use super::*;
    use std::{cell::RefCell, ffi::c_void};
    #[cfg(not(windows))]
    #[derive(Debug, Copy, Clone, PartialEq)]
    pub struct CUerror(pub u32);
    #[cfg(not(windows))]
    impl CUerror {
        pub const INVALID_VALUE: Self = Self(1);
    }
    #[cfg(windows)]
    pub type CUerror = crate::CUerror;
    pub type CUtexref = *mut c_void;
    pub struct CUdeviceptr_v2(pub *mut c_void);
    pub struct CUarray_format(pub u32);
    pub struct CUfilter_mode(pub u32);
    pub struct CUaddress_mode(pub u32);
    pub struct CUresourcetype(pub u32);
    impl CUresourcetype {
        pub const CU_RESOURCE_TYPE_LINEAR: Self = Self(2);
    }
    pub struct CUDA_RESOURCE_DESC_st__bindgen_ty_1__bindgen_ty_3 {
        pub devPtr: CUdeviceptr_v2,
        pub format: CUarray_format,
        pub numChannels: u32,
        pub sizeInBytes: usize,
    }
    pub struct Res {
        pub linear: CUDA_RESOURCE_DESC_st__bindgen_ty_1__bindgen_ty_3,
    }
    pub struct CUDA_RESOURCE_DESC {
        pub resType: CUresourcetype,
        pub res: Res,
    }
    pub struct CUDA_TEXTURE_DESC {
        pub addressMode: [CUaddress_mode; 3],
        pub filterMode: CUfilter_mode,
        pub flags: u32,
        pub maxAnisotropy: u32,
    }
    #[derive(Default)]
    pub struct Mock {
        pub next: u64,
        pub objects: BTreeMap<u64, bool>,
        pub globals: BTreeMap<usize, usize>,
        pub syncs: usize,
        pub destroys: usize,
        pub fail_create: bool,
        pub fail_sync: bool,
        pub fail_write: bool,
    }
    thread_local! {pub static STATE:RefCell<Mock>=RefCell::new(Mock::default());}
    pub unsafe fn ReadProcessMemory(
        _: *mut c_void,
        a: *const c_void,
        b: *mut c_void,
        n: usize,
        r: *mut usize,
    ) -> i32 {
        std::ptr::copy_nonoverlapping(a.cast::<u8>(), b.cast::<u8>(), n);
        *r = n;
        1
    }
    pub unsafe fn cuCtxSynchronize() -> Result<(), CUerror> {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.syncs += 1;
            if s.fail_sync {
                Err(CUerror::INVALID_VALUE)
            } else {
                Ok(())
            }
        })
    }
    pub unsafe fn cuMemcpyHtoD_v2(
        dst: CUdeviceptr_v2,
        src: *const c_void,
        n: usize,
    ) -> Result<(), CUerror> {
        if STATE.with(|s| s.borrow().fail_write) {
            return Err(CUerror::INVALID_VALUE);
        }
        std::ptr::copy_nonoverlapping(src.cast::<u8>(), dst.0.cast::<u8>(), n);
        Ok(())
    }
    pub unsafe fn cuMemcpyDtoH_v2(
        dst: *mut c_void,
        src: CUdeviceptr_v2,
        n: usize,
    ) -> Result<(), CUerror> {
        std::ptr::copy_nonoverlapping(src.0.cast::<u8>(), dst.cast::<u8>(), n);
        Ok(())
    }
    fn new_object(pool: bool) -> u64 {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.next += 1;
            let id = s.next;
            s.objects.insert(id, pool);
            id
        })
    }
    pub unsafe fn cuTexObjectCreate(
        out: *mut u64,
        _: *const CUDA_RESOURCE_DESC,
        _: *const CUDA_TEXTURE_DESC,
        _: *const c_void,
    ) -> Result<(), CUerror> {
        if STATE.with(|s| s.borrow().fail_create) {
            return Err(CUerror::INVALID_VALUE);
        }
        *out = new_object(true);
        Ok(())
    }
    pub unsafe fn cuTexObjectDestroy(obj: u64) -> Result<(), CUerror> {
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            assert!(s.objects.remove(&obj).is_some(), "double free");
            s.destroys += 1;
        });
        Ok(())
    }
    pub unsafe fn cuTexRefSetAddress_v2(
        offset: *mut usize,
        host: CUtexref,
        p: CUdeviceptr_v2,
        _: usize,
    ) -> Result<(), CUerror> {
        let host = &mut *(host as *mut [u8; SIZE]);
        let old = u64::from_le_bytes(host[OBJECT..OBJECT + 8].try_into().unwrap());
        if old != 0 {
            assert!(
                !STATE.with(|s| s.borrow().objects[&old]),
                "legacy setter must not own a pooled object"
            );
            cuTexObjectDestroy(old)?;
        }
        let new = if p.0.is_null() { 0 } else { new_object(false) };
        host[OBJECT..OBJECT + 8].copy_from_slice(&new.to_le_bytes());
        *offset = 0;
        if new != 0 {
            let global = STATE.with(|s| s.borrow().globals[&(host.as_ptr() as usize)]);
            std::ptr::copy_nonoverlapping(host.as_ptr(), global as *mut u8, SIZE);
        }
        Ok(())
    }
    pub fn fixture() -> (Box<[u8; SIZE]>, Box<[u8; SIZE]>) {
        let mut h = Box::new([0; SIZE]);
        h[80..84].copy_from_slice(&4u32.to_le_bytes());
        h[84..88].copy_from_slice(&32u32.to_le_bytes());
        let d = Box::new([0; SIZE]);
        STATE.with(|s| {
            s.borrow_mut()
                .globals
                .insert(h.as_ptr() as usize, d.as_ptr() as usize)
        });
        (h, d)
    }
    pub fn object(b: &[u8; SIZE]) -> u64 {
        u64::from_le_bytes(b[OBJECT..OBJECT + 8].try_into().unwrap())
    }
}
#[cfg(test)]
mod ownership_tests {
    use super::mock::*;
    use super::*;
    fn setup() {
        STATE.with(|s| *s.borrow_mut() = Mock::default());
    }
    #[test]
    fn repeated_bind_unbind_keeps_host_null_and_reuses_owned_object() {
        setup();
        let (h, d) = fixture();
        let mut p = Pool::default();
        for _ in 0..100 {
            assert_eq!(
                p.address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64),
                Ok(Some(0))
            );
            assert_eq!(object(&h), 0);
            assert_ne!(object(&d), 0);
            p.address(1, h.as_ptr() as usize, None, 0, 0).unwrap();
            assert_eq!(object(&d), 0);
        }
        assert_eq!(p.hits, 99);
        assert_eq!(p.objects.len(), 1);
        p.drain().unwrap();
        assert!(STATE.with(|s| s.borrow().objects.is_empty()));
    }
    #[test]
    fn shared_object_stays_alive_until_all_native_bindings_restored() {
        setup();
        let (h, d) = fixture();
        let (h2, d2) = fixture();
        let mut p = Pool::default();
        p.address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64)
            .unwrap();
        p.address(2, h2.as_ptr() as usize, Some(d2.as_ptr() as usize), 256, 64)
            .unwrap();
        assert_eq!(object(&d), object(&d2));
        p.drain().unwrap();
        assert_eq!(object(&h), object(&d));
        assert_eq!(object(&h2), object(&d2));
        assert!(STATE.with(|s| s.borrow().objects.values().all(|pool| !*pool)));
    }
    #[test]
    fn setter_while_bound_rebuilds_only_when_changed() {
        setup();
        let (mut h, d) = fixture();
        let mut p = Pool::default();
        p.address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64)
            .unwrap();
        let first = object(&d);
        p.configuration_changed(1).unwrap();
        assert_eq!(first, object(&d));
        h[4] = 1;
        p.configuration_changed(1).unwrap();
        assert_ne!(first, object(&d));
        assert_eq!(object(&h), 0);
        p.drain().unwrap();
    }
    #[test]
    fn capacity_saturation_restores_native_and_disables() {
        setup();
        let (h, d) = fixture();
        let mut p = Pool::default();
        for i in 1..=64 {
            p.address(
                1,
                h.as_ptr() as usize,
                Some(d.as_ptr() as usize),
                i * 256,
                64,
            )
            .unwrap();
        }
        assert_eq!(
            p.address(
                1,
                h.as_ptr() as usize,
                Some(d.as_ptr() as usize),
                65 * 256,
                64
            ),
            Ok(None)
        );
        assert!(p.disabled);
        assert!(p.objects.is_empty());
        assert_eq!(object(&h), object(&d));
    }
    #[test]
    fn failure_paths_preserve_ownership() {
        setup();
        let (h, d) = fixture();
        let mut p = Pool::default();
        STATE.with(|s| s.borrow_mut().fail_create = true);
        assert_eq!(
            p.address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64),
            Ok(Some(0))
        );
        assert_eq!(object(&h), object(&d));
        assert!(p.objects.is_empty());
    }
    #[test]
    fn failed_first_publication_is_pinned_until_native_restoration() {
        setup();
        let (h, d) = fixture();
        let mut p = Pool::default();
        STATE.with(|s| s.borrow_mut().fail_write = true);
        assert!(p
            .address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64)
            .is_err());
        assert_eq!(p.active.len(), 1);
        assert_eq!(p.objects.len(), 1);
        assert_eq!(object(&h), 0);
        STATE.with(|s| s.borrow_mut().fail_write = false);
        p.drain().unwrap();
        assert_eq!(object(&h), object(&d));
        assert!(p.objects.is_empty());
    }
    #[test]
    fn unsupported_configuration_returns_native_fallback() {
        setup();
        let (mut h, d) = fixture();
        h[0] = 1;
        let mut p = Pool::default();
        assert_eq!(
            p.address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64),
            Ok(None)
        );
        assert!(p.objects.is_empty());
        assert_eq!(object(&h), 0);
    }
    #[test]
    fn sync_or_publish_failure_does_not_free_live_object() {
        setup();
        let (h, d) = fixture();
        let mut p = Pool::default();
        p.address(1, h.as_ptr() as usize, Some(d.as_ptr() as usize), 256, 64)
            .unwrap();
        let obj = object(&d);
        STATE.with(|s| s.borrow_mut().fail_sync = true);
        assert!(p.address(1, h.as_ptr() as usize, None, 0, 0).is_err());
        assert_eq!(object(&d), obj);
        assert!(p.objects.values().any(|o| *o == obj));
        STATE.with(|s| {
            let mut s = s.borrow_mut();
            s.fail_sync = false;
            s.fail_write = true;
        });
        assert!(p.address(1, h.as_ptr() as usize, None, 0, 0).is_err());
        assert_eq!(object(&d), obj);
        STATE.with(|s| s.borrow_mut().fail_write = false);
        p.drain().unwrap();
    }
}

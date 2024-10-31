// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

use std::{
    mem::MaybeUninit,
    num::{NonZeroU16, NonZeroU32, NonZeroUsize},
    ptr::NonNull,
};

use windows::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};

use crate::runtime::{
    fail::Fail,
    libxdp,
    memory::{BufferPool, DemiBuffer},
};

//======================================================================================================================
// Structures
//======================================================================================================================

/// A wrapper structure for a XDP user memory region.
pub struct UmemReg {
    _buffer: Vec<MaybeUninit<u8>>,
    pool: BufferPool,
    umem: libxdp::XSK_UMEM_REG,
}

//======================================================================================================================
// Implementations
//======================================================================================================================

impl UmemReg {
    /// Creates a new XDP user memory region with `count` blocks of `chunk_size` bytes.
    pub fn new(count: NonZeroU32, chunk_size: NonZeroU16) -> Result<Self, Fail> {
        let total_size: u64 = count.get() as u64 * chunk_size.get() as u64;
        let mut buffer: Vec<MaybeUninit<u8>> = Vec::with_capacity(total_size as usize);

        let pool: BufferPool =
            BufferPool::new(chunk_size.get()).map_err(|_| Fail::new(libc::EINVAL, "bad buffer size"))?;

        let page_size: NonZeroUsize = get_page_size();
        unsafe { pool.pool().populate(NonNull::from(buffer.as_mut_slice()), page_size)? };

        if pool.pool().is_empty() {
            return Err(Fail::new(libc::ENOMEM, "out of memory"));
        }

        let umem: libxdp::XSK_UMEM_REG = libxdp::XSK_UMEM_REG {
            TotalSize: total_size,
            ChunkSize: chunk_size.get() as u32,
            Headroom: u32::try_from(DemiBuffer::metadata_size()).map_err(Fail::from)?,
            Address: buffer.as_mut_ptr() as *mut core::ffi::c_void,
        };

        Ok(Self {
            _buffer: buffer,
            pool,
            umem,
        })
    }

    /// Get a buffer from the umem pool.
    pub fn get_buffer(&self) -> Option<DemiBuffer> {
        DemiBuffer::new_in_pool(&self.pool)
    }

    /// Gets a reference to the underlying XDP user memory region.
    pub fn as_ref(&self) -> &libxdp::XSK_UMEM_REG {
        &self.umem
    }

    /// Returns a raw pointer to the the start address of the user memory region.
    pub fn address(&self) -> *mut core::ffi::c_void {
        self.umem.Address
    }
}

//======================================================================================================================
// Functions
//======================================================================================================================

fn get_page_size() -> NonZeroUsize {
    let mut si: SYSTEM_INFO = SYSTEM_INFO::default();

    // Safety: `si` is allocated and aligned correctly for Windows API access.
    unsafe { GetSystemInfo(&mut si as *mut SYSTEM_INFO) };

    NonZeroUsize::new(si.dwPageSize as usize).expect("invariant violation from Windows API: zero page size")
}

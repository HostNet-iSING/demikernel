// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

//==============================================================================
// Imports
//==============================================================================

use crate::{
    catcollar::{
        runtime::RequestId,
        IoUringRuntime,
    },
    runtime::{
        fail::Fail,
        memory::DemiBuffer,
        QDesc,
    },
};
use ::std::{
    future::Future,
    net::SocketAddrV4,
    os::fd::RawFd,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

//==============================================================================
// Structures
//==============================================================================

/// Pushto Operation Descriptor
pub struct PushtoFuture {
    /// Underlying runtime.
    rt: IoUringRuntime,
    /// Associated queue descriptor.
    qd: QDesc,
    /// Associated file descriptor.
    fd: RawFd,
    /// Destination address.
    addr: SocketAddrV4,
    /// Associated receive buffer.
    buf: DemiBuffer,
}

//==============================================================================
// Associate Functions
//==============================================================================

/// Associate Functions for Pushto Operation Descriptors
impl PushtoFuture {
    /// Creates a descriptor for a pushto operation.
    pub fn new(rt: IoUringRuntime, qd: QDesc, fd: RawFd, addr: SocketAddrV4, buf: DemiBuffer) -> Self {
        Self { rt, qd, fd, addr, buf }
    }

    /// Returns the queue descriptor associated to the target push operation descriptor.
    pub fn get_qd(&self) -> QDesc {
        self.qd
    }
}

//==============================================================================
// Trait Implementations
//==============================================================================

/// Future Trait Implementation for Pushto Operation Descriptors
impl Future for PushtoFuture {
    type Output = Result<(), Fail>;

    /// Polls the target [PushtoFuture].
    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        let self_: &mut PushtoFuture = self.get_mut();
        let request_id: RequestId = self_.rt.pushto(self_.fd, self_.addr, self_.buf.clone())?;

        match self_.rt.peek(request_id) {
            // Operation completed.
            Ok((_, size)) if size >= 0 => {
                trace!("data pushed ({:?} bytes)", size);
                Poll::Ready(Ok(()))
            },
            // Operation not completed, thus parse errno to find out what happened.
            Ok((None, size)) if size < 0 => {
                let errno: i32 = -size;
                // Operation in progress.
                if errno == libc::EWOULDBLOCK || errno == libc::EAGAIN {
                    ctx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                // Operation failed.
                else {
                    let message: String = format!("pushto(): operation failed (errno={:?})", errno);
                    error!("{}", message);
                    return Poll::Ready(Err(Fail::new(errno, &message)));
                }
            },
            // Operation failed.
            Err(e) => {
                warn!("push failed ({:?})", e);
                Poll::Ready(Err(e))
            },
            // Should not happen.
            _ => panic!("push failed: unknown error"),
        }
    }
}

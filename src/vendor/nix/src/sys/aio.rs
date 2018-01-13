use {Error, Errno, Result};
use std::os::unix::io::RawFd;
use libc::{c_void, off_t, size_t};
use libc;
use std::fmt;
use std::fmt::Debug;
use std::io::Write;
use std::io::stderr;
use std::marker::PhantomData;
use std::mem;
use std::ptr::{null, null_mut};
use sys::signal::*;
use sys::time::TimeSpec;

/// Mode for `AioCb::fsync`.  Controls whether only data or both data and
/// metadata are synced.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AioFsyncMode {
    /// do it like `fsync`
    O_SYNC = libc::O_SYNC,
    /// on supported operating systems only, do it like `fdatasync`
    #[cfg(any(target_os = "openbsd", target_os = "bitrig",
              target_os = "netbsd", target_os = "macos", target_os = "ios",
              target_os = "linux"))]
    O_DSYNC = libc::O_DSYNC
}

/// When used with `lio_listio`, determines whether a given `aiocb` should be
/// used for a read operation, a write operation, or ignored.  Has no effect for
/// any other aio functions.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LioOpcode {
    LIO_NOP = libc::LIO_NOP,
    LIO_WRITE = libc::LIO_WRITE,
    LIO_READ = libc::LIO_READ
}

/// Mode for `lio_listio`.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LioMode {
    /// Requests that `lio_listio` block until all requested operations have
    /// been completed
    LIO_WAIT = libc::LIO_WAIT,
    /// Requests that `lio_listio` return immediately
    LIO_NOWAIT = libc::LIO_NOWAIT,
}

/// Return values for `AioCb::cancel and aio_cancel_all`
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AioCancelStat {
    /// All outstanding requests were canceled
    AioCanceled = libc::AIO_CANCELED,
    /// Some requests were not canceled.  Their status should be checked with
    /// `AioCb::error`
    AioNotCanceled = libc::AIO_NOTCANCELED,
    /// All of the requests have already finished
    AioAllDone = libc::AIO_ALLDONE,
}

/// The basic structure used by all aio functions.  Each `aiocb` represents one
/// I/O request.
pub struct AioCb<'a> {
    aiocb: libc::aiocb,
    /// Tracks whether the buffer pointed to by aiocb.aio_buf is mutable
    mutable: bool,
    /// Could this `AioCb` potentially have any in-kernel state?
    in_progress: bool,
    phantom: PhantomData<&'a mut [u8]>
}

impl<'a> AioCb<'a> {
    /// Constructs a new `AioCb` with no associated buffer.
    ///
    /// The resulting `AioCb` structure is suitable for use with `AioCb::fsync`.
    /// * `fd`  File descriptor.  Required for all aio functions.
    /// * `prio` If POSIX Prioritized IO is supported, then the operation will
    /// be prioritized at the process's priority level minus `prio`
    /// * `sigev_notify` Determines how you will be notified of event
    /// completion.
    pub fn from_fd(fd: RawFd, prio: ::c_int,
                    sigev_notify: SigevNotify) -> AioCb<'a> {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = 0;
        a.aio_nbytes = 0;
        a.aio_buf = null_mut();

        let aiocb = AioCb { aiocb: a, mutable: false, in_progress: false,
                            phantom: PhantomData};
        aiocb
    }

    /// Constructs a new `AioCb`.
    ///
    /// * `fd`  File descriptor.  Required for all aio functions.
    /// * `offs` File offset
    /// * `buf` A memory buffer
    /// * `prio` If POSIX Prioritized IO is supported, then the operation will
    /// be prioritized at the process's priority level minus `prio`
    /// * `sigev_notify` Determines how you will be notified of event
    /// completion.
    /// * `opcode` This field is only used for `lio_listio`.  It determines
    /// which operation to use for this individual aiocb
    pub fn from_mut_slice(fd: RawFd, offs: off_t, buf: &'a mut [u8],
                          prio: ::c_int, sigev_notify: SigevNotify,
                          opcode: LioOpcode) -> AioCb {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf.len() as size_t;
        // casting an immutable buffer to a mutable pointer looks unsafe, but
        // technically its only unsafe to dereference it, not to create it.
        a.aio_buf = buf.as_ptr() as *mut c_void;
        a.aio_lio_opcode = opcode as ::c_int;

        let aiocb = AioCb { aiocb: a, mutable: true, in_progress: false,
                            phantom: PhantomData};
        aiocb
    }

    /// Like `from_mut_slice`, but works on constant slices rather than
    /// mutable slices.
    ///
    /// An `AioCb` created this way cannot be used with `read`, and its
    /// `LioOpcode` cannot be set to `LIO_READ`.  This method is useful when
    /// writing a const buffer with `AioCb::write`, since from_mut_slice can't
    /// work with const buffers.
    // Note: another solution to the problem of writing const buffers would be
    // to genericize AioCb for both &mut [u8] and &[u8] buffers.  AioCb::read
    // could take the former and AioCb::write could take the latter.  However,
    // then lio_listio wouldn't work, because that function needs a slice of
    // AioCb, and they must all be the same type.  We're basically stuck with
    // using an unsafe function, since aio (as designed in C) is an unsafe API.
    pub fn from_slice(fd: RawFd, offs: off_t, buf: &'a [u8],
                      prio: ::c_int, sigev_notify: SigevNotify,
                      opcode: LioOpcode) -> AioCb {
        let mut a = AioCb::common_init(fd, prio, sigev_notify);
        a.aio_offset = offs;
        a.aio_nbytes = buf.len() as size_t;
        a.aio_buf = buf.as_ptr() as *mut c_void;
        assert!(opcode != LioOpcode::LIO_READ, "Can't read into an immutable buffer");
        a.aio_lio_opcode = opcode as ::c_int;

        let aiocb = AioCb { aiocb: a, mutable: false, in_progress: false,
                            phantom: PhantomData};
        aiocb
    }

    fn common_init(fd: RawFd, prio: ::c_int,
                   sigev_notify: SigevNotify) -> libc::aiocb {
        // Use mem::zeroed instead of explicitly zeroing each field, because the
        // number and name of reserved fields is OS-dependent.  On some OSes,
        // some reserved fields are used the kernel for state, and must be
        // explicitly zeroed when allocated.
        let mut a = unsafe { mem::zeroed::<libc::aiocb>()};
        a.aio_fildes = fd;
        a.aio_reqprio = prio;
        a.aio_sigevent = SigEvent::new(sigev_notify).sigevent();
        a
    }

    /// Update the notification settings for an existing `aiocb`
    pub fn set_sigev_notify(&mut self, sigev_notify: SigevNotify) {
        self.aiocb.aio_sigevent = SigEvent::new(sigev_notify).sigevent();
    }

    /// Cancels an outstanding AIO request.
    pub fn cancel(&mut self) -> Result<AioCancelStat> {
        match unsafe { libc::aio_cancel(self.aiocb.aio_fildes, &mut self.aiocb) } {
            libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
            libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
            libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
            -1 => Err(Error::last()),
            _ => panic!("unknown aio_cancel return value")
        }
    }

    /// Retrieve error status of an asynchronous operation.  If the request has
    /// not yet completed, returns `EINPROGRESS`.  Otherwise, returns `Ok` or
    /// any other error.
    pub fn error(&mut self) -> Result<()> {
        match unsafe { libc::aio_error(&mut self.aiocb as *mut libc::aiocb) } {
            0 => Ok(()),
            num if num > 0 => Err(Error::from_errno(Errno::from_i32(num))),
            -1 => Err(Error::last()),
            num => panic!("unknown aio_error return value {:?}", num)
        }
    }

    /// An asynchronous version of `fsync`.
    pub fn fsync(&mut self, mode: AioFsyncMode) -> Result<()> {
        let p: *mut libc::aiocb = &mut self.aiocb;
        self.in_progress = true;
        Errno::result(unsafe { libc::aio_fsync(mode as ::c_int, p) }).map(drop)
    }

    /// Asynchronously reads from a file descriptor into a buffer
    pub fn read(&mut self) -> Result<()> {
        assert!(self.mutable, "Can't read into an immutable buffer");
        let p: *mut libc::aiocb = &mut self.aiocb;
        self.in_progress = true;
        Errno::result(unsafe { libc::aio_read(p) }).map(drop)
    }

    /// Retrieve return status of an asynchronous operation.  Should only be
    /// called once for each `AioCb`, after `AioCb::error` indicates that it has
    /// completed.  The result is the same as for `read`, `write`, of `fsync`.
    // Note: this should be just `return`, but that's a reserved word
    pub fn aio_return(&mut self) -> Result<isize> {
        let p: *mut libc::aiocb = &mut self.aiocb;
        self.in_progress = false;
        Errno::result(unsafe { libc::aio_return(p) })
    }

    /// Asynchronously writes from a buffer to a file descriptor
    pub fn write(&mut self) -> Result<()> {
        let p: *mut libc::aiocb = &mut self.aiocb;
        self.in_progress = true;
        Errno::result(unsafe { libc::aio_write(p) }).map(drop)
    }

}

/// Cancels outstanding AIO requests.  All requests for `fd` will be cancelled.
pub fn aio_cancel_all(fd: RawFd) -> Result<AioCancelStat> {
    match unsafe { libc::aio_cancel(fd, null_mut()) } {
        libc::AIO_CANCELED => Ok(AioCancelStat::AioCanceled),
        libc::AIO_NOTCANCELED => Ok(AioCancelStat::AioNotCanceled),
        libc::AIO_ALLDONE => Ok(AioCancelStat::AioAllDone),
        -1 => Err(Error::last()),
        _ => panic!("unknown aio_cancel return value")
    }
}

/// Suspends the calling process until at least one of the specified `AioCb`s
/// has completed, a signal is delivered, or the timeout has passed.  If
/// `timeout` is `None`, `aio_suspend` will block indefinitely.
pub fn aio_suspend(list: &[&AioCb], timeout: Option<TimeSpec>) -> Result<()> {
    // We must use transmute because Rust doesn't understand that a pointer to a
    // Struct is the same as a pointer to its first element.
    let plist = unsafe {
        mem::transmute::<&[&AioCb], *const [*const libc::aiocb]>(list)
    };
    let p = plist as *const *const libc::aiocb;
    let timep = match timeout {
        None    => null::<libc::timespec>(),
        Some(x) => x.as_ref() as *const libc::timespec
    };
    Errno::result(unsafe {
        libc::aio_suspend(p, list.len() as i32, timep)
    }).map(drop)
}


/// Submits multiple asynchronous I/O requests with a single system call.  The
/// order in which the requests are carried out is not specified.
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub fn lio_listio(mode: LioMode, list: &[&mut AioCb],
                  sigev_notify: SigevNotify) -> Result<()> {
    let sigev = SigEvent::new(sigev_notify);
    let sigevp = &mut sigev.sigevent() as *mut libc::sigevent;
    // We must use transmute because Rust doesn't understand that a pointer to a
    // Struct is the same as a pointer to its first element.
    let plist = unsafe {
        mem::transmute::<&[&mut AioCb], *const [*mut libc::aiocb]>(list)
    };
    let p = plist as *const *mut libc::aiocb;
    Errno::result(unsafe {
        libc::lio_listio(mode as i32, p, list.len() as i32, sigevp)
    }).map(drop)
}

impl<'a> Debug for AioCb<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("AioCb")
            .field("aio_fildes", &self.aiocb.aio_fildes)
            .field("aio_offset", &self.aiocb.aio_offset)
            .field("aio_buf", &self.aiocb.aio_buf)
            .field("aio_nbytes", &self.aiocb.aio_nbytes)
            .field("aio_lio_opcode", &self.aiocb.aio_lio_opcode)
            .field("aio_reqprio", &self.aiocb.aio_reqprio)
            .field("aio_sigevent", &SigEvent::from(&self.aiocb.aio_sigevent))
            .field("mutable", &self.mutable)
            .field("in_progress", &self.in_progress)
            .field("phantom", &self.phantom)
            .finish()
    }
}

impl<'a> Drop for AioCb<'a> {
    /// If the `AioCb` has no remaining state in the kernel, just drop it.
    /// Otherwise, collect its error and return values, so as not to leak
    /// resources.
    fn drop(&mut self) {
        if self.in_progress {
            // Well-written programs should never get here.  They should always
            // wait for an AioCb to complete before dropping it
            let _ = write!(stderr(), "WARNING: dropped an in-progress AioCb");
            loop {
                let ret = aio_suspend(&[&self], None);
                match ret {
                    Ok(()) => break,
                    Err(Error::Sys(Errno::EINVAL)) => panic!(
                        "Inconsistent AioCb.in_progress value"),
                    Err(Error::Sys(Errno::EINTR)) => (),    // Retry interrupted syscall
                    _ => panic!("Unexpected aio_suspend return value {:?}", ret)
                };
            }
            let _ = self.aio_return();
        }
    }
}

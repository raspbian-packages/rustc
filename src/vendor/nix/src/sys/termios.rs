use {Errno, Result};
use libc::c_int;
use std::mem;
use std::os::unix::io::RawFd;

pub use self::ffi::consts::*;
pub use self::ffi::consts::SetArg::*;
pub use self::ffi::consts::FlushArg::*;
pub use self::ffi::consts::FlowArg::*;

mod ffi {
    pub use self::consts::*;

    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd", target_os = "linux"))]
    mod non_android {
        use super::consts::*;
        use libc::c_int;

        // `Termios` contains bitflags which are not considered
        // `foreign-function-safe` by the compiler.
        #[allow(improper_ctypes)]
        extern {
            pub fn cfgetispeed(termios: *const Termios) -> speed_t;
            pub fn cfgetospeed(termios: *const Termios) -> speed_t;
            pub fn cfsetispeed(termios: *mut Termios, speed: speed_t) -> c_int;
            pub fn cfsetospeed(termios: *mut Termios, speed: speed_t) -> c_int;
            pub fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int;
            pub fn tcsetattr(fd: c_int,
                             optional_actions: c_int,
                             termios: *const Termios) -> c_int;
            pub fn tcdrain(fd: c_int) -> c_int;
            pub fn tcflow(fd: c_int, action: c_int) -> c_int;
            pub fn tcflush(fd: c_int, action: c_int) -> c_int;
            pub fn tcsendbreak(fd: c_int, duration: c_int) -> c_int;
        }
    }

    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd", target_os = "linux"))]
    pub use self::non_android::*;

    // On Android before 5.0, Bionic directly inline these to ioctl() calls.
    #[inline]
    #[cfg(all(target_os = "android", not(target_arch = "mips")))]
    mod android {
        use libc;
        use libc::c_int;
        use super::consts::*;

        const TCGETS: c_int = 0x5401;
        const TCSBRK: c_int = 0x5409;
        const TCXONC: c_int = 0x540a;
        const TCFLSH: c_int = 0x540b;
        const TCSBRKP: c_int = 0x5425;

        pub unsafe fn cfgetispeed(termios: *const Termios) -> speed_t {
            ((*termios).c_cflag & CBAUD).bits() as speed_t
        }
        pub unsafe fn cfgetospeed(termios: *const Termios) -> speed_t {
            ((*termios).c_cflag & CBAUD).bits() as speed_t
        }
        pub unsafe fn cfsetispeed(termios: *mut Termios, speed: speed_t) -> c_int {
            (*termios).c_cflag.remove(CBAUD);
            (*termios).c_cflag.insert(ControlFlags::from_bits_truncate(speed) & CBAUD);
            0
        }
        pub unsafe fn cfsetospeed(termios: *mut Termios, speed: speed_t) -> c_int {
            (*termios).c_cflag.remove(CBAUD);
            (*termios).c_cflag.insert(ControlFlags::from_bits_truncate(speed) & CBAUD);
            0
        }
        pub unsafe fn tcgetattr(fd: c_int, termios: *mut Termios) -> c_int {
            libc::ioctl(fd, TCGETS, termios)
        }
        pub unsafe fn tcsetattr(fd: c_int,
                                optional_actions: c_int,
                                termios: *const Termios) -> c_int {
            libc::ioctl(fd, optional_actions, termios)
        }
        pub unsafe fn tcdrain(fd: c_int) -> c_int {
            libc::ioctl(fd, TCSBRK, 1)
        }
        pub unsafe fn tcflow(fd: c_int, action: c_int) -> c_int {
            libc::ioctl(fd, TCXONC, action)
        }
        pub unsafe fn tcflush(fd: c_int, action: c_int) -> c_int {
            libc::ioctl(fd, TCFLSH, action)
        }
        pub unsafe fn tcsendbreak(fd: c_int, duration: c_int) -> c_int {
            libc::ioctl(fd, TCSBRKP, duration)
        }
    }

    #[cfg(target_os = "android")]
    pub use self::android::*;


    #[cfg(any(target_os = "macos", target_os = "freebsd", target_os = "dragonfly", target_os = "openbsd", target_os = "netbsd"))]
    pub mod consts {

        use libc;

        use libc::{c_int, c_uint, c_ulong, c_uchar};

        pub type tcflag_t = libc::tcflag_t;
        pub type cc_t = libc::cc_t;
        pub type speed_t = libc::speed_t;

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct Termios {
            pub c_iflag: InputFlags,
            pub c_oflag: OutputFlags,
            pub c_cflag: ControlFlags,
            pub c_lflag: LocalFlags,
            pub c_cc: [cc_t; NCCS],
            pub c_ispeed: speed_t,
            pub c_ospeed: speed_t
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum BaudRate {
            B0,
            B50,
            B75,
            B110,
            B134,
            B150,
            B200,
            B300,
            B600,
            B1200,
            B1800,
            B2400,
            B4800,
            B9600,
            B19200,
            B38400,
            B7200,
            B14400,
            B28800,
            B57600,
            B76800,
            B115200,
            B230400,
            #[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
            B460800,
            #[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
            B921600,
        }

        impl From<speed_t> for BaudRate {
            fn from(s: speed_t) -> BaudRate {

                use libc::{
                    B0, B50, B75, B110, B134, B150,
                    B200, B300, B600, B1200, B1800, B2400,
                    B4800, B9600, B19200, B38400,
                    B7200, B14400, B28800, B57600,
                    B76800, B115200, B230400};

                #[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
                use libc::{B460800, B921600};

                match s {
                    B0 => BaudRate::B0,
                    B50 => BaudRate::B50,
                    B75 => BaudRate::B75,
                    B110 => BaudRate::B110,
                    B134 => BaudRate::B134,
                    B150 => BaudRate::B150,
                    B200 => BaudRate::B200,
                    B300 => BaudRate::B300,
                    B600 => BaudRate::B600,
                    B1200 => BaudRate::B1200,
                    B1800 => BaudRate::B1800,
                    B2400 => BaudRate::B2400,
                    B4800 => BaudRate::B4800,
                    B9600 => BaudRate::B9600,
                    B19200 => BaudRate::B19200,
                    B38400 => BaudRate::B38400,
                    B7200 => BaudRate::B7200,
                    B14400 => BaudRate::B14400,
                    B28800 => BaudRate::B28800,
                    B57600 => BaudRate::B57600,
                    B76800 => BaudRate::B76800,
                    B115200 => BaudRate::B115200,
                    B230400 => BaudRate::B230400,
                    #[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
                    B460800 => BaudRate::B460800,
                    #[cfg(any(target_os = "netbsd", target_os = "freebsd"))]
                    B921600 => BaudRate::B921600,
                    b @ _ => unreachable!("Invalid baud constant: {}", b),
                }
            }
        }

        pub const VEOF: usize     = 0;
        pub const VEOL: usize     = 1;
        pub const VEOL2: usize    = 2;
        pub const VERASE: usize   = 3;
        pub const VWERASE: usize  = 4;
        pub const VKILL: usize    = 5;
        pub const VREPRINT: usize = 6;
        pub const VINTR: usize    = 8;
        pub const VQUIT: usize    = 9;
        pub const VSUSP: usize    = 10;
        pub const VDSUSP: usize   = 11;
        pub const VSTART: usize   = 12;
        pub const VSTOP: usize    = 13;
        pub const VLNEXT: usize   = 14;
        pub const VDISCARD: usize = 15;
        pub const VMIN: usize     = 16;
        pub const VTIME: usize    = 17;
        pub const VSTATUS: usize  = 18;
        pub const NCCS: usize     = 20;

        bitflags! {
            pub flags InputFlags: tcflag_t {
                const IGNBRK  = 0x00000001,
                const BRKINT  = 0x00000002,
                const IGNPAR  = 0x00000004,
                const PARMRK  = 0x00000008,
                const INPCK   = 0x00000010,
                const ISTRIP  = 0x00000020,
                const INLCR   = 0x00000040,
                const IGNCR   = 0x00000080,
                const ICRNL   = 0x00000100,
                const IXON    = 0x00000200,
                const IXOFF   = 0x00000400,
                const IXANY   = 0x00000800,
                const IMAXBEL = 0x00002000,
                #[cfg(not(target_os = "dragonfly"))]
                const IUTF8   = 0x00004000,
            }
        }

        bitflags! {
            pub flags OutputFlags: tcflag_t {
                const OPOST  = 0x00000001,
                const ONLCR  = 0x00000002,
                const OXTABS = 0x00000004,
                const ONOEOT = 0x00000008,
            }
        }

        bitflags! {
            pub flags ControlFlags: tcflag_t {
                const CIGNORE    = 0x00000001,
                const CSIZE      = 0x00000300,
                const CS5        = 0x00000000,
                const CS6        = 0x00000100,
                const CS7        = 0x00000200,
                const CS8        = 0x00000300,
                const CSTOPB     = 0x00000400,
                const CREAD      = 0x00000800,
                const PARENB     = 0x00001000,
                const PARODD     = 0x00002000,
                const HUPCL      = 0x00004000,
                const CLOCAL     = 0x00008000,
                const CCTS_OFLOW = 0x00010000,
                const CRTSCTS    = 0x00030000,
                const CRTS_IFLOW = 0x00020000,
                const CDTR_IFLOW = 0x00040000,
                const CDSR_OFLOW = 0x00080000,
                const CCAR_OFLOW = 0x00100000,
                const MDMBUF     = 0x00100000,
            }
        }

        bitflags! {
            pub flags LocalFlags: tcflag_t {
                const ECHOKE     = 0x00000001,
                const ECHOE      = 0x00000002,
                const ECHOK      = 0x00000004,
                const ECHO       = 0x00000008,
                const ECHONL     = 0x00000010,
                const ECHOPRT    = 0x00000020,
                const ECHOCTL    = 0x00000040,
                const ISIG       = 0x00000080,
                const ICANON     = 0x00000100,
                const ALTWERASE  = 0x00000200,
                const IEXTEN     = 0x00000400,
                const EXTPROC    = 0x00000800,
                const TOSTOP     = 0x00400000,
                const FLUSHO     = 0x00800000,
                const NOKERNINFO = 0x02000000,
                const PENDIN     = 0x20000000,
                const NOFLSH     = 0x80000000,
            }
        }

        pub const NL0: c_int  = 0x00000000;
        pub const NL1: c_int  = 0x00000100;
        pub const NL2: c_int  = 0x00000200;
        pub const NL3: c_int  = 0x00000300;
        pub const TAB0: c_int = 0x00000000;
        pub const TAB1: c_int = 0x00000400;
        pub const TAB2: c_int = 0x00000800;
        pub const TAB3: c_int = 0x00000004;
        pub const CR0: c_int  = 0x00000000;
        pub const CR1: c_int  = 0x00001000;
        pub const CR2: c_int  = 0x00002000;
        pub const CR3: c_int  = 0x00003000;
        pub const FF0: c_int  = 0x00000000;
        pub const FF1: c_int  = 0x00004000;
        pub const BS0: c_int  = 0x00000000;
        pub const BS1: c_int  = 0x00008000;
        pub const VT0: c_int  = 0x00000000;
        pub const VT1: c_int  = 0x00010000;

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub enum SetArg {
            TCSANOW   = 0,
            TCSADRAIN = 1,
            TCSAFLUSH = 2,
            TCSASOFT  = 16,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub enum FlushArg {
            TCIFLUSH  = 1,
            TCOFLUSH  = 2,
            TCIOFLUSH = 3,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub enum FlowArg {
            TCOOFF = 1,
            TCOON  = 2,
            TCIOFF = 3,
            TCION  = 4,
        }
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub mod consts {
        use libc::{c_int, c_uint, c_uchar};

        pub type tcflag_t = c_uint;
        pub type cc_t = c_uchar;
        pub type speed_t = c_uint;

        #[repr(C)]
        #[derive(Clone, Copy)]
        pub struct Termios {
            pub c_iflag: InputFlags,
            pub c_oflag: OutputFlags,
            pub c_cflag: ControlFlags,
            pub c_lflag: LocalFlags,
            pub c_line: cc_t,
            pub c_cc: [cc_t; NCCS],
            pub c_ispeed: speed_t,
            pub c_ospeed: speed_t
        }

        #[derive(Clone, Copy, Debug, PartialEq)]
        pub enum BaudRate {
            B0,
            B50,
            B75,
            B110,
            B134,
            B150,
            B200,
            B300,
            B600,
            B1200,
            B1800,
            B2400,
            B4800,
            B9600,
            B19200,
            B38400,
            B57600,
            B115200,
            B230400,
            B460800,
            B500000,
            B576000,
            B921600,
            B1000000,
            B1152000,
            B1500000,
            B2000000,
            B2500000,
            B3000000,
            B3500000,
            B4000000,
        }

        impl From<speed_t> for BaudRate {
            fn from(s: speed_t) -> BaudRate {

                use libc::{
                    B0, B50, B75, B110, B134, B150,
                    B200, B300, B600, B1200, B1800, B2400,
                    B4800, B9600, B19200, B38400, B57600,
                    B115200, B230400, B460800, B500000,
                    B576000, B921600, B1000000, B1152000,
                    B1500000, B2000000, B2500000, B3000000,
                    B3500000, B4000000};

                match s {
                    B0 => BaudRate::B0,
                    B50 => BaudRate::B50,
                    B75 => BaudRate::B75,
                    B110 => BaudRate::B110,
                    B134 => BaudRate::B134,
                    B150 => BaudRate::B150,
                    B200 => BaudRate::B200,
                    B300 => BaudRate::B300,
                    B600 => BaudRate::B600,
                    B1200 => BaudRate::B1200,
                    B1800 => BaudRate::B1800,
                    B2400 => BaudRate::B2400,
                    B4800 => BaudRate::B4800,
                    B9600 => BaudRate::B9600,
                    B19200 => BaudRate::B19200,
                    B38400 => BaudRate::B38400,
                    B57600 => BaudRate::B57600,
                    B115200 => BaudRate::B115200,
                    B230400 => BaudRate::B230400,
                    B460800 => BaudRate::B460800,
                    B500000 => BaudRate::B500000,
                    B576000 => BaudRate::B576000,
                    B921600 => BaudRate::B921600,
                    B1000000 => BaudRate::B1000000,
                    B1152000 => BaudRate::B1152000,
                    B1500000 => BaudRate::B1500000,
                    B2000000 => BaudRate::B2000000,
                    B2500000 => BaudRate::B2500000,
                    B3000000 => BaudRate::B3000000,
                    B3500000 => BaudRate::B3500000,
                    B4000000 => BaudRate::B4000000,
                    b @ _ => unreachable!("Invalid baud constant: {}", b),
                }
            }
        }

        pub const VEOF: usize     = 4;
        pub const VEOL: usize     = 11;
        pub const VEOL2: usize    = 16;
        pub const VERASE: usize   = 2;
        pub const VWERASE: usize  = 14;
        pub const VKILL: usize    = 3;
        pub const VREPRINT: usize = 12;
        pub const VINTR: usize    = 0;
        pub const VQUIT: usize    = 1;
        pub const VSUSP: usize    = 10;
        pub const VSTART: usize   = 8;
        pub const VSTOP: usize    = 9;
        pub const VLNEXT: usize   = 15;
        pub const VDISCARD: usize = 13;
        pub const VMIN: usize     = 6;
        pub const VTIME: usize    = 5;
        pub const NCCS: usize     = 32;

        bitflags! {
            pub flags InputFlags: tcflag_t {
                const IGNBRK  = 0x00000001,
                const BRKINT  = 0x00000002,
                const IGNPAR  = 0x00000004,
                const PARMRK  = 0x00000008,
                const INPCK   = 0x00000010,
                const ISTRIP  = 0x00000020,
                const INLCR   = 0x00000040,
                const IGNCR   = 0x00000080,
                const ICRNL   = 0x00000100,
                const IXON    = 0x00000400,
                const IXOFF   = 0x00001000,
                const IXANY   = 0x00000800,
                const IMAXBEL = 0x00002000,
                const IUTF8   = 0x00004000,
            }
        }

        bitflags! {
            pub flags OutputFlags: tcflag_t {
                const OPOST  = 0x00000001,
                const ONLCR  = 0x00000004,
            }
        }

        bitflags! {
            pub flags ControlFlags: tcflag_t {
                const CSIZE      = 0x00000030,
                const CS5        = 0x00000000,
                const CS6        = 0x00000010,
                const CS7        = 0x00000020,
                const CS8        = 0x00000030,
                const CSTOPB     = 0x00000040,
                const CREAD      = 0x00000080,
                const PARENB     = 0x00000100,
                const PARODD     = 0x00000200,
                const HUPCL      = 0x00000400,
                const CLOCAL     = 0x00000800,
                const CRTSCTS    = 0x80000000,
                #[cfg(target_os = "android")]
                const CBAUD      = 0o0010017,
            }
        }

        bitflags! {
            pub flags LocalFlags: tcflag_t {
                const ECHOKE     = 0x00000800,
                const ECHOE      = 0x00000010,
                const ECHOK      = 0x00000020,
                const ECHO       = 0x00000008,
                const ECHONL     = 0x00000040,
                const ECHOPRT    = 0x00000400,
                const ECHOCTL    = 0x00000200,
                const ISIG       = 0x00000001,
                const ICANON     = 0x00000002,
                const IEXTEN     = 0x00008000,
                const EXTPROC    = 0x00010000,
                const TOSTOP     = 0x00000100,
                const FLUSHO     = 0x00001000,
                const PENDIN     = 0x00004000,
                const NOFLSH     = 0x00000080,
            }
        }

        pub const NL0: c_int  = 0x00000000;
        pub const NL1: c_int  = 0x00000100;
        pub const TAB0: c_int = 0x00000000;
        pub const TAB1: c_int = 0x00000800;
        pub const TAB2: c_int = 0x00001000;
        pub const TAB3: c_int = 0x00001800;
        pub const CR0: c_int  = 0x00000000;
        pub const CR1: c_int  = 0x00000200;
        pub const CR2: c_int  = 0x00000400;
        pub const CR3: c_int  = 0x00000600;
        pub const FF0: c_int  = 0x00000000;
        pub const FF1: c_int  = 0x00008000;
        pub const BS0: c_int  = 0x00000000;
        pub const BS1: c_int  = 0x00002000;
        pub const VT0: c_int  = 0x00000000;
        pub const VT1: c_int  = 0x00004000;

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub enum SetArg {
            TCSANOW   = 0,
            TCSADRAIN = 1,
            TCSAFLUSH = 2,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub enum FlushArg {
            TCIFLUSH  = 0,
            TCOFLUSH  = 1,
            TCIOFLUSH = 2,
        }

        // XXX: We're using `repr(C)` because `c_int` doesn't work here.
        // See https://github.com/rust-lang/rust/issues/10374.
        #[derive(Clone, Copy)]
        #[repr(C)]
        pub enum FlowArg {
            TCOOFF = 0,
            TCOON  = 1,
            TCIOFF = 2,
            TCION  = 3,
        }
    }
}

pub fn cfgetispeed(termios: &Termios) -> BaudRate {
    unsafe {
        ffi::cfgetispeed(termios).into()
    }
}

pub fn cfgetospeed(termios: &Termios) -> BaudRate {
    unsafe {
        ffi::cfgetospeed(termios).into()
    }
}

pub fn cfsetispeed(termios: &mut Termios, baud: BaudRate) -> Result<()> {
    Errno::result(unsafe {
        ffi::cfsetispeed(termios, baud as speed_t)
    }).map(drop)
}

pub fn cfsetospeed(termios: &mut Termios, baud: BaudRate) -> Result<()> {
    Errno::result(unsafe {
        ffi::cfsetospeed(termios, baud as speed_t)
    }).map(drop)
}

pub fn tcgetattr(fd: RawFd) -> Result<Termios> {
    let mut termios = unsafe { mem::uninitialized() };

    let res = unsafe {
        ffi::tcgetattr(fd, &mut termios)
    };

    try!(Errno::result(res));

    Ok(termios)
}

pub fn tcsetattr(fd: RawFd,
                 actions: SetArg,
                 termios: &Termios) -> Result<()> {
    Errno::result(unsafe {
        ffi::tcsetattr(fd, actions as c_int, termios)
    }).map(drop)
}

pub fn tcdrain(fd: RawFd) -> Result<()> {
    Errno::result(unsafe {
        ffi::tcdrain(fd)
    }).map(drop)
}

pub fn tcflow(fd: RawFd, action: FlowArg) -> Result<()> {
    Errno::result(unsafe {
        ffi::tcflow(fd, action as c_int)
    }).map(drop)
}

pub fn tcflush(fd: RawFd, action: FlushArg) -> Result<()> {
    Errno::result(unsafe {
        ffi::tcflush(fd, action as c_int)
    }).map(drop)
}

pub fn tcsendbreak(fd: RawFd, action: c_int) -> Result<()> {
    Errno::result(unsafe {
        ffi::tcsendbreak(fd, action)
    }).map(drop)
}

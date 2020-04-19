/* Copyright 2020 Torbjørn Birch Moltu
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

//! Exposes the UDP-Lite socket type with an APi similar to `std::net::UdpSocket`.
//!
//! UDP-Lite is a layer 3 networking protocol very similar to UDP, that allows
//! receiving partially corrupted packets.
//! 
//! In addition to not being reliable (ie. datagrams can disappear), 
//! UDP-Lite is only useful if the layer 2 protocol supports disabling checksums,
//! and is not all that usable on the wider internet.
//! (My ISPs router doesn't recognize the protocol, so its NAT drops all packets.)
//! The protocol is also only implemented on Linux and FreeBSD.
//! (It looks like Android hasn't disabled it, but I'm not certain).
//!
//! this crate is tested on both (non-Android) Linux and FreeBSD.
//!
//! # Examples
//!
//! ```
//! use udplite::UdpLiteSocket;
//! use std::net::*;
//!
//! let a = UdpLiteSocket::bind((Ipv4Addr::LOCALHOST, 0))
//!     .expect("create UDP-Lite socket bound to 127.0.0.1:0");
//! let b = UdpLiteSocket::bind((Ipv4Addr::LOCALHOST, 0))
//!     .expect("create another socket bound to 127.0.0.1:0");
//!
//! // reduce sent and required checksum coverage (whole datagram by default)
//! a.set_send_checksum_coverage(Some(5)).expect("set partial checksum coverage");
//! b.set_recv_checksum_coverage_filter(Some(1)).expect("set required checksum coverage");
//!
//! let b_addr = b.local_addr().expect("get addr of socket b");
//! a.send_to(b"Hello UDP-Lite", b_addr).expect("send datagram");
//!
//! let mut buf = [0u8; 20];
//! let received_bytes = b.recv(&mut buf).expect("receive datagram");
//! assert_eq!(received_bytes, "Hello UDP-Lite".len());
//! assert_eq!(&buf[..5], b"Hello");
//! ```
//!
//! # Current implementation details
//!
//! To significantly reduce the amount of `unsafe` code necessary in this crate,
//! most methods are provided through `Deref` to [`UdpSocket`](https://doc.rust-lang.org/std/net/struct.UdpSocket.html).
//! This creates one wart/gotcha/unsoundness though:
//! `UdpSocket`s `.try_clone()` is available, returning an `UdpSocket` that is
//! actually UDP-Lite. The method is shadowed by [`UdpLiteSocket`](struct.UdpLiteSocket.html)s
//! own [`.try_clone()`]()(struct.UdpLiteSocket.html#method.try_clone)
//!
//! # Minimum Rust version
//!
//! udplite will require Rust 1.36.0 (for `std::io::IoSlice`).
//!
//! # Possible future features (open an issue if you want one)
//!
//! * Optional mio integration (both mio v0.6 and v0.7).
//! * Optional tokio integration.
//! * Vectored I/O (`std`s `UdpSocket` doesn't have this yet either).
//! * Exposing more POSIX socket options and flags for `send()` and `recv()`.
//! * Sending and receiving multiple datagrams at a time.
//! * Getting TTL and/or timestamp of received datagrams.

extern crate libc;

use std::os::raw::{c_int, c_void};
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::net::{UdpSocket, SocketAddr, ToSocketAddrs};
use std::{fmt, io, mem};
use std::io::ErrorKind::*;
use std::ops::Deref;
use std::fmt::Debug;

use libc::{AF_INET, AF_INET6, SOCK_DGRAM, IPPROTO_UDPLITE, SOCK_CLOEXEC};
use libc::{socket, bind, getsockopt, setsockopt, socklen_t};
use libc::{sockaddr_storage, sockaddr_in, sockaddr_in6, sockaddr, sa_family_t};
use libc::{UDPLITE_SEND_CSCOV, UDPLITE_RECV_CSCOV};

pub struct UdpLiteSocket {
    as_udp: UdpSocket,
}

impl Debug for UdpLiteSocket {
    fn fmt(&self,  fmtr: &mut fmt::Formatter) -> fmt::Result {
        let mut repr = fmtr.debug_struct("UdpLiteSocket");
        if let Ok(addr) = self.local_addr() {
            repr.field("addr", &addr);
        }
        repr.field("fd", &self.as_raw_fd());
        repr.finish()
    }
}

impl FromRawFd for UdpLiteSocket {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        UdpLiteSocket { as_udp: UdpSocket::from_raw_fd(fd) }
    }
}
impl AsRawFd for UdpLiteSocket {
    fn as_raw_fd(&self) -> RawFd {
        self.as_udp.as_raw_fd()
    }
}
impl IntoRawFd for UdpLiteSocket {
    fn into_raw_fd(self) -> RawFd {
        self.as_udp.into_raw_fd()
    }
}

impl Deref for UdpLiteSocket {
    type Target = UdpSocket;
    fn deref(&self) -> &UdpSocket {
        &self.as_udp
    }
}

fn rust_addr_to_sockaddr(addr: &SocketAddr,  storage: &mut sockaddr_storage)
-> socklen_t {
    match addr {
        SocketAddr::V4(addrv4) => {
            storage.ss_family = AF_INET as sa_family_t;
            // shadow to avoid aliasing
            let storage = unsafe {
                &mut*{storage as *mut sockaddr_storage as *mut sockaddr_in}
            };
            storage.sin_addr.s_addr = u32::from(*addrv4.ip()).to_be();
            storage.sin_port = addrv4.port().to_be();
            mem::size_of::<sockaddr_in>() as socklen_t
        }
        SocketAddr::V6(addrv6) => {
            storage.ss_family = AF_INET6 as sa_family_t;
            let storage = unsafe {
                &mut*{storage as *mut sockaddr_storage as *mut sockaddr_in6}
            };
            storage.sin6_port = addrv6.port().to_be();
            storage.sin6_flowinfo = addrv6.flowinfo();
            storage.sin6_addr.s6_addr = addrv6.ip().octets();
            storage.sin6_scope_id = addrv6.scope_id();
            mem::size_of::<sockaddr_in6>() as socklen_t
        }
    }
}

fn try_bind(addr: &SocketAddr) -> Result<UdpLiteSocket, io::Error> {
    // safe because it doesn't store any fancy Rust types
    let mut storage = unsafe { mem::zeroed::<sockaddr_storage>() };
    let addr_len = rust_addr_to_sockaddr(&addr, &mut storage);
    let addr_type = storage.ss_family as c_int;
    let sock = unsafe {
        match socket(addr_type, SOCK_DGRAM | SOCK_CLOEXEC, IPPROTO_UDPLITE) {
            -1 => return Err(io::Error::last_os_error()),
            fd => UdpLiteSocket::from_raw_fd(fd),
        }
    };
    unsafe {
        let general_ptr = &storage as *const sockaddr_storage as *const sockaddr;
        loop {
            if bind(sock.as_raw_fd(), general_ptr, addr_len) == -1 {
                let error = io::Error::last_os_error();
                if error.kind() != Interrupted {
                    break Err(error);
                }
            } else {
                break Ok(sock);
            }
        }
    }
}

impl UdpLiteSocket {
    /// Create an UDP-Lite socket bound to an address and port.
    pub fn bind<A: ToSocketAddrs>(addrs: A) -> Result<Self, io::Error> {
        let addrs = match addrs.to_socket_addrs() {
            Ok(iterator) => iterator,
            Err(error) => return Err(error)
        };
        let mut error = io::Error::new(InvalidInput, "could not resolve to any addresses");
        for addr in addrs {
            match try_bind(&addr) {
                Err(e) => error = e,
                ok => return ok,
            }
        }
        Err(error)
    }

    pub fn try_clone(&self) -> Result<Self, io::Error> {
        match self.as_udp.try_clone() {
            Ok(clone) => Ok(UdpLiteSocket { as_udp: clone }),
            Err(e) => Err(e),
        }
    }

    // send_cscov(,) -> Result<u16>
    // set_send_cscov(, u16) -> Result<()>
    // send(, &[u8], SocketAddr) -
    // send_vectored
    // send_to()
    // send_many
    // 
    /// Change how many bytes of the payload of sent datagrams are covered by checksum.
    ///
    /// `None` means the entire datagram is covered, and this is the default
    /// for newly created sockets.
    ///
    /// # Errors
    ///
    /// This will fail if the file descriptor for some reason is not a UDP-Lite
    /// socket, which should not happen in bug-free programs.
    pub fn set_send_checksum_coverage(&self,  coverage: Option<u16>)
    -> Result<(), io::Error> {
        let coverage: c_int = match coverage {
            Some(payload) => payload as c_int + 8,
            None => 0,
        };
        let ret = unsafe {
            setsockopt(
                self.as_raw_fd(),
                IPPROTO_UDPLITE,
                UDPLITE_SEND_CSCOV,
                &coverage as *const c_int as *const c_void,
                mem::size_of::<c_int>() as socklen_t,
            )
        };
        match ret {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }

    /// Get how many bytes of the payload of sent datagrams are covered by checksum.
    ///
    /// `None` means the entire datagram is covered, and this is the default
    /// for newly created sockets.
    pub fn send_checksum_coverage(&self)
    -> Result<Option<u16>, io::Error> {
        let mut coverage: c_int = -1;
        let ret = unsafe {
            let mut len = mem::size_of::<c_int>() as socklen_t;
            getsockopt(
                self.as_raw_fd(),
                IPPROTO_UDPLITE,
                UDPLITE_SEND_CSCOV,
                &mut coverage as *mut c_int as *mut c_void,
                &mut len as *mut socklen_t,
            )
        };
        match (ret, coverage) {
            (0, 0) => Ok(None),
            (0, 8..=0xffff) => Ok(Some(coverage as u16 - 8)),
            (0, 1..=7) => Err(io::Error::new(InvalidData, "Returned coverage only partially covers header")),
            (0, _) => Err(io::Error::new(InvalidData, "Returned coverage is outside of valid range (for IPv6)")),
            (-1, _) => Err(io::Error::last_os_error()),
            (_, _) => Err(io::Error::new(InvalidData, "Unexpected return value from getsockopt()")),
        }
    }

    // send(, &[u8], SocketAddr) -
    // send_vectored
    // send_to()
    // send_many
    // 
    /// Set the required checksum coverage of payloads of received datagrams.
    ///
    /// Received datagrams with lesser coverage will be discarded by the OS.
    //
    // FIXME what does `None` mean here?
    pub fn set_recv_checksum_coverage_filter(&self,  coverage: Option<u16>)
    -> Result<(), io::Error> {
        let coverage: c_int = match coverage {
            Some(payload) => payload as c_int + 8,
            None => 0,
        };
        let ret = unsafe {
            setsockopt(
                self.as_raw_fd(),
                IPPROTO_UDPLITE,
                UDPLITE_RECV_CSCOV,
                &coverage as *const c_int as *const c_void,
                mem::size_of::<c_int>() as socklen_t,
            )
        };
        match ret {
            -1 => Err(io::Error::last_os_error()),
            _ => Ok(()),
        }
    }

    /// Set the required checksum coverage of payloads of received datagrams.
    ///
    /// Received datagrams with lesser coverage will be discarded by the OS.
    //
    // FIXME what does `None` mean here?
    pub fn recv_checksum_coverage_filter(&self)
    -> Result<Option<u16>, io::Error> {
        let mut coverage: c_int = -1;
        let ret = unsafe {
            let mut len = mem::size_of::<c_int>() as socklen_t;
            getsockopt(
                self.as_raw_fd(),
                IPPROTO_UDPLITE,
                UDPLITE_RECV_CSCOV,
                &mut coverage as *mut c_int as *mut c_void,
                &mut len as *mut socklen_t,
            )
        };
        match (ret, coverage) {
            (0, 0) => Ok(None),
            (0, 8..=0xffff) => Ok(Some(coverage as u16 - 8)),
            (0, _) => Err(io::Error::new(InvalidData, "Returned coverage is outside of valid range")),
            (-1, _) => Err(io::Error::last_os_error()),
            (_, _) => Err(io::Error::new(InvalidData, "Unexpected return value from getsockopt()")),
        }
    }
}
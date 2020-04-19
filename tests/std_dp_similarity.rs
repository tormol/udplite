extern crate udplite;

use std::net::{UdpSocket, SocketAddr, ToSocketAddrs, Ipv4Addr, Ipv6Addr};
use std::io;
use std::iter;
use std::os::unix::io::{FromRawFd, IntoRawFd};
use udplite::UdpLiteSocket;

#[test]
fn debug_implementation_unbound() {
    let udplite_socket = unsafe { UdpLiteSocket::from_raw_fd(-1) };
    let std_udp_socket = unsafe { UdpSocket::from_raw_fd(-1) };
    let udplite_dbg = format!("{:?}", udplite_socket);
    let std_udp_dbg = format!("{:?}", std_udp_socket);
    assert_eq!(&udplite_dbg[7..], &std_udp_dbg[3..]);
    let udplite_alt_dbg = format!("{:#?}", udplite_socket);
    let std_udp_alt_dbg = format!("{:#?}", std_udp_socket);
    assert_eq!(&udplite_alt_dbg[7..], &std_udp_alt_dbg[3..]);
}

#[test]
fn debug_implementation_ipv4_localhost() {
    let std_udp_socket = UdpSocket::bind((Ipv4Addr::new(127,0,0,1), 0))
        .expect("bind UDP to localhost");
    let std_udp_dbg = format!("{:?}", std_udp_socket);
    let std_udp_alt_dbg = format!("{:#?}", std_udp_socket);
    let udplite_socket = unsafe {
        UdpLiteSocket::from_raw_fd(std_udp_socket.into_raw_fd())
    };
    let udplite_dbg = format!("{:?}", udplite_socket);
    let udplite_alt_dbg = format!("{:#?}", udplite_socket);
    assert_eq!(&udplite_dbg[7..], &std_udp_dbg[3..]);
    assert_eq!(&udplite_alt_dbg[7..], &std_udp_alt_dbg[3..]);
}

#[test]
fn debug_implementation_ipv6_any() {
    let std_udp_socket = UdpSocket::bind((Ipv6Addr::from([0; 16]), 0))
        .expect("bind UDP to localhost");
    let std_udp_dbg = format!("{:?}", std_udp_socket);
    let std_udp_alt_dbg = format!("{:#?}", std_udp_socket);
    let udplite_socket = unsafe {
        UdpLiteSocket::from_raw_fd(std_udp_socket.into_raw_fd())
    };
    let udplite_dbg = format!("{:?}", udplite_socket);
    let udplite_alt_dbg = format!("{:#?}", udplite_socket);
    assert_eq!(&udplite_dbg[7..], &std_udp_dbg[3..]);
    assert_eq!(&udplite_alt_dbg[7..], &std_udp_alt_dbg[3..]);
}

#[test]
fn zero_addrs_error() {
    struct NoAddrs;
    impl ToSocketAddrs for NoAddrs {
        type Iter = iter::Empty<SocketAddr>;
        fn to_socket_addrs(&self) -> Result<Self::Iter, io::Error> {
            Ok(iter::empty())
        }
    }
    let udplite_err = UdpLiteSocket::bind(NoAddrs).expect_err("no addrs provided");
    let std_udp_err = UdpSocket::bind(NoAddrs).expect_err("no addrs provided");
    assert_eq!(format!("{:?}", udplite_err), format!("{:?}", std_udp_err));
}

#[test]
fn invalid_to_addr_error() {
    let udplite_err = UdpLiteSocket::bind("").expect_err("empty str");
    let std_udp_err = UdpSocket::bind("").expect_err("empty str");
    assert_eq!(format!("{:?}", udplite_err), format!("{:?}", std_udp_err));
}

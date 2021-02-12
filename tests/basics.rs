extern crate udplite;
extern crate libc;

use std::net::{Ipv4Addr, Ipv6Addr};
use std::io::ErrorKind;
use udplite::UdpLiteSocket;

#[test]
fn create_ipv4_socket() {
    UdpLiteSocket::bind((Ipv4Addr::new(0, 0, 0, 0), 0))
        .expect("create IPv4 UDP-Lite socket (bind to 0.0.0.0:0)");
}

#[test]
fn create_ipv6_socket() {
    UdpLiteSocket::bind((Ipv6Addr::from([0; 16]), 0))
        .expect("create IPv6 UDP-Lite socket (bind to [::]:0)");
}

#[test]
fn create_nonblocking_socket() {
    let socket = UdpLiteSocket::bind_nonblocking((Ipv4Addr::new(0, 0, 0, 0), 0))
        .expect("create nonblocking IPv4 UDP-Lite socket (bind to 0.0.0.0:0)");
    assert_eq!(
        socket.recv_from(&mut[0; 10])
            .expect_err("fail recv with WouldBlock")
            .kind(),
        ErrorKind::WouldBlock
    );
}

#[test]
fn nonblocking_doesnt_fail_bind() {
    match UdpLiteSocket::bind_nonblocking("example.net:1") {
        Ok(socket) => {
            assert_eq!(
                socket.recv_from(&mut[0; 10])
                    .expect_err("fail recv with WouldBlock")
                    .kind(),
                ErrorKind::WouldBlock
            );
        }
        Err(ref e) if e.raw_os_error() == Some(libc::EINPROGRESS) => {
            panic!("bind_nonblocking() failed with WouldBlock");
        }
        Err(_) => {}
    }
}

#[test]
fn send_connected_ipv6() {
    let a = UdpLiteSocket::bind((Ipv6Addr::LOCALHOST, 0))
        .expect("create UDP-Lite socket bound to [::1]:0");
    let b = UdpLiteSocket::bind((Ipv6Addr::LOCALHOST, 0))
        .expect("create another socket bound to [::1]:0");

    let a_addr = a.local_addr().expect("get local addr of socket a");
    let b_addr = b.local_addr().expect("get local addr of socket b");
    a.connect(b_addr)
        .expect(&format!("connect socket a to addr of socket b ({})", b_addr));
    b.connect(a_addr)
        .expect(&format!("connect socket b to addr of socket a ({})", a_addr));

    let msg = "Hello";
    let sent_bytes = a.send(msg.as_bytes())
        .expect(&format!(
                "Send from socket a ({:?}) to addr of socket b ({})",
                a, b_addr
        ));
    assert_eq!(sent_bytes, msg.len());
    let mut buf = [0u8; 20];
    let received_bytes = b.recv(&mut buf)
        .expect(&format!(
                "Receive from socket b ({:?}) connected to addr of socket a ({})",
                b, a_addr
        ));
    assert_eq!(&buf[..received_bytes], msg.as_bytes());
}

#[test]
fn set_get_recv_cscov() {
    let socket = UdpLiteSocket::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("create IPv4 UDP-Lite socket (bind to 127.0.0.1:0)");
    socket.set_recv_checksum_coverage_filter(Some(100))
        .expect("Set receive cscov filter to largeish");
    assert_eq!(socket.recv_checksum_coverage_filter().expect("get receive cscov"), Some(100));
    socket.set_recv_checksum_coverage_filter(Some(0))
        .expect("Set receive cscov filter to minimum");
    assert_eq!(socket.recv_checksum_coverage_filter().expect("get receive cscov"), Some(0));
    socket.set_recv_checksum_coverage_filter(None)
        .expect("Set receive cscov filter to full datagram");
    assert_eq!(socket.recv_checksum_coverage_filter().expect("get receive cscov"), None);
    socket.set_recv_checksum_coverage_filter(Some(!0-8))
        .expect("Set receive cscov filter to max representable");
    assert_eq!(socket.recv_checksum_coverage_filter().expect("get receive cscov"), Some(!0-8));
}

#[test]
fn set_get_send_cscov() {
    let socket = UdpLiteSocket::bind((Ipv4Addr::LOCALHOST, 0))
        .expect("create IPv4 UDP-Lite socket (bind to 127.0.0.1:0)");
    socket.set_send_checksum_coverage(Some(100)).expect("Set send cscov to largeish");
    assert_eq!(socket.send_checksum_coverage().expect("get send cscov"), Some(100));
    socket.set_send_checksum_coverage(Some(0)).expect("Set send cscov to minimum");
    assert_eq!(socket.send_checksum_coverage().expect("get send cscov"), Some(0));
    socket.set_send_checksum_coverage(None).expect("Set send cscov to full datagram");
    assert_eq!(socket.send_checksum_coverage().expect("get send cscov"), None);
    socket.set_send_checksum_coverage(Some(!0-8)).expect("Set send cscov to max representable");
    assert_eq!(socket.send_checksum_coverage().expect("get send cscov"), Some(!0-8));
}

#[test]
fn try_clone_returns_udplite() {
    let socket = UdpLiteSocket::bind((Ipv4Addr::new(127, 0, 0, 1), 0))
        .expect("create UDP-Lite socket");
    let clone = socket.try_clone().expect("duplicate UDP-Lite socket");
    clone.set_send_checksum_coverage(Some(100))
        .expect("change checksum coverage of cloned UDP-Lite socket");
}

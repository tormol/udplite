extern crate udplite;

use udplite::UdpLiteSocket;

fn main() {
    let any = UdpLiteSocket::bind("0.0.0.0:0").expect("create UDP-Lite socket");

    println!("addr of randomly bound socket {:?}: {:?}", any, any.local_addr());
    println!(
        "default checksum coverage: send={:?}, recv filter={:?}",
        any.send_checksum_coverage(),
        any.recv_checksum_coverage_filter(),
    );

    any.set_send_checksum_coverage(Some(0)).expect("set send cscov to the minimum");
    any.set_recv_checksum_coverage_filter(Some(0)).expect("disable recv cscov filter");

    any.send_to(b"Hello, UDP-Lite", any.local_addr().unwrap()).expect("send datagram");
    let mut buf = [0; 20];
    let len = any.recv(&mut buf).expect("receive datagram");
    println!("received {}", String::from_utf8_lossy(&buf[..len]));
}

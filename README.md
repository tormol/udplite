# udplite

A Rust library for using [UDP-Lite sockets](http://man7.org/linux/man-pages/man7/udplite.7.html). ([RFC 3828](https://tools.ietf.org/html/rfc3828))

[![crates.io](https://img.shields.io/crates/v/udplite.svg)](https://crates.io/crates/udplite) [![Build Status](https://api.cirrus-ci.com/github/tormol/udplite.svg)](https://cirrus-ci.com/github/tormol/udplite) ![License](https://img.shields.io/crates/l/udplite.svg) [![Documentation](https://docs.rs/udplite/badge.svg)](https://docs.rs/udplite/)

```rust
extern crate udplite;

let socket = udplite::UdpLiteSocket::bind("[::]:0").expect("Create UDP-Lite socket");
socket.set_send_checksum_coverage(Some(0)).expect("disable checksum coverage for payload");
socket.connect("[::1]:7").expect("set destination");
socket.send(b"Hello UDP-Lite").expect("send datagram");
```

This crate is a work in progress.
It's not been released on crates.io yet as it depends on constants not yet available in libc.

## Supported operating systems

UDP-Lite is only implemented by Linux and FreeBSD.
Whether Android supports it I'm not sure about: The kernel has [the constants](https://android.googlesource.com/kernel/common/+/refs/heads/android-mainline/include/net/udplite.h), but some of them are missing from bionic (the Android libc). ([only `IPPROTO_UDPLITE` is present](https://android.googlesource.com/platform/bionic.git/+/refs/heads/master/libc/kernel/uapi/linux/in.h))

The FreeBSD implementation also behaves strangely: sent packets that are not entirely covered completely by the checksum (`UDPLITE_SEND_CSCOV`) seems to be discarded by the OS. (meanwhile such packets sent from Linux are received)

## mio integration

Like UDP sockets, UDP-Lite sockets can be registered with epoll / kqueue, and therefore used with [mio](https://github.com/tokio-rs/mio).
This feature is not enabled by default; enable it in Cargo.toml with:

```toml
[dependencies]
udplite = {version="0.0.0", features=["mio_07"]}
```

Also remember to enable nonblocking mode for the sockets. (`UdpLiteSocket.set_nonblocking(true)`)

## Minimum supported Rust version

The minimum supported Rust version is 1.36.
Older versions might currently work, but I plan to use `std::io::IoSlice`.

## License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

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

UDP-Lite is only implemented by Linux, Android and FreeBSD.

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

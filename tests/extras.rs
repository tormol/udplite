extern crate udplite;

use udplite::UdpLiteSocket;

// Only checks that the flag is set, doesn't actually test whether the fd is closed or not

#[test]
fn new_has_cloexec() {
    let socket = UdpLiteSocket::bind("127.0.0.1:0").expect("Create IPv4 UDP-Lite socket");
    assert!(socket.is_cloexec().expect("get close-on-exec"), "new sockets have close-on-exec set");
}

#[test]
fn set_cloexec() {
    let socket = UdpLiteSocket::bind("127.0.0.1:0").expect("Create IPv4 UDP-Lite socket");
    socket.set_cloexec(true).expect("set close-on-exec");
    assert!(
        socket.is_cloexec().expect("get close-on-exec"),
        "setting close-on-exec is idempotent"
    );
    socket.set_cloexec(false).expect("disable close-on-exec");
    assert!(!(socket.is_cloexec().expect("get close-on-exec")), "cloaring close-on-exec works");
    socket.set_cloexec(true).expect("enable close-on-exec");
    assert!(socket.is_cloexec().expect("get close-on-exec"), "re-enabling close-on-exec works");
}

#[test]
fn cloned_has_cloexec() {
    let socket = UdpLiteSocket::bind("127.0.0.1:0").expect("Create IPv4 UDP-Lite socket");

    let clone = socket.try_clone().expect("clone socket");
    assert!(
        clone.is_cloexec().expect("get close-on-exec"),
        "cloned socket have close-on-exec set"
    );

    socket.set_cloexec(false).expect("disable close-on-exec");
    let clone = socket.try_clone().expect("clone socket");
    assert!(
        clone.is_cloexec().expect("get close-on-exec"),
        "cloned sockets have close-on-exec set even if disabled on the original"
    );
}

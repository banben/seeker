use smoltcp::wire::{IpAddress, IpCidr};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::process::Command;
use tracing::info;

pub struct DNSSetup {
    original: Vec<u8>,
}

impl DNSSetup {
    pub fn new() -> Self {
        info!("setup dns");
        let mut resolv = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/etc/resolv.conf")
            .unwrap();
        let mut buf = vec![];
        let _ = resolv.read_to_end(&mut buf).unwrap();

        info!(
            "original resolve.conf: {}",
            std::str::from_utf8(&buf).unwrap()
        );
        resolv.set_len(0).unwrap();
        resolv.seek(SeekFrom::Start(0)).unwrap();
        resolv.write_all(b"nameserver 127.0.0.1").unwrap();

        DNSSetup { original: buf }
    }
}

impl Drop for DNSSetup {
    fn drop(&mut self) {
        info!("clear dns");
        let mut resolv = OpenOptions::new()
            .write(true)
            .open("/etc/resolv.conf")
            .unwrap();
        resolv.write_all(&self.original).unwrap();
    }
}

pub fn setup_ip(tun_name: &str, ip: IpAddress, cidr: IpCidr) {
    let ip_s = ip.to_string();
    let output = Command::new("ip")
        .args(&["addr", "replace", &ip_s, "dev", tun_name])
        .output()
        .expect("run ip addr");
    if !output.status.success() {
        panic!(
            "stdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).expect("utf8"),
            std::str::from_utf8(&output.stderr).expect("utf8")
        );
    }
    let output = Command::new("ip")
        .args(&["link", "set", tun_name, "up"])
        .output()
        .expect("run ip addr");
    if !output.status.success() {
        panic!(
            "stdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).expect("utf8"),
            std::str::from_utf8(&output.stderr).expect("utf8")
        );
    }
    let output = Command::new("ip")
        .args(&["route", "add", cidr.to_string().as_str(), "dev", tun_name])
        .output()
        .expect("add route");
    if !output.status.success() {
        panic!(
            "stdout: {}\nstderr: {}",
            std::str::from_utf8(&output.stdout).expect("utf8"),
            std::str::from_utf8(&output.stderr).expect("utf8")
        );
    }
}

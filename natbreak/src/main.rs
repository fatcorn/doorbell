mod nat_type_sniffer;

use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:6666").unwrap();

    let (tx, rx): (Sender<(Vec<u8>, SocketAddr)>, Receiver<(Vec<u8>, SocketAddr)>) = mpsc::channel();
    let _ =  thread::spawn(move || {
        println!("enter thread");
        let socket = UdpSocket::bind("0.0.0.0:7777").unwrap();
        loop {
            let (buf, addr) = rx.recv().unwrap();
            socket.send_to(buf.as_slice(), addr);
        }
    }
    );
    loop {
        let mut buf = [0u8; 1500];
        let (rsz, src) = socket.recv_from(&mut buf).unwrap();
        let buf = &mut buf[..rsz];
        println!("get {} address from {:?}", String::from_utf8(buf.to_vec()).unwrap(), src);
        buf.reverse();
        tx.send((buf.to_vec(), src));
    }
}

mod tests {
    use std::net::{Ipv4Addr, SocketAddrV4, UdpSocket};
    use std::sync::Arc;
    use std::thread;
    use std::thread::{sleep, spawn};
    use std::time::Duration;

    #[test]
    fn test_send_and_receive_by_udp() {
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:8888").unwrap());
        let target_addr = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);

        let socket_copy = socket.clone();
        let r_thread =  thread::spawn(move || {
            println!("enter thread");
            loop {

                let mut buf = [0u8; 1500];
                let (rsz, src) = socket_copy.recv_from(&mut buf).unwrap();
                let buf = &mut buf[..rsz];
                println!("get {} address from {:?}", String::from_utf8(buf.to_vec()).unwrap(), src);
            }
        }
        );
        // r_thread.join();

        loop {
            socket.send_to("hello".as_bytes(), target_addr);
            sleep(Duration::from_secs(1))
        }
    }
}

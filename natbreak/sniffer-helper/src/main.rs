use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crossbeam_channel::{tick, unbounded};
use borsh::{BorshSerialize, BorshDeserialize};
use lazy_static::lazy_static;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
struct Seeker {
    addr: SocketAddr
}
const PONG: &[u8] = "PONG".as_bytes();

lazy_static!{
    pub static ref ASSISTANT: SocketAddr = SocketAddr::new("127.0.0.1".parse().unwrap(), 9527);
}


fn main() {
    let socket = Arc::new(UdpSocket::bind("0.0.0.0:6666").unwrap());

    // 端口转发通道
    let (port_transmit_tx, port_transmit_rx) = unbounded();

    // self转发通道
    let (self_response_tx, self_response_rx) = unbounded();

    // 本机其他接口转发线程
    let _ =  thread::spawn(move || {
        println!("enter port transmit thread");
        let socket = UdpSocket::bind("0.0.0.0:7777").unwrap();
        loop {
            let addr = port_transmit_rx.recv().unwrap();
            socket.send_to(PONG, addr);
        }
    });


    // 协助转发线程
    thread::spawn(|| {
        let _ =  thread::spawn(move || {
            println!("enter server transmit thread");
            let socket = UdpSocket::bind("0.0.0.0:9527").unwrap();
            loop {
                let mut buf = [0u8; 1500];
                let (rsz, src) = socket.recv_from(&mut buf).unwrap();
                let mut buf = &buf[..rsz];
                let seeker : Result<Seeker, _> = Seeker::deserialize(&mut buf);
                if seeker.is_err() {
                    continue
                }

                socket.send_to(PONG, seeker.unwrap().addr);
            }
        });
    });

    let self_socket = socket.clone();
    // self response线程
    thread::spawn(|| {
        let _ =  thread::spawn(move || {
            println!("enter self response thread");
            let ticker = tick(Duration::from_secs(15));
            loop {
                ticker.recv();
                let chan_len = self_response_rx.len();
                for _ in  0..chan_len {
                    let addr = self_response_rx.recv().unwrap();
                    self_socket.send_to(PONG, addr);
                }
            }
        });
    });

    loop {
        let mut buf = [0u8; 1500];
        let (rsz, src) = socket.recv_from(&mut buf).unwrap();
        let buf = &mut buf[..rsz];
        println!("get {} address from {:?}", String::from_utf8(buf.to_vec()).unwrap(), src);
        let data = String::from_utf8(buf.to_vec()).unwrap();
        if data != "ping" {
            continue
        }
        // 本机不同端口转发
        port_transmit_tx.send( src);

        // 助手转发转发
        let seeker = Seeker { addr: src.clone() };
        let seeker_data = borsh::to_vec(&seeker).unwrap();
        socket.send_to(seeker_data.as_slice(), *ASSISTANT);

        // 后续自主发起响应
        self_response_tx.send(src.clone());
    }
}

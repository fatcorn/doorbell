mod config;

use std::net::{SocketAddr, UdpSocket};
use std::net::SocketAddr::V4;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crossbeam_channel::{unbounded};
use borsh::{BorshSerialize, BorshDeserialize};
use lazy_static::lazy_static;
use sniffer::nat_type_sniffer::{Request, Response};
use config::{Config, CONFIG_FILE};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
struct Seeker {
    addr: SocketAddr
}


lazy_static!{

    static ref assistant_ip: String = {
        let config = Config::load((*CONFIG_FILE).clone().unwrap());
        config.unwrap().assistant_ip
    };

    pub static ref ASSISTANT: SocketAddr = SocketAddr::new(assistant_ip.parse().unwrap(), 9527);
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
            let rep = Response::Pong;
            let rep_data = borsh::to_vec(&rep).unwrap();
            socket.send_to(rep_data.as_slice(), addr);
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
                let rep = Response::Pong;
                let rep_data = borsh::to_vec(&rep).unwrap();
                socket.send_to(rep_data.as_slice(), seeker.unwrap().addr);
            }
        });
    });

    let self_socket = socket.clone();
    // self response线程
    thread::spawn(|| {
        let _ =  thread::spawn(move || {
            println!("enter self response thread");
            loop {
                let ret: (Response, SocketAddr) = self_response_rx.recv().unwrap();
                let (response, addr) = ret;
                let rep_data = borsh::to_vec(&response).unwrap();
                self_socket.send_to(rep_data.as_slice(), addr);
            }
        });
    });

    loop {
        let mut buf = [0u8; 1500];
        let (rsz, src) = socket.recv_from(&mut buf).unwrap();
        let buf = &mut buf[..rsz];
        println!("get {:?} address from {:?}", buf, src);
        let req: Result<Request, _> = Request::deserialize(&mut &buf[..rsz]);
        if req.is_err(){
            continue
        }
        match req.unwrap() {
            Request::Ping =>  {
                // 本机不同端口转发
                port_transmit_tx.send( src);

                // 助手转发转发
                let seeker = Seeker { addr: src.clone() };
                let seeker_data = borsh::to_vec(&seeker).unwrap();
                socket.send_to(seeker_data.as_slice(), *ASSISTANT);

                // 后续自主发起响应
                let rep = Response::Pong;
                self_response_tx.send((rep, src.clone()));
            }

            Request::AddressCheck => {
                let rep;
                if let SocketAddr::V4(addr) = src {
                    rep = Response::CheckPong {addr};
                } else { panic!() }
                self_response_tx.send((rep, src.clone()));
            }
        }
    }
}

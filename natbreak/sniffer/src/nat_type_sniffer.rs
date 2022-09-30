use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::{future, thread};
use std::time::Duration;
use crossbeam_channel::{bounded, select, tick};
use borsh::{BorshDeserialize, BorshSerialize};

const TIMEOUT: u64 = 35;
const SOCKET_TIMEOUT: u64 = 30;

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum NatType{
    // 完全锥形
    FullCone,
    // 受限锥形
    RestrictedCone,
    // 端口受限锥形
    PortRestrictedCone,
    // 对称型
    Symmetric,
    // IPV6
    IPV6,
    // 未知类型
    Unknown,
}

pub fn sniffer() -> NatType {
    let ticker = tick(Duration::from_secs(TIMEOUT));
    let (tx, rx) = bounded(1);
    let (out_sig_tx, out_sig_rx) = bounded(1);

    let thread_judge = thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:8888").unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(SOCKET_TIMEOUT)));
        let target_addr = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);

        let data = "ping".as_bytes();
        socket.send_to(data, target_addr);

        let  mut valid_addr :SocketAddr;
        loop {
            select! {
                recv(out_sig_rx) -> sig => {
                    return
                }

                default => {
                    let (rsz, src) = socket.recv_from(&buf).unwrap();
                    if rsz == 0 {
                        return
                    }
                    let mut buf = [0u8; 1500];
                    let rev_data = String::from_utf8_lossy(&buf[..rsz]).into_string();
                    if rev_data == "pong" {
                        valid_addr = src;
                        break
                    }
                }
            }
        }

        match valid_addr {
            SocketAddr::V4(addr_v4)=> {
                if *addr_v4.ip().clone() == Ipv4Addr::new(141, 164, 51, 24) {
                    if addr_v4.port() == 6666 {
                        // todo,再发一条消息至server2，，它将返回是否是对称型
                        let data = "ping".as_bytes();
                        socket.send_to(data, target_addr);
                        socket.set_read_timeout(Some(Duration::from_secs(TIMEOUT - SOCKET_TIMEOUT - 1)));
                        let mut buf = [0u8; 1500];
                        let (rsz, src) = socket.recv_from(&buf).unwrap();
                        if rsz == 0 {
                            return
                        }
                        let nat_type : NatType =  NatType::deserialize(&mut buf[..rsz]).unwrap();
                        rx.send(nat_type)
                        // rx.send(NatType::PortRestrictedCone)
                    }
                    rx.send(NatType::RestrictedCone)
                } else {
                    rx.send(NatType::FullCone)
                }
            }
            _ => {
            }
        }
    });


    select! {
        recv(ticker) -> msg =>{
            out_sig_tx.send(1);
            return NatType::Unknown
        }
        recv(rx) -> nat_type =>{
            return nat_type
        }
    }

}
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::{thread};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use borsh::{BorshDeserialize, BorshSerialize};

// const TIMEOUT: u64 = 5;
const SOCKET_TIMEOUT: u64 = 2;

#[derive(Clone, Copy,Debug, PartialEq, BorshDeserialize, BorshSerialize)]
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

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Request{
    // 锥形检测
    Ping,
    // 对称形检测(地址检测)
    AddressCheck,
}

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize, Copy)]
pub enum Response{
    // 锥形检测响应
    Pong,
    // 对称形检测
    CheckPong{ addr: SocketAddrV4},
}



pub fn sniffer() -> NatType {
    // let ticker = tick(Duration::from_secs(TIMEOUT));
    // let (tx, rx) = bounded(1);
    // let (out_sig_tx, out_sig_rx) = bounded(1);

    let nat_type = Arc::new(Mutex::new(NatType::Unknown));
    // let nat_type = Arc::new(RwLock::new(NatType::Unknown));

    let nat_type_clone = Arc::clone(&nat_type);
    // 对称形判断线程
    let thread_symmetric_judge = thread::spawn(move || {

        let socket = UdpSocket::bind("0.0.0.0:6666").unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(SOCKET_TIMEOUT)));
        let target_addr1 = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);
        let target_addr2 = SocketAddrV4::new(Ipv4Addr::new(158, 247, 233, 22), 6666);

        let check_req = Request::AddressCheck;
        let data = borsh::to_vec( &check_req).unwrap();
        socket.send_to(data.as_slice(), target_addr1);
        socket.send_to(data.as_slice(), target_addr2);

        let mut responses = vec![];

        loop {
            let mut buf = [0u8; 1500];
            let (rsz, src) = socket.recv_from(&mut buf).unwrap();
            if src.eq(&SocketAddr::from(target_addr1)) || src.eq(&SocketAddr::from(target_addr2)) {
                let response: Response = Response::deserialize(&mut &buf[..rsz]).unwrap();
                println!("get CheckPong resp");
                if let Response::CheckPong{addr: _ } = response  {
                    responses.push(response);
                }

                if responses.len() == 2 {
                    let response1 = responses.get(1).unwrap();
                    let response2 = responses.get(2).unwrap();
                    let addr1;
                    if let Response::CheckPong { addr} = response1 {
                        addr1 = addr;
                    } else { panic!() }
                    let addr2;
                    if let Response::CheckPong { addr} = response2 {
                        addr2 = addr;
                    } else { panic!() }
                    if addr1.eq(addr2) {
                        let mut nat = nat_type_clone.lock().unwrap();
                        if nat.eq(&NatType::Unknown) || nat.eq(&NatType::RestrictedCone) {
                            *nat = NatType::Symmetric;
                            return
                        }
                    } else {
                        return
                    }
                }
            }
        }
    });

    let nat_type_clone = Arc::clone(&nat_type);
    // 锥形判断线程
    let thread_cone_judge = thread::spawn(move || {
        let socket = UdpSocket::bind("0.0.0.0:8888").unwrap();
        socket.set_read_timeout(Some(Duration::from_secs(SOCKET_TIMEOUT)));
        let target_addr = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);

        let ping_req = Request::Ping;
        let data = borsh::to_vec(&ping_req).unwrap();
        socket.send_to(data.as_slice(), target_addr);

        let src_addr = Ipv4Addr::new(141, 164, 51, 24);
        let src_port = 6666;
        loop {
            let mut buf = [0u8; 1500];
            let (rsz, src) = socket.recv_from(&mut buf).unwrap();

            let response: Result<Response, _> = Response::deserialize(&mut &buf[..rsz]);
            if response.is_err()  {
                continue
            }
            if Response::Pong != response.unwrap() {
                continue
            }
            match src {
                SocketAddr::V4(addr_v4)=> {
                    let mut nat_type_write = nat_type_clone.lock().unwrap();
                    if addr_v4.ip().eq(&src_addr) {
                        if addr_v4.port() == src_port {
                            if nat_type_write.eq(&NatType::Unknown) {
                                *nat_type_write = NatType::PortRestrictedCone;
                            }
                        } else {
                            *nat_type_write = NatType::RestrictedCone;
                        }
                    } else {
                        //已是完全锥形，直接返回
                        *nat_type_write = NatType::FullCone;
                        return
                    }
                }
                _ => {}
            }
        }
    });
    thread_cone_judge.join();
    thread_symmetric_judge.join();
    // //超时退出
    // select! {
    //     recv(ticker) -> msg =>{
    //         return  *nat_type.read().unwrap()
    //     }
    // }
    println!("Result: {:?}",  *nat_type.lock().unwrap());
    let ret = *nat_type.lock().unwrap();
    ret
}

#[cfg(test)]
mod tests {
    use crate::nat_type_sniffer::sniffer;

    #[test]
    fn test_sniffer() {
        let my_nat_type = sniffer();
        println!("my_nat_type {:?}", my_nat_type)
    }
}
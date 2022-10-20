use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use crossbeam_channel::{bounded, Receiver, Sender};
use sniffer::nat_type_sniffer::{NatType, Request, Response};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, PartialEq, BorshDeserialize, BorshSerialize)]
enum Message {
    //
    BreakPing,
    //
    BreakPong,
}

// 😢，失败原因，如果nat 路由不支持，Hairpin转换，在同一内网类，无法进行穿透
pub fn breaker() -> bool{
    // todo 暂时供测试使用
    let socket1 = UdpSocket::bind("0.0.0.0:6666").unwrap();
    let socket2 = UdpSocket::bind("0.0.0.0:8888").unwrap();
    let target_addr = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);
    let (awake_sig_tx1, awake_sig_rx1) = bounded(0);
    let (awake_sig_tx2, awake_sig_rx2) = bounded(0);

    let req = Request::AddressCheck;
    let req_data = borsh::to_vec(&req).unwrap();
    socket1.send_to(req_data.as_slice(), target_addr);
    socket2.send_to(req_data.as_slice(), target_addr);

    let rc_socket1 = Arc::new(socket1);
    let rc_socket1_clone1 = Arc::clone(&rc_socket1);
    let rc_socket1_clone2 = Arc::clone(&rc_socket1);

    let rc_socket2 = Arc::new(socket2);
    let rc_socket2_clone1 = Arc::clone(&rc_socket2);
    let rc_socket2_clone2 = Arc::clone(&rc_socket2);

    let address_transmit_task = |socket: Arc<UdpSocket>, sender: Sender<SocketAddrV4>| {
        loop {
            let mut buf = [0u8; 1500];
            let rec_size = socket.recv(&mut buf).unwrap();
            let response :Response = Response::deserialize(&mut &buf[..rec_size]).unwrap();
            if let Response::CheckPong {addr} = response {
                println!("local addr {} -> addr {}", socket.local_addr().unwrap(), addr);
                sender.send(addr);
                break
            }
        }
    };

    let check_task1 = thread::spawn(move || {
        //发送至cli2
        address_transmit_task(rc_socket1_clone1, awake_sig_tx2)
    });

    let check_task2 = thread::spawn(move || {
        //发送至cli1
        address_transmit_task(rc_socket2_clone1, awake_sig_tx1)
    });

    let break_task = |receiver: Receiver<SocketAddrV4>, socket: Arc<UdpSocket>,| {
        let other_address = receiver.recv().unwrap();

        let break_ping_msg = Message::BreakPing;
        let msg_data = borsh::to_vec(&break_ping_msg).unwrap();
        let mut n = 10;
        let local_addr = socket.local_addr().unwrap();
        while n > 0{
            println!("local addr {} send to {}", local_addr , other_address);
            socket.send_to(msg_data.as_slice(), other_address.clone());
            n -= 1;
            sleep(Duration::from_secs(1));
        }

        loop {
            println!("start rev msg");
            let mut buf = [0u8; 1500];
            let (rec_size, addr) = socket.recv_from(&mut buf).unwrap();
            let msg : Message = Message::deserialize(&mut &buf[..rec_size]).unwrap();
            if Message::BreakPong == msg {
                println!("get msg pong from {}", addr);
                socket.send_to(msg_data.as_slice(), other_address);
                sleep(Duration::from_secs(1));
            }
        }
    };
    let listener_task1 = thread::spawn(move || {
        println!("im lis task1");
        break_task(awake_sig_rx1, rc_socket1_clone2);
    });

    let listener_task2 = thread::spawn(move || {
        println!("im lis task2");
        break_task(awake_sig_rx2, rc_socket2_clone2);
    });

    check_task1.join();
    check_task2.join();
    listener_task1.join();
    listener_task2.join();

    true
}

#[cfg(test)]
mod tests {
    use crate::breaker::breaker;

    #[test]
    fn test_breaker() {
        breaker();
    }
}
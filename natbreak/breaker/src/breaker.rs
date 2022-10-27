use std::fs::File;
use std::io::Read;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::rc::Rc;
use std::str::FromStr;
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
            sleep(Duration::from_millis(1));
        }

        loop {
            println!("start rev msg");
            let mut buf = [0u8; 1500];
            let (rec_size, addr) = socket.recv_from(&mut buf).unwrap();
            let msg : Message = Message::deserialize(&mut &buf[..rec_size]).unwrap();

            match msg {
                Message::BreakPong => {
                    println!("get msg pong from {}", addr);
                    socket.send_to(msg_data.as_slice(), other_address);
                    sleep(Duration::from_secs(1));
                }

                Message::BreakPing => {
                    let break_ping_msg = Message::BreakPong;
                    let msg_data = borsh::to_vec(&break_ping_msg).unwrap();
                    println!("get msg ping from {}", addr);
                    socket.send_to(msg_data.as_slice(), other_address);
                    sleep(Duration::from_secs(1));
                }
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

/// 不同网络中的穿透
pub fn breaker_with_diff_nat() -> bool{
    // todo 暂时供测试使用
    let socket = UdpSocket::bind("0.0.0.0:6666").unwrap();
    let target_addr = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);
    let (break_target_addr_tx, break_target_addr_rx) = bounded(0);

    let req = Request::AddressCheck;
    let req_data = borsh::to_vec(&req).unwrap();
    socket.send_to(req_data.as_slice(), target_addr);

    let arc_socket = Arc::new(socket);
    let arc_socket_clone1 = arc_socket.clone();
    let arc_socket_clone2 = arc_socket.clone();
    let send_task = thread::spawn(move || {
        let target_addr = break_target_addr_rx.recv().unwrap();
        println!("start send break ping to {}", target_addr);
        loop {
            println!("start send break ping to {}", target_addr);
            let break_ping_msg = Message::BreakPing;
            let msg_data = borsh::to_vec(&break_ping_msg).unwrap();
            arc_socket_clone1.send_to(msg_data.as_slice(), target_addr);
            sleep(Duration::from_secs(1))
        }
    });

    let recev_task = thread::spawn(move || {
        loop {
            println!("start rev msg");
            let mut buf = [0u8; 1500];
            let (rec_size, addr) = arc_socket_clone2.recv_from(&mut buf).unwrap();
            let msg : Message = Message::deserialize(&mut &buf[..rec_size]).unwrap();
            match msg {
                Message::BreakPong => {
                    println!("get msg pong from {}", addr);
                }

                Message::BreakPing => {
                    let break_pong_msg = Message::BreakPong;
                    let msg_data = borsh::to_vec(&break_pong_msg).unwrap();
                    println!("get msg ping from {}", addr);
                    arc_socket_clone2.send_to(msg_data.as_slice(), addr);
                    sleep(Duration::from_secs(1));
                }
            }
        }
    });

    let read_task = thread::spawn(move || {
        loop {
            let file = File::open("/root/break_addr");
            if file.is_err() {
                sleep(Duration::from_secs(1));
                continue
            }

            let mut file = file.unwrap();
            let mut addr= String::new();
            let _ = file.read_to_string(&mut addr);
            let addr = addr.replace("\n", "");
            let socket_addr = SocketAddrV4::from_str(&addr).unwrap();
            break_target_addr_tx.send(socket_addr);
            break;
        }
    });


    send_task.join();
    recev_task.join();
    read_task.join();

    true
}

/// 端口轮换穿透，不同网络中的穿透
pub fn break_with_guess() -> bool{
    // todo 暂时供测试使用
    let socket = UdpSocket::bind("0.0.0.0:6666").unwrap();
    let target_addr = SocketAddrV4::new(Ipv4Addr::new(141, 164, 51, 24), 6666);
    let (break_target_addr_tx, break_target_addr_rx) = bounded(0);

    let req = Request::AddressCheck;
    let req_data = borsh::to_vec(&req).unwrap();
    socket.send_to(req_data.as_slice(), target_addr);

    let arc_socket = Arc::new(socket);
    let arc_socket_clone1 = arc_socket.clone();
    let arc_socket_clone2 = arc_socket.clone();
    let send_task = thread::spawn(move || {
        let mut target_addr: SocketAddrV4  = break_target_addr_rx.recv().unwrap();
        println!("start send break ping to {}", target_addr);
        let mut port = 1;
        let mut port_increase_direction = true;
        loop {
            target_addr.set_port(port);
            // println!("start send break ping to {}", target_addr);
            let break_ping_msg = Message::BreakPing;
            let msg_data = borsh::to_vec(&break_ping_msg).unwrap();
            arc_socket_clone1.send_to(msg_data.as_slice(), target_addr.clone());
            sleep(Duration::from_millis(1));
            if port_increase_direction {
                port += 1;
            } else {
                port -= 1;
            }

            if port == 1 {
                port_increase_direction = true;
            }
            if port == 65535 {
                port_increase_direction = false;
            }
        }
    });

    let recev_task = thread::spawn(move || {
        loop {
            println!("start rev msg");
            let mut buf = [0u8; 1500];
            let (rec_size, addr) = arc_socket_clone2.recv_from(&mut buf).unwrap();
            let msg : Message = Message::deserialize(&mut &buf[..rec_size]).unwrap();
            match msg {
                Message::BreakPong => {
                    println!("get msg pong from {}", addr);
                }

                Message::BreakPing => {
                    let break_pong_msg = Message::BreakPong;
                    let msg_data = borsh::to_vec(&break_pong_msg).unwrap();
                    println!("get msg ping from {}", addr);
                    arc_socket_clone2.send_to(msg_data.as_slice(), addr);
                    sleep(Duration::from_secs(1));
                }
            }
        }
    });

    let read_task = thread::spawn(move || {
        loop {
            let file = File::open("/root/break_addr");
            if file.is_err() {
                sleep(Duration::from_secs(1));
                continue
            }

            let mut file = file.unwrap();
            let mut addr= String::new();
            let _ = file.read_to_string(&mut addr);
            let addr = addr.replace("\n", "");
            let socket_addr = SocketAddrV4::from_str(&addr).unwrap();
            break_target_addr_tx.send(socket_addr);
            break;
        }
    });


    send_task.join();
    recev_task.join();
    read_task.join();

    true
}

#[cfg(test)]
mod tests {
    use crate::breaker::{break_with_guess, breaker, breaker_with_diff_nat};

    #[test]
    fn test_breaker() {
        breaker();
    }

    #[test]
    fn test_diff_nat_breaker() {
        breaker_with_diff_nat();
    }

    #[test]
    fn test_break_with_guess() {
        break_with_guess();
    }
}
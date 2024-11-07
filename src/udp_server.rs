
use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};
use std::io;

const SERVER: Token = Token(0);

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    let addr = "127.0.0.1:8000".parse().unwrap();
    let mut socket = UdpSocket::bind(addr)?;

    poll.registry().register(
        &mut socket,
        SERVER,
        Interest::READABLE
    )?;

    let mut buffer = [0; 1024];

    loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (len, addr) = socket.recv_from(&mut buffer)?;
                    socket.send_to(&buffer[..len], addr)?;
                }
                _ => unreachable!(),
            }
        }
    }
}
use mio::*;
use mio::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::time::{Duration, Instant};

const SERVER: Token = Token(0);
const TIMER: Token = Token(1);

struct Connection {
    socket: TcpStream,
    last_active: Instant,
}

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    // 设置TCP服务器
    let addr = "127.0.0.1:8000".parse().unwrap();
    let mut server = TcpListener::bind(addr)?;

    poll.registry().register(
        &mut server,
        SERVER,
        Interest::READABLE
    )?;

    let mut connections = HashMap::new();
    let timeout = Duration::from_secs(60); // 60秒超时

    loop {
        poll.poll(&mut events, Some(Duration::from_secs(1)))?;

        for event in events.iter() {
            match event.token() {
                SERVER => {
                    let (mut socket, addr) = server.accept()?;
                    let token = Token(connections.len() + 2);

                    poll.registry().register(
                        &mut socket,
                        token,
                        Interest::READABLE | Interest::WRITABLE
                    )?;

                    connections.insert(token, Connection {
                        socket,
                        last_active: Instant::now(),
                    });
                }
                token => {
                    if let Some(conn) = connections.get_mut(&token) {
                        if event.is_readable() {
                            let mut buffer = [0; 1024];
                            match conn.socket.read(&mut buffer) {
                                Ok(n) if n > 0 => {
                                    conn.socket.write_all(&buffer[..n])?;
                                    conn.last_active = Instant::now();
                                }
                                _ => {
                                    connections.remove(&token);
                                    continue;
                                }
                            }
                        }
                    }
                }
            }
        }

        // 清理超时连接
        connections.retain(|_, conn| {
            Instant::now().duration_since(conn.last_active) < timeout
        });
    }
}
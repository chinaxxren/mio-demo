use std::io::{self, Read, Write};

use mio::net::TcpListener;
use mio::{Events, Interest, Poll, Token};

const SERVER: Token = Token(0);

fn main() -> io::Result<()> {
    // 创建一个新的 Poll 实例
    let mut poll = Poll::new()?;
    // 创建一个 Events 实例，用于存储发生的事件
    let mut events = Events::with_capacity(1024);

    // 解析服务器地址
    let addr = "127.0.0.1:8000".parse().unwrap();
    // 绑定 TCP 监听器到指定地址
    let mut server = TcpListener::bind(addr)?;

    // 在 Poll 注册表中注册服务器，监听可读事件
    poll.registry()
       .register(&mut server, SERVER, Interest::READABLE)?;

    // 创建一个空的 Vec 来存储连接
    let mut connections = Vec::new();

    // 主循环，持续处理事件
    loop {
        // 等待事件发生
        poll.poll(&mut events, None)?;

        // 遍历所有发生的事件
        for event in events.iter() {
            // 根据事件的 Token 来处理不同的逻辑
            match event.token() {
                // 如果是服务器的 Token，表示有新的连接请求
                SERVER => {
                    // 接受新的连接
                    let (mut connection, _) = server.accept()?;
                    // 为新连接分配一个 Token
                    let token = Token(connections.len() + 1);

                    // 在 Poll 注册表中注册新连接，监听可读事件
                    poll.registry()
                       .register(&mut connection, token, Interest::READABLE)?;

                    // 将新连接添加到 connections Vec 中
                    connections.push(connection);
                }
                // 如果是其他 Token，表示对应的连接有数据可读
                token => {
                    // 从 connections Vec 中获取对应的连接
                    let connection = &mut connections[token.0 - 1];
                    // 创建一个 1024 字节的缓冲区
                    let mut buffer = [0; 1024];

                    // 尝试从连接中读取数据到缓冲区
                    match connection.read(&mut buffer) {
                        // 如果读取成功且读取到的数据长度大于 0
                        Ok(n) if n > 0 => {
                            // 将缓冲区中的数据全部写回到连接中
                            connection.write_all(&buffer[..n])?;
                        }
                        // 如果读取失败或读取到的数据长度为 0，表示连接关闭或发生错误
                        _ => {
                            // 连接关闭或错误处理
                        }
                    }
                }
            }
        }
    }
}


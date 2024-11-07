
use mio::{Events, Interest, Poll, Token};
use std::fs::File;
use std::io::{self, Read};
use std::os::unix::io::AsRawFd;
use mio::unix::SourceFd;

const FILE: Token = Token(0);

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    // 打开文件
    let mut file = File::open("test.txt")?;
    let fd = file.as_raw_fd();

    // 将文件描述符包装为 SourceFd
    let mut source = SourceFd(&fd);

    // 注册到 poll
    poll.registry().register(
        &mut source,
        FILE,
        Interest::READABLE
    )?;

    let mut buffer = Vec::new();
    let mut temp_buf = [0; 1024];

    'outer: loop {
        poll.poll(&mut events, None)?;

        for event in events.iter() {
            if event.token() == FILE {
                loop {
                    match file.read(&mut temp_buf) {
                        Ok(0) => {
                            // 文件读取完成
                            break 'outer;
                        }
                        Ok(n) => {
                            buffer.extend_from_slice(&temp_buf[..n]);
                        }
                        Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                            // 需要等待更多数据
                            break;
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
        }
    }

    println!("Read {} bytes", buffer.len());

    // 将读取的内容转换为字符串（假设是UTF-8编码）
    match String::from_utf8(buffer) {
        Ok(content) => println!("Content: {}", content),
        Err(_) => println!("File contains non-UTF8 data"),
    }

    Ok(())
}
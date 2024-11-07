use std::io;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use mio::*;

const CUSTOM: Token = Token(0);

// 自定义事件源
struct CustomSource {
    receiver: Receiver<String>,
    has_data: Arc<Mutex<bool>>,
}

impl CustomSource {
    fn new() -> (Sender<String>, Self) {
        let (tx, rx) = channel();
        let has_data = Arc::new(Mutex::new(false));
        let has_data_clone = has_data.clone();

        // 在传入闭包前克隆 sender
        let tx_clone = tx.clone();

        // 创建一个监听发送事件的线程
        std::thread::spawn(move || {
            loop {
                if let Ok(_) = tx_clone.send("Hello, MIO!".to_string()) {
                    let mut flag = has_data_clone.lock().unwrap();
                    *flag = true;
                }
                std::thread::sleep(Duration::from_secs(1));
            }
        });

        (tx, CustomSource {
            receiver: rx,
            has_data,
        })
    }

    fn try_recv(&mut self) -> io::Result<Option<String>> {
        match self.receiver.try_recv() {
            Ok(msg) => {
                let mut flag = self.has_data.lock().unwrap();
                *flag = false;
                Ok(Some(msg))
            }
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
            Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Channel closed")),
        }
    }

    fn has_data(&self) -> bool {
        *self.has_data.lock().unwrap()
    }
}

impl event::Source for CustomSource {
    fn register(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        Ok(())
    }

    fn reregister(
        &mut self,
        registry: &Registry,
        token: Token,
        interests: Interest,
    ) -> io::Result<()> {
        Ok(())
    }

    fn deregister(&mut self, registry: &Registry) -> io::Result<()> {
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    let (sender, mut custom_source) = CustomSource::new();

    // 注册自定义事件源
    poll.registry().register(&mut custom_source, CUSTOM, Interest::READABLE)?;

    // 在另一个线程中发送消息
    let sender_clone = sender.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_secs(1));
        sender_clone.send("Hello from thread!".to_string()).unwrap();
    });

    println!("Waiting for messages...");

    let mut message_count = 0;
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;

        if custom_source.has_data() {
            if let Ok(Some(msg)) = custom_source.try_recv() {
                println!("Received message: {}", msg);
                message_count += 1;

                if message_count >= 3 {
                    break;
                }
            }
        }
    }

    Ok(())
}
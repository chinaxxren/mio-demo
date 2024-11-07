use mio::*;
use std::time::{Duration, Instant};

const TIMER: Token = Token(0);

struct Timer {
    deadline: Instant,
}

impl Timer {
    fn new(timeout: Duration) -> Timer {
        Timer {
            deadline: Instant::now() + timeout,
        }
    }

    fn is_expired(&self) -> bool {
        Instant::now() >= self.deadline
    }
}

fn main() -> std::io::Result<()> {
    let mut poll = Poll::new()?;
    let mut events = Events::with_capacity(1024);

    let timer = Timer::new(Duration::from_secs(5));

    loop {
        let timeout = if timer.is_expired() {
            println!("Timer expired!");
            break;
        } else {
            Some(Duration::from_millis(100))
        };

        poll.poll(&mut events, timeout)?;
    }

    Ok(())
}
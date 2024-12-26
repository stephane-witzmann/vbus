use std::sync::atomic::AtomicBool;
use std::sync::{atomic, Arc};

pub fn atomic_flag() -> (AtomicFlagReader, AtomicFlagWriter) {
    let flag = Arc::new(atomic::AtomicBool::new(false));
    (
        AtomicFlagReader::new(flag.clone()),
        AtomicFlagWriter::new(flag),
    )
}

#[derive(Clone)]
pub struct AtomicFlagReader {
    flag: Arc<AtomicBool>,
}

impl AtomicFlagReader {
    fn new(flag: Arc<AtomicBool>) -> Self {
        Self { flag }
    }

    pub fn check(&self) -> bool {
        self.flag.load(atomic::Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct AtomicFlagWriter {
    flag: Arc<AtomicBool>,
}

impl AtomicFlagWriter {
    fn new(flag: Arc<AtomicBool>) -> Self {
        Self { flag }
    }

    pub fn raise(&mut self) {
        self.flag.store(true, atomic::Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_raise() {
        let (reader, mut writer) = atomic_flag();
        assert_eq!(reader.check(), false);

        writer.raise();
        assert_eq!(reader.check(), true);
    }

    #[test]
    fn test_stop_thread() {
        const FLAG_WAIT_TIME: Duration = Duration::from_millis(300);
        const THREAD_WAIT_TIME: Duration = Duration::from_millis(50);

        assert!(THREAD_WAIT_TIME < FLAG_WAIT_TIME);

        let (reader, mut writer) = atomic_flag();

        let join_handle = std::thread::spawn(move || {
            let mut counter = 0;
            loop {
                if reader.check() {
                    return counter;
                }

                counter += 1;
                sleep(THREAD_WAIT_TIME);
            }
        });

        sleep(FLAG_WAIT_TIME);
        writer.raise();

        let counter = join_handle.join().unwrap();
        assert!(counter > 0);
    }
}

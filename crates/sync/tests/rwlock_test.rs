use sync::rwlock::SpinRwLock;

#[test]
fn test_read_write_aquire() {
    let rwlock = SpinRwLock::new(10);
    let read_guard = rwlock.read();
    assert_eq!(*read_guard, 10);
    drop(read_guard);

    let mut write_guard = rwlock.write();
    assert_eq!(*write_guard, 10);
    *write_guard = 20;
    drop(write_guard);

    let read_guard = rwlock.read();
    assert_eq!(*read_guard, 20);
    drop(read_guard);
}

#[test]
fn write_stale_on_read() {
    let rwlock = SpinRwLock::new(10);
    let _read_guard = rwlock.read();
    let write_guard = rwlock.try_write(true);
    assert!(write_guard.is_none());
}

#[test]
fn read_stale_on_write() {
    let rwlock = SpinRwLock::new(10);
    let _write_guard = rwlock.write();
    let read_guard = rwlock.try_read();
    assert!(read_guard.is_none());
}

#[test]
fn concurrent_writers_no_lost_updates() {
    use std::sync::Arc;
    use std::thread;

    let rwlock = Arc::new(SpinRwLock::new(0));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let lock = Arc::clone(&rwlock);
            thread::spawn(move || {
                for _ in 0..10_000 {
                    let mut guard = lock.write();
                    *guard += 1;
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    assert_eq!(*rwlock.read(), 80_000);
}

#[test]
fn writer_waits_for_all_readers() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    let rwlock = Arc::new(SpinRwLock::new(0));
    let barrier = Arc::new(Barrier::new(3)); // 2 readers + main thread

    let readers: Vec<_> = (0..2)
        .map(|_| {
            let lock = Arc::clone(&rwlock);
            let b = Arc::clone(&barrier);
            thread::spawn(move || {
                let guard = lock.read();
                b.wait();
                thread::sleep(Duration::from_millis(50));
                drop(guard);
            })
        })
        .collect();

    barrier.wait();
    assert!(rwlock.try_write(true).is_none());

    for r in readers {
        r.join().unwrap();
    }

    assert!(rwlock.try_write(true).is_some());
}

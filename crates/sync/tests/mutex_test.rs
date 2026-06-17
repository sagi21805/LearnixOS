use sync::mutex::SpinMutex;

#[test]
fn test_lock_acquire() {
    let mutex = SpinMutex::new(10);
    let mut guard = mutex.lock();
    assert_eq!(*guard, 10);
    *guard = 20;
    drop(guard);
    let guard = mutex.lock();
    assert_eq!(*guard, 20);
    drop(guard);
}

#[test]
fn lock_stale_while_held() {
    let mutex = SpinMutex::new(10);
    let _guard = mutex.lock();
    let second_guard = mutex.try_lock();
    assert!(second_guard.is_none());
}

#[test]
fn concurrent_lockers_no_lost_updates() {
    use std::sync::Arc;
    use std::thread;

    let mutex = Arc::new(SpinMutex::new(0));
    let threads: Vec<_> = (0..8)
        .map(|_| {
            let lock = Arc::clone(&mutex);
            thread::spawn(move || {
                for _ in 0..10_000 {
                    let mut guard = lock.lock();
                    *guard += 1;
                }
            })
        })
        .collect();

    for t in threads {
        t.join().unwrap();
    }

    assert_eq!(*mutex.lock(), 80_000);
}

#[test]
fn second_thread_waits_for_release() {
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::Duration;

    let mutex = Arc::new(SpinMutex::new(0));
    let barrier = Arc::new(Barrier::new(2));

    let holder = {
        let lock = Arc::clone(&mutex);
        let b = Arc::clone(&barrier);
        thread::spawn(move || {
            let guard = lock.lock();
            b.wait(); // signal "I'm holding the lock"
            thread::sleep(Duration::from_millis(50));
            drop(guard);
        })
    };

    barrier.wait();
    assert!(mutex.try_lock().is_none());

    holder.join().unwrap();

    assert!(mutex.try_lock().is_some());
}

#[test]
fn lock_not_stuck_after_panic_while_held() {
    use std::sync::Arc;
    use std::thread;

    let mutex = Arc::new(SpinMutex::new(0));
    let lock = Arc::clone(&mutex);

    let result = thread::spawn(move || {
        let mut guard = lock.lock();
        *guard = 1;
        panic!("simulated failure while holding lock");
    })
    .join();

    assert!(result.is_err());
    assert!(mutex.try_lock().is_some());
}

#[derive(Clone, Copy)]
struct Pair {
    a: i64,
    b: i64,
} // invariant: a == b always

#[test]
fn no_concurrent_holders_under_contention() {
    use std::sync::Arc;
    use std::thread;

    let mutex = Arc::new(SpinMutex::new(Pair { a: 0, b: 0 }));
    let writer_lock = Arc::clone(&mutex);

    let writer = thread::spawn(move || {
        for i in 1..=5000 {
            let mut g = writer_lock.lock();
            g.a = i;
            g.b = i;
        }
    });

    let checkers: Vec<_> = (0..4)
        .map(|_| {
            let lock = Arc::clone(&mutex);
            thread::spawn(move || {
                for _ in 0..5000 {
                    let g = lock.lock();
                    assert_eq!(
                        g.a, g.b,
                        "mutual exclusion violated: saw inconsistent \
                         state"
                    );
                }
            })
        })
        .collect();

    writer.join().unwrap();
    for c in checkers {
        c.join().unwrap();
    }
}

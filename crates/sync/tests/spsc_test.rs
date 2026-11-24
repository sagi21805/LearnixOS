use std::{hint::spin_loop, thread};

use sync::spsc::SpscRingBuffer;

#[test]
fn consumer_sees_items_in_push_order() {
    let mut buffer = SpscRingBuffer::new(Box::new([0; 4]));
    let (mut producer, mut consumer) = buffer.split();
    thread::scope(|s| {
        s.spawn(move || {
            for item in 0..100 {
                while producer.push(item).is_none() {
                    spin_loop()
                }
            }
        });
        s.spawn(move || {
            for expected in 0..100 {
                let item = loop {
                    if let Some(item) = consumer.pop() {
                        break item;
                    }
                    spin_loop()
                };
                assert_eq!(item, expected);
            }
        });
    });
}

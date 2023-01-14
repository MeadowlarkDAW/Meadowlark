//! Algorithm ported from:
//! https://github.com/free-audio/clap-helpers/blob/main/include/clap/helpers/reducing-param-queue.hh
//! https://github.com/free-audio/clap-helpers/blob/main/include/clap/helpers/reducing-param-queue.hxx
//!
//! MIT License
//!
//! Copyright (c) 2021 Alexandre BIQUE
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use atomic_refcell::{AtomicRefCell, AtomicRefMut};
use basedrop::Shared;
use fnv::FnvHashMap;
use std::collections::hash_map::Entry;
use std::hash::Hash;
use std::sync::atomic::{AtomicUsize, Ordering};

static NULL_USIZE: usize = std::usize::MAX;

pub trait ReducFnvValue: Send + 'static {
    #[allow(unused_variables)]
    fn update(&mut self, new_value: &Self) {}
}

pub struct ReducingFnvQueue<K: Hash + Eq + Send + 'static, V: ReducFnvValue> {
    // TODO: Once we feel sound about the correctness of this algorithm, we can
    // change this to use the `AtomicRefCell` type instead.
    queues: [AtomicRefCell<FnvHashMap<K, V>>; 2],

    free: AtomicUsize,
    producer: AtomicUsize,
    consumer: AtomicUsize,
}

impl<K: Hash + Eq + Send + 'static, V: ReducFnvValue> ReducingFnvQueue<K, V> {
    pub fn new_channel(
        capacity: usize,
        coll_handle: &basedrop::Handle,
    ) -> (ReducFnvProducer<K, V>, ReducFnvConsumer<K, V>) {
        assert_ne!(capacity, 0);

        let mut queue0: FnvHashMap<K, V> = FnvHashMap::default();
        let mut queue1: FnvHashMap<K, V> = FnvHashMap::default();

        // TODO: Do we really need to multiply the capacity by two like in the
        // C++ implementation?
        queue0.reserve(capacity * 2);
        queue1.reserve(capacity * 2);

        let queues = [AtomicRefCell::new(queue0), AtomicRefCell::new(queue1)];

        let free = AtomicUsize::new(0);
        let producer = AtomicUsize::new(1);
        let consumer = AtomicUsize::new(NULL_USIZE);

        let shared = Shared::new(coll_handle, Self { queues, free, producer, consumer });

        (ReducFnvProducer { shared: Shared::clone(&shared) }, ReducFnvConsumer { shared })
    }
}

pub struct ReducFnvProducer<K: Hash + Eq + Send + 'static, V: ReducFnvValue> {
    shared: Shared<ReducingFnvQueue<K, V>>,
}

impl<K: Hash + Eq + Send + 'static, V: ReducFnvValue> ReducFnvProducer<K, V> {
    pub fn set(&mut self, key: K, value: V) {
        let producer_i = self.shared.producer.load(Ordering::SeqCst);

        if producer_i == NULL_USIZE {
            panic!("ReducingFnvQueue::set(): producer pointer is invalid");
        }

        let mut producer = self.shared.queues[producer_i].borrow_mut();

        let _ = producer.insert(key, value);
    }

    pub fn set_or_update(&mut self, key: K, value: V) {
        let producer_i = self.shared.producer.load(Ordering::SeqCst);

        if producer_i == NULL_USIZE {
            panic!("ReducingFnvQueue::set_or_update(): producer pointer is invalid");
        }

        let mut producer = self.shared.queues[producer_i].borrow_mut();

        match producer.entry(key) {
            Entry::Occupied(mut old_value) => old_value.get_mut().update(&value),
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }

    pub fn producer_done(&mut self) {
        if self.shared.consumer.load(Ordering::SeqCst) != NULL_USIZE {
            return;
        }

        let temp = self.shared.producer.load(Ordering::SeqCst);
        if temp == NULL_USIZE {
            panic!("ReducingFnvQueue::producer_done(): temp pointer is invalid");
        }

        let free = self.shared.free.load(Ordering::SeqCst);
        if free == NULL_USIZE {
            panic!("ReducingFnvQueue::producer_done(): free pointer is invalid");
        }

        self.shared.producer.store(free, Ordering::SeqCst);
        self.shared.free.store(NULL_USIZE, Ordering::SeqCst);
        self.shared.consumer.store(temp, Ordering::SeqCst)
    }

    pub fn produce<'a, P: FnOnce(ReducFnvProducerRefMut<'a, K, V>)>(&'a mut self, p: P) {
        let producer_i = self.shared.producer.load(Ordering::SeqCst);

        if producer_i == NULL_USIZE {
            panic!("ReducingFnvQueue::produce(): producer pointer is invalid");
        }

        {
            let producer = self.shared.queues[producer_i].borrow_mut();

            (p)(ReducFnvProducerRefMut { producer });
        }

        if self.shared.consumer.load(Ordering::SeqCst) != NULL_USIZE {
            return;
        }

        let temp = producer_i;

        let free = self.shared.free.load(Ordering::SeqCst);
        if free == NULL_USIZE {
            panic!("ReducingFnvQueue::produce(): free pointer is invalid");
        }

        self.shared.producer.store(free, Ordering::SeqCst);
        self.shared.free.store(NULL_USIZE, Ordering::SeqCst);
        self.shared.consumer.store(temp, Ordering::SeqCst)
    }
}

pub struct ReducFnvProducerRefMut<'a, K: Hash + Eq + Send + 'static, V: ReducFnvValue> {
    producer: AtomicRefMut<'a, FnvHashMap<K, V>>,
}

impl<'a, K: Hash + Eq + Send + 'static, V: ReducFnvValue> ReducFnvProducerRefMut<'a, K, V> {
    pub fn set(&mut self, key: K, value: V) {
        let _ = self.producer.insert(key, value);
    }

    pub fn set_or_update(&mut self, key: K, value: V) {
        match self.producer.entry(key) {
            Entry::Occupied(mut old_value) => old_value.get_mut().update(&value),
            Entry::Vacant(entry) => {
                entry.insert(value);
            }
        }
    }
}

pub struct ReducFnvConsumer<K: Hash + Eq + Send + 'static, V: ReducFnvValue> {
    shared: Shared<ReducingFnvQueue<K, V>>,
}

impl<K: Hash + Eq + Send + 'static, V: ReducFnvValue> ReducFnvConsumer<K, V> {
    pub fn consume<C: FnMut(&K, &V)>(&mut self, mut c: C) {
        let consumer_i = self.shared.consumer.load(Ordering::SeqCst);

        if consumer_i == NULL_USIZE {
            return;
        }

        {
            let mut consumer = self.shared.queues[consumer_i].borrow_mut();

            for (key, value) in consumer.iter() {
                (c)(key, value);
            }

            consumer.clear();
        }

        let free = self.shared.free.load(Ordering::SeqCst);

        if free != NULL_USIZE {
            return;
        }

        self.shared.free.store(consumer_i, Ordering::SeqCst);
        self.shared.consumer.store(NULL_USIZE, Ordering::SeqCst);
    }
}

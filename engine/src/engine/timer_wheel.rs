use fnv::FnvHashMap;
use hierarchical_hash_wheel_timer::wheels::cancellable::{
    CancellableTimerEntry, QuadWheelWithOverflow,
};
use hierarchical_hash_wheel_timer::wheels::Skip;
use meadowlark_plugin_api::ext::timer::TimerID;
use std::hash::Hash;
use std::rc::Rc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TimerEntryKey {
    MainIdle,
    GarbageCollect,
    PluginRegistered { plugin_unique_id: u64, timer_id: TimerID },
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct TimerEntry {
    pub key: TimerEntryKey,
    interval: Duration,
}

impl CancellableTimerEntry for TimerEntry {
    type Id = TimerEntryKey;
    fn id(&self) -> &Self::Id {
        &self.key
    }
}

pub(crate) struct EngineTimerWheel {
    main_idle_entry: Rc<TimerEntry>,
    garbage_collect_entry: Rc<TimerEntry>,
    registered_plugin_entries: FnvHashMap<TimerEntryKey, Rc<TimerEntry>>,
    wheel: QuadWheelWithOverflow<TimerEntry>,
    next_expected_tick_instant: Instant,
}

impl EngineTimerWheel {
    pub fn new(main_idle_interval_ms: u32, garbage_collect_interval_ms: u32) -> Self {
        assert_ne!(main_idle_interval_ms, 0);
        assert!(garbage_collect_interval_ms >= main_idle_interval_ms);

        let main_idle_interval = Duration::from_millis(u64::from(main_idle_interval_ms));
        let main_idle_entry =
            Rc::new(TimerEntry { key: TimerEntryKey::MainIdle, interval: main_idle_interval });

        let garbage_collect_interval =
            Duration::from_millis(u64::from(garbage_collect_interval_ms));
        let garbage_collect_entry = Rc::new(TimerEntry {
            key: TimerEntryKey::GarbageCollect,
            interval: garbage_collect_interval,
        });

        let mut wheel = QuadWheelWithOverflow::new();
        wheel.insert_ref_with_delay(Rc::clone(&main_idle_entry), main_idle_entry.interval).unwrap();
        wheel
            .insert_ref_with_delay(
                Rc::clone(&garbage_collect_entry),
                garbage_collect_entry.interval,
            )
            .unwrap();

        Self {
            main_idle_entry,
            garbage_collect_entry,
            registered_plugin_entries: FnvHashMap::default(),
            wheel,
            next_expected_tick_instant: Instant::now() + main_idle_interval,
        }
    }

    pub fn register_plugin_timer(
        &mut self,
        plugin_unique_id: u64,
        timer_id: TimerID,
        interval_ms: u32,
    ) {
        assert_ne!(interval_ms, 0);

        let key = TimerEntryKey::PluginRegistered { plugin_unique_id, timer_id };
        let interval = Duration::from_millis(u64::from(interval_ms));
        let new_entry = Rc::new(TimerEntry { key, interval });

        if self.registered_plugin_entries.insert(key, Rc::clone(&new_entry)).is_some() {
            if let Err(e) = self.wheel.cancel(&key) {
                log::error!("Unexpected error while cancelling timer: {:?}", e);
            }
        }

        if let Err(e) = self.wheel.insert_ref_with_delay(new_entry, interval) {
            self.registered_plugin_entries.remove(&key);
            log::error!("Unexpected error while inserting entry into timer: {:?}", e);
        }
    }

    pub fn unregister_plugin_timer(&mut self, plugin_unique_id: u64, timer_id: TimerID) {
        let key = TimerEntryKey::PluginRegistered { plugin_unique_id, timer_id };

        if self.registered_plugin_entries.remove(&key).is_some() {
            if let Err(e) = self.wheel.cancel(&key) {
                log::error!("Unexpected error while cancelling timer: {:?}", e);
            }
        }
    }

    pub fn unregister_all_timers_on_plugin(&mut self, plugin_unique_id: u64) {
        let mut entries_to_remove: Vec<TimerEntryKey> = self
            .registered_plugin_entries
            .iter()
            .filter_map(|(key, _)| {
                if let TimerEntryKey::PluginRegistered {
                    plugin_unique_id: key_plugin_unique_id,
                    ..
                } = key
                {
                    if *key_plugin_unique_id == plugin_unique_id {
                        Some(*key)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        for key in entries_to_remove.drain(..) {
            if let TimerEntryKey::PluginRegistered {
                plugin_unique_id: key_plugin_unique_id,
                timer_id,
            } = key
            {
                self.unregister_plugin_timer(key_plugin_unique_id, timer_id);
            }
        }
    }

    pub fn advance(&mut self, elapsed_entries: &mut Vec<Rc<TimerEntry>>) {
        // Calculate much time has passed since the expected instant of the next tick.
        let time_elapsed = Instant::now().duration_since(self.next_expected_tick_instant);

        // Calculate how many ticks we need to run on the timer wheel (how many
        // milliseconds have passed since the last time we ticked/skipped).
        let num_ticks = 1 + ((time_elapsed.as_secs_f64() * 1_000.0).floor() as u64);

        // Tick through the timer wheel and collect all the the entries that have elapsed.
        for _ in 0..num_ticks {
            elapsed_entries.append(&mut self.wheel.tick());
        }

        // Re-schedule the entries which have elapsed so they are periodic (as apposed to one-shot).
        for entry in elapsed_entries.iter() {
            if let Err(e) = self.wheel.insert_ref_with_delay(Rc::clone(entry), entry.interval) {
                log::error!("Unexpected error while re-scheduling event in timer: {:?}", e);
            }
        }

        // Calculate how many ticks until the next event.
        let mut num_ticks_to_next_event = num_ticks;
        if let Skip::Millis(ms) = self.wheel.can_skip() {
            // The timer wheel has this many milliseconds (ticks) it can skip without
            // triggering an event, so advance the timer by that many ticks.
            self.wheel.skip(ms);
            num_ticks_to_next_event += u64::from(ms);
        }

        self.next_expected_tick_instant += Duration::from_millis(num_ticks_to_next_event);
    }

    pub fn next_expected_tick_instant(&self) -> Instant {
        self.next_expected_tick_instant
    }

    pub fn reset(&mut self) {
        self.wheel = QuadWheelWithOverflow::new();
        self.wheel
            .insert_ref_with_delay(Rc::clone(&self.main_idle_entry), self.main_idle_entry.interval)
            .unwrap();
        self.wheel
            .insert_ref_with_delay(
                Rc::clone(&self.garbage_collect_entry),
                self.garbage_collect_entry.interval,
            )
            .unwrap();
    }
}

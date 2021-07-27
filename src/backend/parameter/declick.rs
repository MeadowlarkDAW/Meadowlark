// Code used from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/declick.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
//
//  Thanks wrl! :)

use std::fmt;

use rusty_daw_time::{SampleRate, Seconds};

use super::smooth::{Smooth, SmoothStatus};

const DECLICK_SETTLE: f32 = 0.001;

pub struct DeclickOutput<'a, T> {
    pub from: &'a T,
    pub to: &'a T,

    pub fade: &'a [f32],
    pub status: SmoothStatus,
}

pub struct Declick<T: Sized + Clone> {
    current: T,
    next: Option<T>,
    staged: Option<T>,

    fade: Smooth<f32>,
}

impl<T> Declick<T>
where
    T: Sized + Clone + Eq,
{
    pub fn new(initial: T) -> Self {
        Self {
            current: initial,
            next: None,
            staged: None,

            fade: Smooth::new(0.0),
        }
    }

    pub fn reset(&mut self, to: T) {
        self.current = to;
        self.next = None;
        self.staged = None;

        self.fade.reset(0.0);
    }

    pub fn set(&mut self, to: T) {
        if self.dest() == &to {
            return;
        }

        if self.next.is_none() {
            self.next = Some(to);

            self.fade.reset(0.0);
            self.fade.set(1.0);
        } else {
            self.staged = Some(to);
        }
    }

    pub fn set_speed(&mut self, sample_rate: SampleRate, seconds: Seconds) {
        self.fade.set_speed(sample_rate, seconds);
    }

    #[inline]
    pub fn output(&self) -> DeclickOutput<T> {
        let fade = self.fade.output();

        DeclickOutput {
            from: &self.current,
            to: self.next.as_ref().unwrap_or(&self.current),

            fade: fade.values,
            status: fade.status,
        }
    }

    #[inline]
    pub fn current_value(&self) -> (&T, SmoothStatus) {
        let (_, status) = self.fade.current_value();

        (&self.current, status)
    }

    #[inline]
    pub fn dest(&self) -> &T {
        self.staged
            .as_ref()
            .or_else(|| self.next.as_ref())
            .unwrap_or(&self.current)
    }

    #[inline]
    pub fn is_active(&self) -> bool {
        self.next.is_some()
    }

    #[inline]
    pub fn process(&mut self, nframes: usize) {
        self.fade.process(nframes);
    }

    pub fn update_status(&mut self) {
        if !self.is_active() {
            return;
        }

        self.fade.update_status_with_epsilon(DECLICK_SETTLE);

        if self.fade.is_active() {
            return;
        }

        self.current = self.next.take().unwrap();
        self.next = self.staged.take();
    }
}

impl<T> fmt::Debug for Declick<T>
where
    T: fmt::Debug + Sized + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(concat!("Declick<", stringify!(T), ">"))
            .field("current", &self.current)
            .field("next", &self.next)
            .field("staged", &self.staged)
            .field("fade", &self.fade)
            .finish()
    }
}

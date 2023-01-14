// Some modified code from baseplug:
//
// https://github.com/wrl/baseplug/blob/trunk/src/declick.rs
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-APACHE
// https://github.com/wrl/baseplug/blob/trunk/LICENSE-MIT
//
//  Thanks wrl! :)

use std::fmt;

use super::smooth::{SmoothF32, SmoothStatus};

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

    fade: SmoothF32,
}

impl<T> Declick<T>
where
    T: Sized + Clone + Eq,
{
    pub fn new(initial: T, max_blocksize: usize) -> Self {
        Self {
            current: initial,
            next: None,
            staged: None,
            fade: SmoothF32::new(0.0, max_blocksize),
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

    pub fn set_speed(&mut self, sample_rate: u32, seconds: f64) {
        self.fade.set_speed(sample_rate, seconds);
    }

    pub fn output(&self) -> DeclickOutput<T> {
        let fade = self.fade.output();

        DeclickOutput {
            from: &self.current,
            to: self.next.as_ref().unwrap_or(&self.current),

            fade: fade.values,
            status: fade.status,
        }
    }

    pub fn current_value(&self) -> (&T, SmoothStatus) {
        let (_, status) = self.fade.current_value();

        (&self.current, status)
    }

    pub fn dest(&self) -> &T {
        self.staged.as_ref().or(self.next.as_ref()).unwrap_or(&self.current)
    }

    pub fn is_active(&self) -> bool {
        self.next.is_some()
    }

    pub fn process(&mut self, frames: usize) {
        self.fade.process(frames);
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

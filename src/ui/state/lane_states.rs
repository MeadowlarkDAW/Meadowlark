use super::{ChannelBaseColor, UiEvent};
use std::ops::RangeBounds;
use vizia::prelude::*;

/// The state of every lane in the timeline.
#[derive(Debug, Lens, Clone)]
pub struct LaneStates {
    /// The state of every lane in the timeline.
    pub lanes: Vec<LaneState>,
    /// The currently active or last clicked lane index.
    active_lane: usize,
}

impl LaneStates {
    /// Creates a new lane states collection.
    pub fn new(lanes: Vec<LaneState>) -> Self {
        Self { lanes, active_lane: 0 }
    }

    // ----- Select -----

    /// Selects the lane at the given `index`.
    pub fn select_lane(&mut self, index: usize) {
        if let Some(lane) = self.lanes.get_mut(index) {
            lane.selected = true;
            self.active_lane = index;
        }
    }

    /// Selects all unselected lanes.
    pub fn select_all_lanes(&mut self) {
        self.unselected_lanes_mut().for_each(|x| x.selected = true);
    }

    /// Unselects the lane at the given `index`.
    pub fn unselect_lane(&mut self, index: usize) {
        if let Some(lane) = self.lanes.get_mut(index) {
            lane.selected = false;
            if self.active_lane == index {
                self.active_lane = 0;
            }
        }
    }

    /// Unselects all selected lanes.
    pub fn unselect_all_lanes(&mut self) {
        self.selected_lanes_mut().for_each(|x| x.selected = false);
    }

    /// Returns an iterator over the selected lanes.
    pub fn selected_lanes(&self) -> impl Iterator<Item = &LaneState> {
        self.lanes.iter().filter(|x| x.selected)
    }

    /// Returns a mutable iterator over the selected lanes.
    pub fn selected_lanes_mut(&mut self) -> impl Iterator<Item = &mut LaneState> {
        self.lanes.iter_mut().filter(|x| x.selected)
    }

    /// Returns an iterator over the unselected lanes.
    pub fn unselected_lanes(&self) -> impl Iterator<Item = &LaneState> {
        self.lanes.iter().filter(|x| !x.selected)
    }

    /// Returns a mutable iterator over the unselected lanes.
    pub fn unselected_lanes_mut(&mut self) -> impl Iterator<Item = &mut LaneState> {
        self.lanes.iter_mut().filter(|x| !x.selected)
    }

    // ----- Clone -----

    /// Clones the lane at the given `index` or returns `None` if it doesn't exist.
    pub fn clone_lane(&mut self, index: usize) -> Option<LaneState> {
        self.lanes.get_mut(index).map(|lane| lane.clone())
    }

    /// Clones the lane at the given `index`.
    ///
    /// # Panics
    ///
    /// Panics if the `index` is out of bounds.
    pub fn clone_lane_unchecked(&mut self, index: usize) -> LaneState {
        self.lanes[index].clone()
    }

    // ----- Insert -----

    /// Inserts a new `lane` at the given `index`.
    pub fn insert_lane(&mut self, index: usize, lane: LaneState) {
        self.lanes.insert(index, lane);
    }

    /// Inserts new `lanes` at the given `index`.
    pub fn insert_lanes(&mut self, index: usize, mut lanes: Vec<LaneState>) {
        self.lanes.reserve(lanes.len());
        lanes.reverse();
        for lane in lanes {
            self.insert_lane(index, lane);
        }
    }

    // ----- Push -----

    /// Pushes a new `lane` into the collection.
    pub fn push_lane(&mut self, lane: LaneState) {
        self.lanes.push(lane);
    }

    /// Appends the given `lanes` into the collection.
    pub fn append_lanes(&mut self, lanes: &mut Vec<LaneState>) {
        self.lanes.append(lanes);
    }

    // ----- Remove -----

    /// Removes the lane at the given `index`.
    pub fn remove_lane(&mut self, index: usize) {
        self.lanes.remove(index);
    }

    /// Drains the lanes inside the given `range`.
    pub fn drain_lanes<R>(&mut self, range: R)
    where
        R: RangeBounds<usize>,
    {
        self.lanes.drain(range);
    }

    /// Removes the lanes inside of the given `vec`.
    pub fn remove_lanes_in_vec(&mut self, mut vec: Vec<usize>) {
        vec.reverse();
        for index in vec {
            self.remove_lane(index);
        }
    }

    /// Removes all lanes.
    pub fn delete_all_lanes(&mut self) {
        self.lanes.clear();
    }

    // ----- Utilities -----

    /// Returns the selected lane index moved by the given `amount` or `None` if it is out of bounds.
    pub fn index_moved_by(&self, amount: i32, index: usize) -> Option<usize> {
        if amount == 0 {
            return Some(index);
        }

        if amount > 0 {
            match index.overflowing_add(amount as usize) {
                (index, false) if index < self.lanes.len() => Some(index),
                _ => None,
            }
        } else {
            match index.overflowing_sub(amount.abs() as usize) {
                (index, false) => Some(index),
                _ => None,
            }
        }
    }

    /// Returns the index of the last selected lane.
    pub fn last_selected_index(&self) -> Option<usize> {
        self.lanes.iter().rposition(|x| x.selected)
    }

    /// Returns the indices of the lanes where the `filter` returned `true`.
    pub fn lane_indices<F>(&self, filter: F) -> Vec<usize>
    where
        F: Fn(&LaneState) -> bool,
    {
        self.lanes
            .iter()
            .enumerate()
            .filter(|(_, lane)| filter(lane))
            .map(|(index, _)| index)
            .collect()
    }
}

impl Model for LaneStates {
    fn event(&mut self, cx: &mut Context, event: &mut Event) {
        event.map(|event, _| match event {
            UiEvent::SelectLane(index) => {
                if !cx.modifiers().contains(Modifiers::CTRL) {
                    self.unselect_all_lanes();
                }

                if cx.modifiers().contains(Modifiers::SHIFT) {
                    self.lanes
                        .iter_mut()
                        .enumerate()
                        .filter(|(i, _)| {
                            if *index > self.active_lane {
                                *i >= self.active_lane && *i <= *index
                            } else {
                                *i >= *index && *i <= self.active_lane
                            }
                        })
                        .for_each(|(_, lane)| {
                            lane.selected = true;
                        });
                    return;
                }

                self.select_lane(*index);
            }
            UiEvent::InsertLane => {
                self.unselect_all_lanes();
                let index = (self.active_lane + 1).min(self.lanes.len());
                self.lanes.insert(index, LaneState::default());
                self.select_lane(index);
            }
            UiEvent::DuplicateSelectedLanes => {
                let mut lanes = Vec::new();
                let new_index = (1 + match self.last_selected_index() {
                    Some(index) => index,
                    None => 0,
                })
                .min(self.lanes.len());

                for index in self.lane_indices(|x| x.selected) {
                    lanes.push(self.clone_lane_unchecked(index));
                    self.unselect_lane(index);
                }

                self.insert_lanes(new_index, lanes);
                self.active_lane = new_index;
            }
            UiEvent::MoveSelectedLanesUp => {
                // TODO: Implement
            }
            UiEvent::MoveSelectedLanesDown => {
                // TODO: Implement
            }
            UiEvent::SelectAllLanes => {
                self.select_all_lanes();
            }
            UiEvent::DeleteSelectedLanes => {
                self.lanes.retain(|x| !x.selected);
                self.select_lane(self.active_lane.min(self.lanes.len().saturating_sub(1)));
            }
            UiEvent::SelectLaneAbove => {
                if let Some(index) = self.index_moved_by(-1, self.active_lane) {
                    self.unselect_all_lanes();
                    self.select_lane(index);
                }
            }
            UiEvent::SelectLaneBelow => {
                if let Some(index) = self.index_moved_by(1, self.active_lane) {
                    self.unselect_all_lanes();
                    self.select_lane(index);
                }
            }
            UiEvent::ActivateSelectedLanes => {
                self.selected_lanes_mut().for_each(|x| x.disabled = false);
            }
            UiEvent::DeactivateSelectedLanes => {
                self.selected_lanes_mut().for_each(|x| x.disabled = true);
            }
            UiEvent::ToggleSelectedLaneActivation => {
                self.selected_lanes_mut().for_each(|x| x.disabled ^= true);
            }
            _ => {}
        });
    }
}

#[derive(Debug, Lens, Clone)]
pub struct LaneState {
    /// The name of this lane.
    ///
    /// This will be `None` if this just uses the default name.
    pub name: Option<String>,

    /// The color of this lane.
    ///
    /// This will be `None` if this just uses the default color.
    pub color: Option<ChannelBaseColor>,

    /// The height of this lane (where 1.0 means the "global default lane height").
    ///
    /// If this is `None`, then this will use `TimelineGridState::lane_height`
    /// instead.
    ///
    /// The UI may mutate this directly without an event.
    pub height: Option<f64>,

    /// Represents if the lane is currently disabled, which means that all clips on this lane are bypassed.
    pub disabled: bool,

    /// Represents if the lane is currently selected.
    pub selected: bool,
}

impl Default for LaneState {
    fn default() -> Self {
        Self { name: None, color: None, height: None, disabled: false, selected: false }
    }
}

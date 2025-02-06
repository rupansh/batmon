// Copyright 2023 developers of the `batmon` project
// SPDX-License-Identifier: MPL-2.0

use crate::batstream::BatLvl;

/// Event Priority
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug)]
pub enum EvPriority {
    Low,
    Normal,
    High,
}

/// Event Priority Threshold
pub struct PriorityThreshold {
    /// Triggers Low Priorirty Event
    pub low: BatLvl,
    /// Triggers Normal Priority Event
    pub normal: BatLvl,
    /// Triggers High Priority Event
    pub high: BatLvl,
}

impl PriorityThreshold {
    pub fn priority(&self, lvl: BatLvl) -> Option<EvPriority> {
        if lvl <= self.high {
            return Some(EvPriority::High);
        }
        if lvl <= self.normal {
            return Some(EvPriority::Normal);
        }
        if lvl <= self.low {
            return Some(EvPriority::Low);
        }
        None
    }
}

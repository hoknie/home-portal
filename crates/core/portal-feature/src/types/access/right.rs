use std::fmt;

use super::{Action, Area};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Right {
    pub area: Area,
    pub action: Action,
}

impl Right {
    pub const fn new(area: Area, action: Action) -> Right {
        Right { area, action }
    }

    pub fn every() -> Vec<Right> {
        Area::ALL
            .into_iter()
            .flat_map(|area| {
                area.actions()
                    .iter()
                    .map(move |action| Right::new(area, *action))
            })
            .collect()
    }
}

impl fmt::Display for Right {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}.{}", self.area.name(), self.action.name())
    }
}

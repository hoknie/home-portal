use std::collections::BTreeSet;

use super::{Action, Area, Right};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Rights {
    admin: bool,
    granted: BTreeSet<Right>,
}

impl Rights {
    pub fn none() -> Rights {
        Rights::default()
    }

    pub fn admin() -> Rights {
        Rights {
            admin: true,
            granted: Right::every().into_iter().collect(),
        }
    }

    pub fn of(rights: impl IntoIterator<Item = Right>) -> Rights {
        Rights {
            admin: false,
            granted: rights
                .into_iter()
                .filter(|right| right.area.accepts(right.action))
                .collect(),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.admin
    }

    pub fn allows(&self, right: Right) -> bool {
        self.admin || self.granted.contains(&right)
    }

    pub fn within(&self, other: &Rights) -> bool {
        other.admin || (!self.admin && self.granted.is_subset(&other.granted))
    }

    pub fn by_area(&self) -> Vec<(Area, Vec<Action>)> {
        Area::ALL
            .into_iter()
            .filter_map(|area| {
                let actions: Vec<Action> = area
                    .actions()
                    .iter()
                    .copied()
                    .filter(|action| self.allows(Right::new(area, *action)))
                    .collect();
                (!actions.is_empty()).then_some((area, actions))
            })
            .collect()
    }
}

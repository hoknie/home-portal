use portal_automations::{effective_groups, effective_user};
use portal_scripts::ProcessIdentity;

pub struct PortalProcess;

impl ProcessIdentity for PortalProcess {
    fn user(&self) -> u32 {
        effective_user()
    }

    fn groups(&self) -> Vec<u32> {
        effective_groups()
    }
}

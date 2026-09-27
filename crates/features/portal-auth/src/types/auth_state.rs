use std::sync::Arc;

use crate::ports::Connection;
use crate::usecases::{ShowSession, SignIn, SignOut};

#[derive(Clone)]
pub struct AuthState {
    pub sign_in: SignIn,
    pub session: ShowSession,
    pub sign_out: SignOut,
    pub connection: Arc<dyn Connection>,
}

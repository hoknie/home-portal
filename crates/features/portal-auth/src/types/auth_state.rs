use std::sync::Arc;

use crate::ports::Connection;
use crate::usecases::{
    ChangePassword, CreateUser, DeleteUser, ListUsers, ShowSession, SignIn, SignOut,
};

#[derive(Clone)]
pub struct AuthState {
    pub sign_in: SignIn,
    pub session: ShowSession,
    pub sign_out: SignOut,
    pub list_users: ListUsers,
    pub create_user: CreateUser,
    pub change_password: ChangePassword,
    pub delete_user: DeleteUser,
    pub connection: Arc<dyn Connection>,
}

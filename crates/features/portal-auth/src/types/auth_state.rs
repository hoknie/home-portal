use std::sync::Arc;

use crate::ports::Connection;
use crate::usecases::{
    ChangeGroup, ChangePassword, ChangeUserGroup, CreateGroup, CreateUser, DeleteGroup, DeleteUser,
    ListGroups, ListUsers, ShowSession, SignIn, SignOut,
};

#[derive(Clone)]
pub struct AuthState {
    pub sign_in: SignIn,
    pub session: ShowSession,
    pub sign_out: SignOut,
    pub list_users: ListUsers,
    pub create_user: CreateUser,
    pub change_password: ChangePassword,
    pub change_group: ChangeUserGroup,
    pub delete_user: DeleteUser,
    pub list_groups: ListGroups,
    pub create_group: CreateGroup,
    pub change_group_entry: ChangeGroup,
    pub delete_group: DeleteGroup,
    pub connection: Arc<dyn Connection>,
}

use crate::usecases::ListSecrets;

#[derive(Clone)]
pub struct SecretsState {
    pub list: ListSecrets,
}

use std::net::IpAddr;
use std::sync::Arc;

use portal_feature::ApiError;
use time::OffsetDateTime;
use tokio::sync::Semaphore;

use super::Throttle;
use crate::helpers::verify_password;

pub struct PasswordChecks {
    permits: Arc<Semaphore>,
    throttle: Throttle,
}

impl Default for PasswordChecks {
    fn default() -> PasswordChecks {
        PasswordChecks {
            permits: Arc::new(Semaphore::new(Self::AT_ONCE)),
            throttle: Throttle::default(),
        }
    }
}

impl PasswordChecks {
    pub const AT_ONCE: usize = 2;
    pub const BUSY_RETRY_SECONDS: u64 = 1;

    pub async fn verify(
        &self,
        client: IpAddr,
        password: String,
        hash: Option<String>,
    ) -> Result<bool, ApiError> {
        let permit =
            self.permits
                .clone()
                .try_acquire_owned()
                .map_err(|_| ApiError::TooManyRequests {
                    retry_after_seconds: Self::BUSY_RETRY_SECONDS,
                })?;
        self.throttle
            .reserve(client, OffsetDateTime::now_utc())
            .map_err(|retry_after_seconds| ApiError::TooManyRequests {
                retry_after_seconds,
            })?;
        let checked =
            tokio::task::spawn_blocking(move || verify_password(&password, hash.as_deref())).await;
        drop(permit);
        match checked {
            Ok(true) => {
                self.throttle.succeed(client);
                Ok(true)
            }
            Ok(false) => {
                self.throttle.fail(client, OffsetDateTime::now_utc());
                Ok(false)
            }
            Err(error) => {
                self.throttle.release(client);
                Err(ApiError::Internal(format!("password check: {error}")))
            }
        }
    }
}

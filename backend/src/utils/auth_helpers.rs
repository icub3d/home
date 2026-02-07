use crate::{error::AppError, middleware::auth::AuthUser};

/// Require the user to be an admin (Owner or Admin role)
pub fn require_admin(auth: &AuthUser) -> Result<(), AppError> {
    if !auth.is_admin() {
        return Err(AppError::AuthError);
    }
    Ok(())
}

pub fn generate_random_token(length: usize) -> String {
    use rand::{distr::Alphanumeric, Rng};
    rand::rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

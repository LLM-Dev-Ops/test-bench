// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Authentication and authorization.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use chrono::{DateTime, Duration, Utc};
use std::collections::HashMap;
use parking_lot::RwLock;
use std::sync::Arc;

/// JWT claims
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID)
    pub sub: String,
    /// Expiration time
    pub exp: i64,
    /// Issued at
    pub iat: i64,
    /// User role
    pub role: UserRole,
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, String>,
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    /// Administrator with full access
    Admin,
    /// Regular user with standard access
    User,
    /// Read-only access
    Viewer,
    /// Service account for API integration
    Service,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Key ID
    pub id: String,
    /// Key value (hashed)
    pub key_hash: String,
    /// User ID
    pub user_id: String,
    /// User role
    pub role: UserRole,
    /// Active status
    pub active: bool,
    /// Created at
    pub created_at: DateTime<Utc>,
    /// Expires at
    pub expires_at: Option<DateTime<Utc>>,
}

/// Authentication service
pub struct AuthService {
    /// JWT secret key
    secret_key: String,
    /// Token expiration time (seconds)
    token_expiration: i64,
    /// API keys storage
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(secret_key: String, token_expiration: i64) -> Self {
        Self {
            secret_key,
            token_expiration,
            api_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a JWT token
    pub fn generate_token(&self, user_id: &str, role: UserRole) -> Result<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.token_expiration);

        let claims = JwtClaims {
            sub: user_id.to_string(),
            exp: exp.timestamp(),
            iat: now.timestamp(),
            role,
            metadata: HashMap::new(),
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret_key.as_bytes()),
        )?;

        Ok(token)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.secret_key.as_bytes()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }

    /// Generate an API key
    pub fn generate_api_key(&self, user_id: &str, role: UserRole) -> Result<(String, ApiKey)> {
        // Generate random API key
        let key = format!("ltb_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

        // Hash the key
        let key_hash = bcrypt::hash(&key, bcrypt::DEFAULT_COST)?;

        let api_key = ApiKey {
            id: uuid::Uuid::new_v4().to_string(),
            key_hash,
            user_id: user_id.to_string(),
            role,
            active: true,
            created_at: Utc::now(),
            expires_at: None,
        };

        // Store API key
        let mut keys = self.api_keys.write();
        keys.insert(api_key.id.clone(), api_key.clone());

        Ok((key, api_key))
    }

    /// Validate an API key
    pub fn validate_api_key(&self, key: &str) -> Option<ApiKey> {
        let keys = self.api_keys.read();

        for api_key in keys.values() {
            if api_key.active {
                // Verify the key hash
                if bcrypt::verify(key, &api_key.key_hash).unwrap_or(false) {
                    // Check expiration
                    if let Some(expires_at) = api_key.expires_at {
                        if expires_at < Utc::now() {
                            continue;
                        }
                    }

                    return Some(api_key.clone());
                }
            }
        }

        None
    }

    /// Revoke an API key
    pub fn revoke_api_key(&self, key_id: &str) -> bool {
        let mut keys = self.api_keys.write();
        if let Some(key) = keys.get_mut(key_id) {
            key.active = false;
            true
        } else {
            false
        }
    }

    /// List API keys for a user
    pub fn list_api_keys(&self, user_id: &str) -> Vec<ApiKey> {
        let keys = self.api_keys.read();
        keys.values()
            .filter(|k| k.user_id == user_id)
            .cloned()
            .collect()
    }

    /// Check if a role has permission
    pub fn has_permission(role: UserRole, required_role: UserRole) -> bool {
        match (role, required_role) {
            (UserRole::Admin, _) => true, // Admin has all permissions
            (r, req) if r == req => true,
            _ => false,
        }
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new(
            "default_secret_change_in_production".to_string(),
            3600, // 1 hour
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_validate_token() {
        let auth = AuthService::default();
        let token = auth.generate_token("user123", UserRole::User).unwrap();

        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.role, UserRole::User);
    }

    #[test]
    fn test_invalid_token() {
        let auth = AuthService::default();
        let result = auth.validate_token("invalid_token");
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_api_key() {
        let auth = AuthService::default();
        let (key, api_key) = auth.generate_api_key("user123", UserRole::User).unwrap();

        assert!(key.starts_with("ltb_"));
        assert_eq!(api_key.user_id, "user123");
        assert_eq!(api_key.role, UserRole::User);
        assert!(api_key.active);
    }

    #[test]
    fn test_validate_api_key() {
        let auth = AuthService::default();
        let (key, _) = auth.generate_api_key("user123", UserRole::User).unwrap();

        let validated = auth.validate_api_key(&key);
        assert!(validated.is_some());
        assert_eq!(validated.unwrap().user_id, "user123");
    }

    #[test]
    fn test_revoke_api_key() {
        let auth = AuthService::default();
        let (_, api_key) = auth.generate_api_key("user123", UserRole::User).unwrap();

        assert!(auth.revoke_api_key(&api_key.id));

        let keys = auth.list_api_keys("user123");
        assert_eq!(keys.len(), 1);
        assert!(!keys[0].active);
    }

    #[test]
    fn test_has_permission() {
        assert!(AuthService::has_permission(UserRole::Admin, UserRole::User));
        assert!(AuthService::has_permission(UserRole::User, UserRole::User));
        assert!(!AuthService::has_permission(UserRole::Viewer, UserRole::User));
    }
}

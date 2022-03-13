use crate::SimpleResult;
use config::{Config, File, FileFormat};
use route_recognizer::Router;
use serde::Deserialize;
use serenity::model::id::UserId;
use serenity::prelude::TypeMapKey;
use std::collections::HashMap;
use std::ops::Deref;

pub trait RbacPermission: AsRef<str> {}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Deserialize)]
pub struct Role(String);

impl Deref for Role {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

pub type U2r = HashMap<UserId, Vec<Role>>;
pub type R2p = HashMap<Role, Router<()>>;

#[derive(Debug, Clone)]
pub struct RbacManager {
    // User to Role
    pub u2r: U2r,
    // Role to Permission
    pub r2p: R2p,
}

impl RbacManager {
    pub fn new() -> SimpleResult<Self> {
        let u2r = Config::builder()
            .add_source(File::new("users.toml", FileFormat::Toml))
            .build()?
            .try_deserialize()?;

        let r2p: HashMap<Role, Vec<String>> = Config::builder()
            .add_source(File::new("permissions.toml", FileFormat::Toml))
            .build()?
            .try_deserialize()?;

        let r2p = r2p
            .into_iter()
            .map(|(r, ps)| {
                (
                    r,
                    ps.into_iter().fold(Router::new(), |mut router, p| {
                        router.add(&p, ());
                        router
                    }),
                )
            })
            .collect();

        Ok(RbacManager { u2r, r2p })
    }
}

pub struct RbacKey;

impl TypeMapKey for RbacKey {
    type Value = RbacManager;
}

pub trait HasRbacPermission {
    fn has_permission<P: RbacPermission>(&self, p: &P, rbac: &RbacManager) -> bool;
}

impl HasRbacPermission for Role {
    fn has_permission<P: RbacPermission>(&self, p: &P, rbac: &RbacManager) -> bool {
        rbac.r2p
            .get(self)
            .map(|r| r.recognize(p.as_ref()).is_ok())
            .unwrap_or(false)
    }
}

impl HasRbacPermission for UserId {
    fn has_permission<P: RbacPermission>(&self, p: &P, rbac: &RbacManager) -> bool {
        rbac.u2r
            .get(self)
            .map(|roles| roles.iter().any(|role| role.has_permission(p, rbac)))
            .unwrap_or(false)
    }
}

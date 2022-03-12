use route_recognizer::Router;
use serenity::model::id::UserId;
use std::collections::HashMap;
use std::ops::Deref;

pub trait RbacPermission: AsRef<str> {}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Role(String);

impl Deref for Role {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

pub struct RbacManager {
    // User to Role
    pub u2r: HashMap<UserId, Vec<Role>>,
    // Role to Permission
    pub r2p: HashMap<Role, Router<()>>,
}

pub trait HasRbacPermission {
    fn has_permission<P: RbacPermission>(&self, p: &P, rbac: &RbacManager) -> bool;
}

impl HasRbacPermission for Role {
    fn has_permission<P: RbacPermission>(&self, p: &P, rbac: &RbacManager) -> bool {
        rbac.r2p[self].recognize(p.as_ref()).is_ok()
    }
}

impl HasRbacPermission for UserId {
    fn has_permission<P: RbacPermission>(&self, p: &P, rbac: &RbacManager) -> bool {
        rbac.u2r[self]
            .iter()
            .any(|role| role.has_permission(p, rbac))
    }
}

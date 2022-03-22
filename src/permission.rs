pub mod rbac;

use async_trait::async_trait;
use serenity::client::Context;

#[async_trait]
pub trait HasPermission<P> {
    async fn has_permission(&self, ctx: &Context, perm: &P) -> bool;
}

macro_rules! _has_permission {
    ($id:expr, $ctx:expr, $p:expr) => {{
        let data = $ctx.data.read().await;
        let rbac = data.get::<$crate::RbacKey>().unwrap();
        $crate::permission::rbac::HasRbacPermission::has_permission($id, $p, rbac)
    }};
}

macro_rules! has_permission {
    ($perm:ident) => {
        #[async_trait::async_trait]
        impl $crate::permission::HasPermission<$perm> for serenity::model::id::UserId {
            async fn has_permission(&self, ctx: &Context, p: &$perm) -> bool {
                $crate::permission::_has_permission!(self, ctx, p)
            }
        }

        #[async_trait::async_trait]
        impl $crate::permission::HasPermission<$perm> for serenity::model::id::RoleId {
            async fn has_permission(&self, ctx: &Context, p: &$perm) -> bool {
                $crate::permission::_has_permission!(self, ctx, p)
            }
        }
    };
}

#[doc(hidden)]
pub(crate) use _has_permission;
pub(crate) use has_permission;

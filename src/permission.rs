use async_trait::async_trait;
use serenity::client::Context;
use serenity::model::id::UserId;

#[async_trait]
pub trait HasPermission<P> {
    async fn has_permission(&self, ctx: &Context, perm: &P) -> bool;
}

pub struct DefaultPermission;

#[async_trait]
impl HasPermission<DefaultPermission> for UserId {
    async fn has_permission(&self, _: &Context, _: &DefaultPermission) -> bool {
        false
    }
}

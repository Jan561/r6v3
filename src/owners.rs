use serenity::model::id::UserId;
use serenity::prelude::TypeMapKey;
use std::collections::HashSet;

pub struct Owners;

impl TypeMapKey for Owners {
    type Value = HashSet<UserId>;
}

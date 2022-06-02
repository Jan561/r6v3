use diesel::backend::Backend;
use diesel::deserialize::FromSql;
use diesel::serialize::ToSql;
use diesel::sql_types::Binary;
use std::fmt;
use std::fmt::{Display, Formatter};
use uuid;

#[derive(Debug, Clone, Copy, FromSqlRow, AsExpression, Hash, Eq, PartialEq)]
#[diesel(sql_type = Binary)]
pub struct UUID(pub uuid::Uuid);

impl UUID {
    pub fn random() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl From<UUID> for uuid::Uuid {
    fn from(s: UUID) -> Self {
        s.0
    }
}

impl Display for UUID {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<B> FromSql<Binary, B> for UUID
where
    B: Backend,
    Vec<u8>: FromSql<Binary, B>,
{
    fn from_sql(bytes: diesel::backend::RawValue<'_, B>) -> diesel::deserialize::Result<Self> {
        let val = <Vec<u8>>::from_sql(bytes)?;
        uuid::Uuid::from_slice(&val).map(UUID).map_err(Into::into)
    }
}

impl<B> ToSql<Binary, B> for UUID
where
    B: Backend,
    [u8]: ToSql<Binary, B>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, B>,
    ) -> diesel::serialize::Result {
        self.0.as_bytes().to_sql(out)
    }
}

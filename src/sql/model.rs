use crate::sql::{Inserted, NotInserted, Sql};
use crate::{SimpleError, SimpleResult};

#[derive(Debug, Clone, Default)]
pub struct MemberBuilder {
    user_id: Option<i64>,
    client_uuid: Option<String>,
    insertion_pending: Option<bool>,
    removal_pending: Option<bool>,
}

impl MemberBuilder {
    pub fn user_id(mut self, user_id: i64) -> Self {
        self.user_id = Some(user_id);
        self
    }

    pub fn client_uuid(mut self, client_uuid: String) -> Self {
        self.client_uuid = Some(client_uuid);
        self
    }

    pub fn insertion_pending(mut self, insertion_pending: Option<bool>) -> Self {
        self.insertion_pending = insertion_pending;
        self
    }

    pub fn removal_pending(mut self, removal_pending: Option<bool>) -> Self {
        self.removal_pending = removal_pending;
        self
    }

    pub fn build(self) -> Member<NotInserted> {
        Member {
            user_id: self.user_id.unwrap(),
            client_uuid: self.client_uuid.unwrap(),
            insertion_pending: self.insertion_pending.unwrap_or(false),
            removal_pending: self.removal_pending.unwrap_or(false),
            state: NotInserted,
        }
    }
}

pub struct Member<State> {
    user_id: i64,
    client_uuid: String,
    insertion_pending: bool,
    removal_pending: bool,
    state: State,
}

impl<State> Member<State> {
    pub fn user_id(&self) -> i64 {
        self.user_id
    }

    pub fn client_uuid(&self) -> &str {
        &self.client_uuid
    }

    pub fn insertion_pending(&self) -> bool {
        self.insertion_pending
    }

    pub fn removal_pending(&self) -> bool {
        self.removal_pending
    }
}

impl Member<NotInserted> {
    pub async fn insert(self, sql: &Sql) -> SimpleResult<Member<Inserted>> {
        let rows_affected = sqlx::query!(
            r#"
            INSERT INTO ts 
            (user_id, client_uuid, insertion_pending, removal_pending)
            VALUES (?, ?, ?, ?)
            ON CONFLICT DO NOTHING
            "#,
            self.user_id,
            self.client_uuid,
            self.insertion_pending,
            self.removal_pending,
        )
        .execute(&sql.connection)
        .await?
        .rows_affected();

        if rows_affected == 0 {
            return Err(SimpleError::NoRowsAffected);
        }

        let s = Member {
            user_id: self.user_id,
            client_uuid: self.client_uuid,
            insertion_pending: self.insertion_pending,
            removal_pending: self.removal_pending,
            state: Inserted {
                connection: sql.clone(),
            },
        };

        Ok(s)
    }
}

impl Member<Inserted> {
    pub async fn get(sql: &Sql, user_id: i64, removal_pending: bool) -> SimpleResult<Option<Self>> {
        let s = sqlx::query!(
            r#"
            SELECT * FROM ts WHERE (user_id = ?) AND (removal_pending = ?)
            "#,
            user_id,
            removal_pending,
        )
        .fetch_optional(&sql.connection)
        .await?
        .map(|entry| Self {
            user_id,
            removal_pending,
            client_uuid: entry.client_uuid,
            insertion_pending: entry.insertion_pending == 1,
            state: Inserted {
                connection: sql.clone(),
            },
        });

        Ok(s)
    }

    pub async fn modify<F>(&mut self, f: F) -> SimpleResult<()>
    where
        F: FnOnce(MemberBuilder) -> MemberBuilder,
    {
        let builder = f(MemberBuilder::default());

        if let Some(client_uuid) = builder.client_uuid {
            self.client_uuid = client_uuid;
        }

        if let Some(insertion_pending) = builder.insertion_pending {
            self.insertion_pending = insertion_pending;
        }

        let old_removal_pending = self.removal_pending;
        if let Some(removal_pending) = builder.removal_pending {
            self.removal_pending = removal_pending;
        }

        self.update(old_removal_pending).await
    }

    pub async fn update(&self, old_removal_pending: bool) -> SimpleResult<()> {
        sqlx::query!(
            r#"
            UPDATE ts
            SET
                client_uuid = ?,
                insertion_pending = ?,
                removal_pending = ?
            WHERE (user_id = ?) AND (removal_pending = ?)
            "#,
            self.client_uuid,
            self.insertion_pending,
            self.removal_pending,
            self.user_id,
            old_removal_pending,
        )
        .execute(&self.state.connection.connection)
        .await?;

        Ok(())
    }

    pub async fn destroy(self) -> SimpleResult<()> {
        sqlx::query!(
            r#"
            DELETE FROM ts
            WHERE (user_id = ?) AND (removal_pending = ?)
            "#,
            self.user_id,
            self.removal_pending,
        )
        .execute(&self.state.connection.connection)
        .await
        .map(|_| ())
        .map_err(Into::into)
    }
}

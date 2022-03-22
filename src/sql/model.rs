use crate::sql::{Inserted, NotInserted, Sql};
use crate::SimpleResult;

#[derive(Debug, Clone, Default)]
pub struct MemberBuilder {
    user_id: Option<i64>,
    client_uuid: Option<String>,
    insertion_pending: Option<bool>,
    removal_pending: Option<bool>,
}

impl MemberBuilder {
    pub fn new() -> Self {
        Default::default()
    }

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

impl Member<NotInserted> {
    pub async fn insert(self, sql: &Sql) -> SimpleResult<Member<Inserted>> {
        sqlx::query!(
            r#"
            INSERT INTO ts 
            (user_id, client_uuid, insertion_pending, removal_pending)
            VALUES (?, ?, ?, ?)"#,
            self.user_id,
            self.client_uuid,
            self.insertion_pending,
            self.removal_pending
        )
        .execute(&sql.connection)
        .await?;

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

        if let Some(removal_pending) = builder.removal_pending {
            self.removal_pending = removal_pending;
        }

        self.update().await
    }

    pub async fn update(&self) -> SimpleResult<()> {
        sqlx::query!(
            r#"
            UPDATE ts
            SET
                client_uuid = ?,
                insertion_pending = ?,
                removal_pending = ?
            WHERE user_id = ?
            "#,
            self.client_uuid,
            self.insertion_pending,
            self.removal_pending,
            self.user_id
        )
        .execute(&self.state.connection.connection)
        .await?;

        Ok(())
    }
}

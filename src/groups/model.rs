use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GroupMember {
    pub group_id: Uuid,
    pub user_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct GroupWithMember {
    pub id: Uuid,
    pub name: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub role: String,
    pub member_count: Option<i64>,
}

impl Group {
    pub async fn find_by_name(pool: &PgPool, name: &str) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>("SELECT * FROM project_groups WHERE name = $1")
            .bind(name)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<Option<Group>, sqlx::Error> {
        sqlx::query_as::<_, Group>("SELECT * FROM project_groups WHERE id = $1")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn list_by_user(
        pool: &PgPool,
        user_id: Uuid,
    ) -> Result<Vec<GroupWithMember>, sqlx::Error> {
        sqlx::query_as::<_, GroupWithMember>(
            r#"SELECT g.id, g.name, g.display_name, g.description, g.created_at, gm.role,
                      (SELECT COUNT(*) FROM group_members WHERE group_id = g.id) as member_count
               FROM project_groups g
               JOIN group_members gm ON g.id = gm.group_id
               WHERE gm.user_id = $1
               ORDER BY g.created_at DESC"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
    }
}

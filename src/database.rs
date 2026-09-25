#![allow(dead_code, unused_variables)]
use serde::Deserialize;

use anyhow::Result;

use sqlx::postgres::PgPool;
use sqlx::sqlite::SqlitePool;

#[derive(Deserialize)]
struct DatabaseConfig {
    #[serde(rename = "database_url")]
    url: String,
}

pub struct SqliteRepository {
    pool: SqlitePool,
}
impl SqliteRepository {
    async fn create_user(&self, login: String, password: String) -> Result<i32> {
        let row = sqlx::query!(
            "INSERT INTO users (login, password) VALUES (?, ?) RETURNING id",
            login,
            password
        )
        .fetch_one(&self.pool)
        .await?;
        Ok(row.ID as i32)
    }

    async fn find_user_with_login(&self, login: String) -> Result<Option<String>> {
        let row = sqlx::query!("SELECT password FROM users WHERE login = ?1", login)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.password))
    }
}

pub struct PostgresRepository {
    pool: PgPool,
}

pub enum DatabaseRepository {
    Sqlite(SqliteRepository),

    Postgres(PostgresRepository),
}

impl DatabaseRepository {
    pub async fn create_user(&self, login: String, password: String) -> Result<i32> {
        match self {
            DatabaseRepository::Sqlite(database) => database.create_user(login, password).await,
            DatabaseRepository::Postgres(database) => {
                todo!("Сделать postgres")
            }
        }
    }

    pub async fn find_user_with_login(&self, login: String) -> Result<Option<String>> {
        match self {
            DatabaseRepository::Sqlite(database) => database.find_user_with_login(login).await,
            DatabaseRepository::Postgres(database) => {
                todo!("Сделать postgres")
            }
        }
    }
}

pub struct DatabaseFactory;

impl DatabaseFactory {
    pub async fn init() -> Result<DatabaseRepository> {
        let config = envy::from_env::<DatabaseConfig>()
            .expect("Missing or invalid arg DATABASE_URL in .env");

        if config.url.starts_with("sqlite:") {
            let pool = SqlitePool::connect(&config.url).await?;

            sqlx::migrate!("migrations/sqlite").run(&pool).await?;

            let repository = SqliteRepository { pool };
            Ok(DatabaseRepository::Sqlite(repository))
        } else if config.url.starts_with("postgres:") || config.url.starts_with("postgresql:") {
            let pool = PgPool::connect(&config.url).await?;

            sqlx::migrate!("migrations/postgres").run(&pool).await?;

            let repository = PostgresRepository { pool };
            Ok(DatabaseRepository::Postgres(repository))
        } else {
            Err(anyhow::anyhow!("Unsupported DBMS, use Sqlite3 or Postgres"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    // Помощник для создания репозитория в памяти для каждого теста
    async fn setup_sqlite_repository() -> SqliteRepository {
        // Используем базу данных в памяти, чтобы тесты были изолированными и быстрыми
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Не удалось подключиться к SQLite в памяти");

        // Создаем таблицу пользователей вручную (вместо запуска миграций для простоты)
        sqlx::query(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                login TEXT NOT NULL UNIQUE,
                password TEXT NOT NULL
            );",
        )
        .execute(&pool)
        .await
        .expect("Не удалось создать тестовую таблицу");

        SqliteRepository { pool }
    }

    #[tokio::test]
    async fn test_create_user_success() {
        let repo = setup_sqlite_repository().await;

        // Проверяем создание пользователя
        let user_id = repo
            .create_user("alice".to_string(), "secret123".to_string())
            .await;

        assert!(user_id.is_ok());
        assert_eq!(user_id.unwrap(), 1); // Первый ID должен быть 1
    }

    #[tokio::test]
    async fn test_find_user_with_login_exists() {
        let repo = setup_sqlite_repository().await;
        let login = "bob".to_string();
        let password = "password_hash".to_string();

        // Сначала создаем пользователя
        repo.create_user(login.clone(), password.clone())
            .await
            .unwrap();

        // Пытаемся его найти
        let found_password = repo.find_user_with_login(login).await.unwrap();

        assert!(found_password.is_some());
        assert_eq!(found_password.unwrap(), password);
    }

    #[tokio::test]
    async fn test_find_user_with_login_not_found() {
        let repo = setup_sqlite_repository().await;

        // Ищем пользователя, которого нет в базе
        let found_password = repo
            .find_user_with_login("non_existent".to_string())
            .await
            .unwrap();

        assert!(found_password.is_none());
    }

    #[tokio::test]
    async fn test_create_user_duplicate_login_error() {
        let repo = setup_sqlite_repository().await;
        let login = "charlie".to_string();

        // Создаем первого пользователя
        repo.create_user(login.clone(), "pass1".to_string())
            .await
            .unwrap();

        // Попытка создать дубликат должна вернуть ошибку (из-за UNIQUE констрейнта)
        let result = repo.create_user(login, "pass2".to_string()).await;

        assert!(result.is_err());
    }
}

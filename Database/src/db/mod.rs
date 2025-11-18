use sqlx::{SqlitePool, Row};
use std::time::Duration;

pub async fn init_pool(url: &str) -> Result<SqlitePool, Box<dyn std::error::Error + Send + Sync>> {
    let pool = SqlitePool::connect(url).await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sqlx::query("PRAGMA foreign_keys = ON;").execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Users (
            UserID INTEGER PRIMARY KEY AUTOINCREMENT,
            Password TEXT NOT NULL,
            Nickname TEXT NOT NULL,
            GroupID INTEGER,
            GroupPermission INTEGER NOT NULL DEFAULT 0,
            IsActive INTEGER NOT NULL DEFAULT 1,
            UnfreezeDateTime TEXT,
            CreatedAt TEXT NOT NULL DEFAULT (DATETIME('now'))
        );"
    ).execute(pool).await?;

    let _ = sqlx::query("ALTER TABLE Users ADD COLUMN IsAdmin INTEGER NOT NULL DEFAULT 0;").execute(pool).await;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Groups (
            GroupID INTEGER PRIMARY KEY AUTOINCREMENT,
            GroupName TEXT NOT NULL UNIQUE,
            Description TEXT,
            Status INTEGER NOT NULL DEFAULT 0,
            CreatedTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            CreatedByUserID INTEGER,
            FOREIGN KEY(CreatedByUserID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS UserGroups (
            UserGroupID INTEGER PRIMARY KEY AUTOINCREMENT,
            UserID INTEGER NOT NULL,
            GroupID INTEGER NOT NULL,
            GroupPermission INTEGER NOT NULL DEFAULT 0,
            JoinTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            Status INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(UserID) REFERENCES Users(UserID),
            FOREIGN KEY(GroupID) REFERENCES Groups(GroupID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Tasks (
            TaskID INTEGER PRIMARY KEY AUTOINCREMENT,
            GroupID INTEGER NOT NULL,
            PublisherID INTEGER NOT NULL,
            Title TEXT NOT NULL,
            Content TEXT,
            PublishTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            Deadline TEXT,
            IsValid INTEGER NOT NULL DEFAULT 1,
            FOREIGN KEY(GroupID) REFERENCES Groups(GroupID),
            FOREIGN KEY(PublisherID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Entries (
            EntryID INTEGER PRIMARY KEY AUTOINCREMENT,
            TaskID INTEGER NOT NULL,
            SubmitterID INTEGER NOT NULL,
            Summary TEXT NOT NULL,
            Content TEXT,
            SubmitTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            AuditStatus INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(TaskID) REFERENCES Tasks(TaskID),
            FOREIGN KEY(SubmitterID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS AuditEntries (
            AuditEntryID INTEGER PRIMARY KEY AUTOINCREMENT,
            EntryID INTEGER NOT NULL,
            AuditorID INTEGER NOT NULL,
            AuditTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            AuditResult INTEGER NOT NULL,
            Description TEXT,
            FOREIGN KEY(EntryID) REFERENCES Entries(EntryID),
            FOREIGN KEY(AuditorID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Notifications (
            NotificationID INTEGER PRIMARY KEY AUTOINCREMENT,
            PublisherID INTEGER NOT NULL,
            GroupID INTEGER NOT NULL DEFAULT 0,
            PublishTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            Title TEXT NOT NULL,
            Content TEXT,
            IsPinned INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY(PublisherID) REFERENCES Users(UserID),
            FOREIGN KEY(GroupID) REFERENCES Groups(GroupID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS NotificationConfirmations (
            ConfirmationID INTEGER PRIMARY KEY AUTOINCREMENT,
            NotificationID INTEGER NOT NULL,
            UserID INTEGER NOT NULL,
            ConfirmTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            FOREIGN KEY(NotificationID) REFERENCES Notifications(NotificationID),
            FOREIGN KEY(UserID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Attachments (
            AttachmentID INTEGER PRIMARY KEY AUTOINCREMENT,
            GroupID INTEGER NOT NULL,
            OwnerType TEXT NOT NULL,
            OwnerID INTEGER NOT NULL,
            UploaderID INTEGER NOT NULL,
            UploadTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            FOREIGN KEY(GroupID) REFERENCES Groups(GroupID),
            FOREIGN KEY(UploaderID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS AuditLogs (
            LogID INTEGER PRIMARY KEY AUTOINCREMENT,
            UserID INTEGER,
            Action TEXT NOT NULL,
            TableName TEXT,
            ActionTime TEXT NOT NULL DEFAULT (DATETIME('now')),
            FOREIGN KEY(UserID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS Sessions (
            Token TEXT PRIMARY KEY,
            UserID INTEGER NOT NULL,
            Expires TEXT,
            FOREIGN KEY(UserID) REFERENCES Users(UserID)
        );"
    ).execute(pool).await?;

    Ok(())
}
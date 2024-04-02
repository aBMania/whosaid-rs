use sea_orm::DbErr;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("database error: {0}")]
    DbError(#[from] DbErr),
}
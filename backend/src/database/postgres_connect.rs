#[macro_export]
macro_rules! init_databases {
    (
        default: [$($default_type:ty),* $(,)?],
        pgvector: [$($pgvector_type:ty),* $(,)?]
    ) => {
        use $crate::database::{SqlxSchema, SchemaMigrator};
        use sqlx::postgres::PgPoolOptions;

        const MIN_POOL_CONN: u32 = 5;
        const MAX_POOL_CONN: u32 = 500;

        // --- Default Pool Setup ---
        static POOL: tokio::sync::OnceCell<sqlx::PgPool> = tokio::sync::OnceCell::const_new();

        async fn connect(drop_tables: bool, create_tables: bool, run_migrations: bool) -> &'static sqlx::PgPool {
            POOL.get_or_init(|| async {
                let database_url = std::env::var("DATABASE_URL")
                    .expect("DATABASE_URL environment variable not set");
                
                let pool = PgPoolOptions::new()
                    .max_connections(MAX_POOL_CONN)
                    .min_connections(MIN_POOL_CONN)
                    .connect(&database_url).await
                    .expect("Failed to connect to default database");

                if drop_tables {

                    panic!("drop_tables is true");
                }

                if create_tables {
                    let trigger_func_sql = r#"
                    CREATE OR REPLACE FUNCTION set_updated_at_unix_timestamp()
                    RETURNS TRIGGER AS $$
                    BEGIN NEW.updated_at = floor(extract(epoch from now())); RETURN NEW; END;
                    $$ language 'plpgsql';
                    "#;
                    sqlx::query(trigger_func_sql).execute(&pool).await
                        .expect("Failed to create timestamp helper function.");

                    $( 
                        let create_table_sql_str = <$default_type as $crate::SqlxSchema>::create_table_sql();
                        if !create_table_sql_str.trim().is_empty() {
                            sqlx::query(&create_table_sql_str).execute(&pool).await
                                .unwrap_or_else(|e| panic!("Failed to create table for '{}'. Error: {:?}", stringify!($default_type), e));
                        }
                    )*

                    $( 
                        let trigger_sql_str = <$default_type as $crate::SqlxSchema>::trigger_sql();
                        if !trigger_sql_str.trim().is_empty() {
                            for statement in trigger_sql_str.split(';').filter(|s| !s.trim().is_empty()) {
                                sqlx::query(statement).execute(&pool).await
                                    .unwrap_or_else(|e| panic!("Failed to execute trigger for '{}'. SQL: {}. Error: {:?}", stringify!($default_type), statement, e));
                            }
                        }
                    )*

                    $(
                        for index_sql in <$default_type as $crate::SqlxSchema>::INDEXES_SQL {
                            sqlx::query(index_sql).execute(&pool).await
                                .unwrap_or_else(|e| panic!("Failed to create index for '{}'. SQL: {}. Error: {:?}", stringify!($default_type), index_sql, e));
                        }
                    )*
                }

                if run_migrations {
                    $(
                        if let Err(e) = <$default_type as SchemaMigrator>::migrate(&pool).await {
                            eprintln!("[MIGRATE][ERROR] Failed to migrate '{}'. Error: {:?}", stringify!($default_type), e);
                        }
                    )*
                }

                pool
            }).await
        }

        // --- Pgvector Pool Setup ---
        static PGVECTOR_POOL: tokio::sync::OnceCell<sqlx::PgPool> = tokio::sync::OnceCell::const_new();

        async fn connect_pgvector(drop_tables: bool, create_tables: bool, run_migrations: bool) -> &'static sqlx::PgPool {
            PGVECTOR_POOL.get_or_init(|| async {
                let database_url = std::env::var("PGVECTOR_URI")
                    .expect("PGVECTOR_URI environment variable not set");
                
                let pool = PgPoolOptions::new()
                    .max_connections(MAX_POOL_CONN)
                    .min_connections(MIN_POOL_CONN)
                    .connect(&database_url).await
                    .expect("Failed to connect to pgvector database");

                sqlx::query("CREATE EXTENSION IF NOT EXISTS vector").execute(&pool).await
                    .expect("Failed to create vector extension.");

                if drop_tables {
                    panic!("drop_tables is true");
                }

                if create_tables {
                     let trigger_func_sql = r#"
                    CREATE OR REPLACE FUNCTION set_updated_at_unix_timestamp()
                    RETURNS TRIGGER AS $$
                    BEGIN NEW.updated_at = floor(extract(epoch from now())); RETURN NEW; END;
                    $$ language 'plpgsql';
                    "#;
                    sqlx::query(trigger_func_sql).execute(&pool).await
                        .expect("Failed to create timestamp helper function.");
                        
                    $( 
                        let create_table_sql_str = <$pgvector_type as $crate::SqlxSchema>::create_table_sql();
                        if !create_table_sql_str.trim().is_empty() {
                            sqlx::query(&create_table_sql_str).execute(&pool).await
                                .unwrap_or_else(|e| panic!("Failed to create table for '{}'. Error: {:?}", stringify!($pgvector_type), e));
                        }
                    )*

                    $( 
                        let trigger_sql_str = <$pgvector_type as $crate::SqlxSchema>::trigger_sql();
                        if !trigger_sql_str.trim().is_empty() {
                            for statement in trigger_sql_str.split(';').filter(|s| !s.trim().is_empty()) {
                                sqlx::query(statement).execute(&pool).await
                                    .unwrap_or_else(|e| panic!("Failed to execute trigger for '{}'. SQL: {}. Error: {:?}", stringify!($pgvector_type), statement, e));
                            }
                        }
                    )*

                    $(
                        for index_sql in <$pgvector_type as $crate::SqlxSchema>::INDEXES_SQL {
                            sqlx::query(index_sql).execute(&pool).await
                                .unwrap_or_else(|e| panic!("Failed to create index for '{}'. SQL: {}. Error: {:?}", stringify!($pgvector_type), index_sql, e));
                        }
                    )*
                }

                if run_migrations {
                    $(
                        if let Err(e) = <$pgvector_type as SchemaMigrator>::migrate(&pool).await {
                            eprintln!("[MIGRATE][ERROR] Failed to migrate '{}'. Error: {:?}", stringify!($pgvector_type), e);
                        }
                    )*
                }

                pool
            }).await
        }
    };
}

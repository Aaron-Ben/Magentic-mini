use std::env;

use crate::common::EnvVars;

pub struct PostgresDbEnv {
    pub postgres_url: String,
    pub pgvector_url: String,
}

impl EnvVars for PostgresDbEnv {
    fn load() -> Self {
        Self {
            postgres_url: env::var("DATABASE_URL").unwrap(),
            pgvector_url: env::var("PGVECTOR_URI").unwrap(),
        }
    }

    fn get_env_var(&self, key: &str) -> String {
        match key {
            "DATABASE_URL" => self.postgres_url.clone(),
            "PGVECTOR_URI" => self.pgvector_url.clone(),
            _ => panic!("Invalid environment variable: {}", key),
        }
    }
}
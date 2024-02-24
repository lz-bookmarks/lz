//! Database bindings and models for the `lz` bookmark manager

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

/// Run the migrations to bring up `lz` on the given DB connection.
pub fn run_migrations<C: refinery::Migrate>(
    conn: &mut C,
) -> Result<refinery::Report, refinery::Error> {
    embedded::migrations::runner().run(conn)
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use rusqlite::Connection;

    #[test]
    fn test_migrations_in_memory() -> Result<()> {
        let mut conn = Connection::open_in_memory()?;
        let report = crate::run_migrations(&mut conn)?;
        println!("Ran migrations: {:?}", report.applied_migrations());
        assert!(report.applied_migrations().len() > 0);
        Ok(())
    }
}

// Shortwave - connection.rs
// Copyright (C) 2021-2022  Felix Häcker <haeckerfelix@gnome.org>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// Based on gnome-podcasts by Jordan Petridis
// https://gitlab.gnome.org/World/podcasts/blob/cf644d508d8d7dab3c9357d12b1262ae6b44c8e8/podcasts-data/src/database.rs

use std::io;
use std::path::PathBuf;

use diesel::prelude::*;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use once_cell::sync::Lazy;

use crate::{config, path};

// Read database migrations
embed_migrations!("./data/database/migrations/");

// Define 'Pool' type
type Pool = r2d2::Pool<ConnectionManager<SqliteConnection>>;

// Database path
pub static DB_PATH: Lazy<PathBuf> = Lazy::new(|| {
    let mut path = path::DATA.clone();
    path.push(format!("{}.db", config::NAME));
    path
});

// Database R2D2 connection pool
static POOL: Lazy<Pool> = Lazy::new(|| init_connection_pool(DB_PATH.to_str().unwrap()));

// Returns a R2D2 SqliteConnection
pub fn connection() -> Pool {
    POOL.clone()
}

// Inits database connection pool, and run migrations.
// If there's no database, it get's created automatically.
fn init_connection_pool(db_path: &str) -> Pool {
    let manager = ConnectionManager::<SqliteConnection>::new(db_path);
    let pool = r2d2::Pool::builder()
        .max_size(1)
        .build(manager)
        .expect("Failed to create pool.");

    let db = pool.get().expect("Failed to initialize pool.");
    run_migrations(&db).expect("Failed to run migrations during init.");

    info!("Initialized database connection pool.");
    pool
}

fn run_migrations(
    connection: &SqliteConnection,
) -> Result<(), diesel::migration::RunMigrationsError> {
    info!("Running DB Migrations...");
    embedded_migrations::run_with_output(connection, &mut io::stdout()).map_err(From::from)
}

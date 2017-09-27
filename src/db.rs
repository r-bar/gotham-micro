use r2d2;
use r2d2_diesel;
use gotham_middleware_diesel::state_data::Diesel;
#[allow(unused_imports)]
use diesel::pg::PgConnection;
#[allow(unused_imports)]
use diesel::sqlite::SqliteConnection;
#[allow(unused_imports)]
use diesel::mysql::MysqlConnection;
use gotham::state::State;

#[cfg(feature = "postgres")]
pub fn connect(
    state: &mut State,
) -> Result<r2d2::PooledConnection<r2d2_diesel::ConnectionManager<PgConnection>>, r2d2::GetTimeout> {
    state.take::<Diesel<PgConnection>>().conn()
}

#[cfg(feature = "sqlite")]
pub fn connect(
    state: State,
) -> Result<r2d2::PooledConnection<SqliteConnection>, r2d2::GetTimeout> {
    state.take::<Diesel<SqliteConnection>>().conn()
}

#[cfg(feature = "mysql")]
pub fn connect(
    state: State,
) -> Result<r2d2::PooledConnection<MysqlConnection>, r2d2::GetTimeout> {
    state.take::<Diesel<MysqlConnection>>().conn()
}

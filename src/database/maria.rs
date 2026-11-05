use async_trait::async_trait;
use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as B64;
use mysql_async::{Conn, Error, Opts, Params, Pool, Row, UrlError, Value};
use mysql_async::{QueryResult, prelude::*};

use super::{Cell, DataParam, DatabaseError, Executor, QueryOutput, Schema};

const SYSTEM_SCHEMAS: [&str; 4] = ["information_schema", "mysql", "performance_schema", "sys"];

fn is_system_schema(name: &str) -> bool {
    SYSTEM_SCHEMAS.iter().any(|s| s.eq_ignore_ascii_case(name))
}

fn quote_ident(name: &str) -> String {
    return format!("`{}`", name.replace('`', "``"));
}

// implement transforming `mysql_async::Error` to `DatabaseError`
impl From<Error> for DatabaseError {
    fn from(e: Error) -> Self {
        match e {
            Error::Server(s) => DatabaseError::Server {
                code: s.code,
                state: s.state,
                message: s.message,
            },
            Error::Io(e) => DatabaseError::Connection {
                message: e.to_string(),
            },
            _ => DatabaseError::Internal {
                message: e.to_string(),
            },
        }
    }
}

// implement transforming `mysql_async::Error` to `DatabaseError`
impl From<UrlError> for DatabaseError {
    fn from(e: UrlError) -> Self {
        DatabaseError::Config {
            message: e.to_string(),
        }
    }
}

/// Convert a parameter to a value
fn param_to_value(p: &DataParam) -> Result<Value, DatabaseError> {
    /* to whoever on Twitter said that use statements in functions is a symbol
     * of AI: shut up, I'm not writing `Param::foo` a hundred times for one match
     * statement */
    use DataParam::{Bool, Bytes, Float, Int, Text, UInt};

    let value = match p {
        DataParam::Null => Value::NULL, // OK Rust got mad when I just wrote `Null`
        Bool(b) => Value::Int(i64::from(*b)),
        Int(i) => Value::Int(*i),
        UInt(u) => Value::UInt(*u),
        Float(f) => Value::Float(*f as f32),
        Text(t) => Value::Bytes(t.clone().into_bytes()),
        // sort of a catch-all
        Bytes(b64) => Value::Bytes(B64.decode(b64).map_err(|e| DatabaseError::InvalidParam {
            message: format!("invalid base64: {e}"),
        })?),
    };

    Ok(value)
}

fn value_to_cell(v: Value) -> Cell {
    match v {
        Value::NULL => Cell::Null,
        Value::Int(i) => Cell::Int(i),
        Value::UInt(u) => Cell::UInt(u),
        Value::Float(f) => Cell::Float(f64::from(f)),
        Value::Double(d) => Cell::Float(d),
        Value::Bytes(b) => match String::from_utf8(b) {
            Ok(s) => Cell::Text(s),
            Err(e) => Cell::Bytes(B64.encode(e.into_bytes())),
        },
        // Only appear over the binary (prepared) protocol.
        Value::Date(y, mo, d, h, mi, s, us) => {
            let mut t = format!("{y:04}-{mo:02}-{d:02} {h:02}:{mi:02}:{s:02}");
            if us != 0 {
                t.push_str(&format!(".{us:06}"));
            }
            Cell::Text(t)
        }
        Value::Time(neg, days, h, m, s, us) => {
            let hours = days * 24 + u32::from(h);
            let sign = if neg { "-" } else { "" };
            let mut t = format!("{sign}{hours:02}:{m:02}:{s:02}");
            if us != 0 {
                t.push_str(&format!(".{us:06}"));
            }
            Cell::Text(t)
        }
    }
}

fn to_output(columns: &[String], rows: &[Row], affected_rows: u64) -> QueryOutput {
    // closure for mapping a single row
    let row_to_cell = |mut row: Row| {
        (0..row.len())
            .map(|i| row.take::<Value, _>(i).map_or(Cell::Null, value_to_cell))
            .collect()
    };

    QueryOutput {
        columns: Vec::from(columns),
        rows: Vec::from(rows).into_iter().map(row_to_cell).collect(),
        affected_rows,
    }
}

/// Helper function to reduce repetitive code. Used in the `Executor` implementation
/// of the `MariaDB` struct.
async fn format_output(
    mut result: QueryResult<'_, 'static, impl Protocol>,
) -> Result<QueryOutput, DatabaseError> {
    let columns = column_names(result.columns_ref());
    let rows: Vec<Row> = result.collect().await?;
    let affected = result.affected_rows();
    result.drop_result().await?;

    Ok(to_output(&columns, &rows, affected))
}

fn column_names(cols: &[mysql_async::Column]) -> Vec<String> {
    cols.iter().map(|c| c.name_str().into_owned()).collect()
}

pub struct MariaDB {
    pool: Pool,
}
impl MariaDB {
    /// Create a new MariaDB instance with a url to the MariaDB connection, e.g.
    /// `mysql://root:pw@127.0.0.1:3306`
    pub fn new(url: &str) -> Result<Self, DatabaseError> {
        Ok(Self {
            pool: Pool::new(Opts::from_url(url)?),
        })
    }

    /// Shuts down the existing MariaDB instance
    pub async fn shutdown(self) -> Result<(), DatabaseError> {
        Ok(self.pool.disconnect().await?)
    }

    /// Checks out a connection, selecting `db` first if given (per-call USE)
    async fn conn(&self, db: Option<&str>) -> Result<Conn, DatabaseError> {
        let mut conn = self.pool.get_conn().await?;
        if let Some(db) = db {
            conn.query_drop(format!("USE {}", quote_ident(db))).await?;
        }
        Ok(conn)
    }
}
/// Implementing the `Executor` trait to allow MariaDB instances to execute queries
#[async_trait]
impl Executor for MariaDB {
    async fn execute_query(
        &self,
        db_name: Option<&str>,
        query: &str,
        params: &[DataParam],
    ) -> Result<QueryOutput, DatabaseError> {
        let values = params
            .iter()
            .map(param_to_value)
            .collect::<Result<Vec<_>, _>>()?;

        let mut conn = self.conn(db_name).await?;
        let result = conn.exec_iter(query, Params::from(values)).await?;

        format_output(result).await
    }

    async fn execute_raw(
        &self,
        db_name: Option<&str>,
        query: &str,
    ) -> Result<QueryOutput, DatabaseError> {
        let mut conn = self.conn(db_name).await?;
        let result = conn.query_iter(query).await?;

        format_output(result).await
    }

    fn list_schemas(&self) -> Result<&[Schema], DatabaseError> {
        todo!();
    }

    fn get_schema(&self, name: &str) -> Result<Option<Schema>, DatabaseError> {
        todo!();
    }
}

//! All public-facing schema and tables in the code repo.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod maria;

/// Engine-agnostic, serializable error. All variants are represented as a struct,
/// allowing serde's `kind` tag to work so that data can be send in HTTP response
/// bodies.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DatabaseError {
    #[error("server error {code} ({state}): {message}")]
    Server {
        code: u16,
        state: String,
        message: String,
    },
    #[error("connection error: {message}")]
    Connection { message: String },
    #[error("invalid parameter: {message}")]
    InvalidParam { message: String },
    #[error("invalid configuration: {message}")]
    Config { message: String },
    #[error("internal error: {message}")]
    Internal { message: String },
}

/// A bound parameter for a prepared statement.
/// Dates, times and decimals go as `Text`; MariaDB converts them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum DataParam {
    Null,
    Bool(bool),
    Int(i64),
    UInt(u64),
    Float(f64),
    Text(String),
    /// Base64-encoded bytes.
    Bytes(String),
}

/// An overview of a single database. Allows for `Table` structs so the webclient
/// to render the tree available.
///
/// Note that a Schema does not contain any actual data; to get actual data,
/// refer to the `Executor` trait's `execute_query()` implementation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Schema {
    pub name: String,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub kind: TableKind,
    pub columns: Vec<Column>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TableKind {
    Table,
    View,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    /// Full column type, e.g. `varchar(255)`.
    pub data_type: String,
    pub nullable: bool,
    pub is_primary_key: bool,
}

/// A single result cell
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Cell {
    Null,
    Int(i64),
    UInt(u64),
    Float(f64),
    Text(String),
    /// Holds binary data that isn't valid UTF-8 in base64
    Bytes(String),
}

/// The output of a query. Returned in the `execute_query()` function of the
/// `Executor` trait.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct QueryOutput {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<Cell>>,
    pub affected_rows: u64,
}

/// Execute database operations on behalf of the caller. While Lady was originally
/// built with MariaDB in mind, this trait exists to create an abstraction to
/// allow for potential implementations of other SQL database systems.
#[async_trait]
pub trait Executor: Send + Sync {
    /// Runs a single prepared statement with bound parameters.
    async fn execute_query(
        &self,
        db_name: Option<&str>,
        query: &str,
        params: &[DataParam],
    ) -> Result<QueryOutput, DatabaseError>;

    /// Run raw SQL over the text protocol, with no parameters. Unlike a prepared
    /// statement, this accepts multi-statement text; only the first result set
    /// is returned.
    async fn execute_raw(
        &self,
        db_name: Option<&str>,
        query: &str,
    ) -> Result<QueryOutput, DatabaseError>;

    fn list_schemas(&self) -> Result<&[Schema], DatabaseError>;

    fn get_schema(&self, name: &str) -> Result<Option<Schema>, DatabaseError>;
}

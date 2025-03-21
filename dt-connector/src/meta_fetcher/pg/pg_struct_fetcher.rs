use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::bail;
use dt_common::meta::struct_meta::{
    statement::{
        pg_create_schema_statement::PgCreateSchemaStatement,
        pg_create_table_statement::PgCreateTableStatement,
    },
    structure::{
        column::{Column, ColumnDefault},
        comment::{Comment, CommentType},
        constraint::{Constraint, ConstraintType},
        index::{Index, IndexKind},
        schema::Schema,
        sequence::Sequence,
        sequence_owner::SequenceOwner,
        table::Table,
    },
};
use dt_common::{
    config::{config_enums::DbType, config_token_parser::ConfigTokenParser},
    error::Error,
    log_error, log_info, log_warn,
    rdb_filter::RdbFilter,
    utils::sql_util::SqlUtil,
};
use futures::TryStreamExt;
use sqlx::{postgres::PgRow, Pool, Postgres, Row};

use super::pg_struct_check_fetcher::PgStructCheckFetcher;

pub struct PgStructFetcher {
    pub conn_pool: Pool<Postgres>,
    pub schema: String,
    pub filter: Option<RdbFilter>,
}

enum ColType {
    Text,
    Char,
}

impl PgStructFetcher {
    pub async fn get_create_schema_statement(&mut self) -> anyhow::Result<PgCreateSchemaStatement> {
        let schema = self.get_schema().await?;
        Ok(PgCreateSchemaStatement { schema })
    }

    pub async fn get_create_table_statements(
        &mut self,
        tb: &str,
    ) -> anyhow::Result<Vec<PgCreateTableStatement>> {
        let mut results = Vec::new();

        let tables = self.get_tables(tb).await?;
        let mut sequences = self.get_sequences(tb).await?;
        let mut sequence_owners = self.get_sequence_owners(tb).await?;
        let mut constraints = self.get_constraints(tb).await?;
        let mut indexes = self.get_indexes(tb).await?;
        let mut column_comments = self.get_column_comments(tb).await?;
        let mut table_comments = self.get_table_comments(tb).await?;

        for (table_name, table) in tables {
            let table_sequences = self.get_table_sequences(&table, &mut sequences).await?;
            let statement = PgCreateTableStatement {
                table,
                sequences: table_sequences,
                sequence_owners: self.get_result(&mut sequence_owners, &table_name),
                constraints: self.get_result(&mut constraints, &table_name),
                indexes: self.get_result(&mut indexes, &table_name),
                column_comments: self.get_result(&mut column_comments, &table_name),
                table_comments: self.get_result(&mut table_comments, &table_name),
            };
            results.push(statement);
        }
        Ok(results)
    }

    async fn get_schema(&mut self) -> anyhow::Result<Schema> {
        let sql = format!(
            "SELECT schema_name 
            FROM information_schema.schemata
            WHERE schema_name='{}'",
            self.schema
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        if let Some(row) = rows.try_next().await? {
            let schema_name = Self::get_str_with_null(&row, "schema_name")?;
            let schema = Schema { name: schema_name };
            return Ok(schema);
        }

        bail! {Error::StructError(format!(
            "schema: {} not found",
            self.schema
        ))}
    }

    async fn get_sequences(&mut self, tb: &str) -> anyhow::Result<HashMap<String, Vec<Sequence>>> {
        let mut results: HashMap<String, Vec<Sequence>> = HashMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND tab.relname = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT obj.sequence_catalog,
                obj.sequence_schema,
                tab.relname AS table_name,
                obj.sequence_name,
                obj.data_type,
                obj.start_value,
                obj.minimum_value,
                obj.maximum_value,
                obj.increment,
                obj.cycle_option
            FROM information_schema.sequences obj
            JOIN pg_class AS seq
                ON (seq.relname = obj.sequence_name)
            JOIN pg_namespace ns
                ON (seq.relnamespace = ns.oid)
            JOIN pg_depend AS dep
                ON (seq.oid = dep.objid)
            JOIN pg_class AS tab
                ON (dep.refobjid = tab.oid)
            WHERE ns.nspname='{}' 
            AND obj.sequence_schema='{}' {} 
            AND dep.deptype='a'",
            &self.schema, &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let (sequence_schema, table_name, sequence_name): (String, String, String) = (
                Self::get_str_with_null(&row, "sequence_schema")?,
                Self::get_str_with_null(&row, "table_name")?,
                Self::get_str_with_null(&row, "sequence_name")?,
            );

            let sequence = Sequence {
                sequence_name,
                database_name: Self::get_str_with_null(&row, "sequence_catalog")?,
                schema_name: sequence_schema,
                data_type: Self::get_str_with_null(&row, "data_type")?,
                start_value: row.get("start_value"),
                increment: row.get("increment"),
                minimum_value: row.get("minimum_value"),
                maximum_value: row.get("maximum_value"),
                cycle_option: Self::get_str_with_null(&row, "cycle_option")?,
            };
            self.push_to_results(&mut results, &table_name, sequence);
        }

        Ok(results)
    }

    async fn get_independent_sequences(
        &mut self,
        sequence_names: &[String],
    ) -> anyhow::Result<Vec<Sequence>> {
        let filter_names: Vec<String> = sequence_names.iter().map(|i| format!("'{}'", i)).collect();
        let filter = format!("AND sequence_name IN ({})", filter_names.join(","));
        let sql = format!(
            "SELECT *
            FROM information_schema.sequences
            WHERE sequence_schema='{}' {}",
            self.schema, filter
        );

        let mut results = Vec::new();
        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let sequence = Sequence {
                sequence_name: Self::get_str_with_null(&row, "sequence_name")?,
                database_name: Self::get_str_with_null(&row, "sequence_catalog")?,
                schema_name: Self::get_str_with_null(&row, "sequence_schema")?,
                data_type: Self::get_str_with_null(&row, "data_type")?,
                start_value: row.get("start_value"),
                increment: row.get("increment"),
                minimum_value: row.get("minimum_value"),
                maximum_value: row.get("maximum_value"),
                cycle_option: Self::get_str_with_null(&row, "cycle_option")?,
            };
            results.push(sequence)
        }

        Ok(results)
    }

    async fn get_table_sequences(
        &mut self,
        table: &Table,
        sequences: &mut HashMap<String, Vec<Sequence>>,
    ) -> anyhow::Result<Vec<Sequence>> {
        let mut table_sequences = self.get_result(sequences, &table.table_name);

        let mut owned_sequence_names = HashSet::new();
        for sequence in table_sequences.iter() {
            owned_sequence_names.insert(sequence.sequence_name.clone());
        }

        let mut independent_squence_names = Vec::new();
        for column in table.columns.iter() {
            if let Some(ColumnDefault::Literal(default_value)) = &column.column_default {
                let (schema, sequence_name) =
                    Self::get_sequence_name_by_default_value(default_value);
                // example, default_value is 'Standard'::text
                if sequence_name.is_empty() {
                    log_warn!(
                        "table: {}.{} has default value: {} for column: {}, not sequence",
                        table.schema_name,
                        table.table_name,
                        default_value,
                        column.column_name
                    );
                    continue;
                }

                // sequence and table should be in the same schema, otherwise we don't support
                if !schema.is_empty() && schema != self.schema {
                    log_error!(
                        "table: {}.{} is using sequence: {}.{} from a different schema",
                        table.schema_name,
                        table.table_name,
                        schema,
                        sequence_name
                    );
                    continue;
                }

                if owned_sequence_names.contains(&sequence_name) {
                    continue;
                }

                log_info!(
                    "table: {}.{} is using independent sequence: {}.{}",
                    table.schema_name,
                    table.table_name,
                    schema,
                    sequence_name
                );
                independent_squence_names.push(sequence_name);
            }
        }

        if !independent_squence_names.is_empty() {
            let independent_squences = self
                .get_independent_sequences(&independent_squence_names)
                .await?;
            table_sequences.extend_from_slice(&independent_squences);
        }

        Ok(table_sequences)
    }

    async fn get_sequence_owners(
        &mut self,
        tb: &str,
    ) -> anyhow::Result<HashMap<String, Vec<SequenceOwner>>> {
        let mut results = HashMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND tab.relname = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT seq.relname,
                tab.relname AS table_name,
                attr.attname AS column_name,
                ns.nspname
            FROM pg_class AS seq
            JOIN pg_namespace ns
                ON (seq.relnamespace = ns.oid)
            JOIN pg_depend AS dep
                ON (seq.oid = dep.objid)
            JOIN pg_class AS tab
                ON (dep.refobjid = tab.oid)
            JOIN pg_attribute AS attr
                ON (attr.attnum = dep.refobjsubid AND attr.attrelid = dep.refobjid)
            WHERE dep.deptype='a'
                AND seq.relkind='S'
                AND ns.nspname = '{}' {}",
            &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);

        while let Some(row) = rows.try_next().await? {
            let (schema_name, table_name, seq_name): (String, String, String) = (
                Self::get_str_with_null(&row, "nspname")?,
                Self::get_str_with_null(&row, "table_name")?,
                Self::get_str_with_null(&row, "relname")?,
            );

            let sequence_owner = SequenceOwner {
                sequence_name: seq_name,
                database_name: String::new(),
                schema_name,
                table_name: table_name.clone(),
                column_name: Self::get_str_with_null(&row, "column_name")?,
            };
            self.push_to_results(&mut results, &table_name, sequence_owner);
        }

        Ok(results)
    }

    async fn get_tables(&mut self, tb: &str) -> anyhow::Result<BTreeMap<String, Table>> {
        let mut results: BTreeMap<String, Table> = BTreeMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND c.table_name = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT c.table_schema,
                c.table_name,
                c.column_name,
                c.data_type,
                c.udt_name,
                c.character_maximum_length,
                c.is_nullable,
                c.column_default,
                c.numeric_precision,
                c.numeric_scale,
                c.is_identity,
                c.identity_generation,
                c.ordinal_position
            FROM information_schema.columns c
            JOIN information_schema.tables t 
                ON c.table_schema = t.table_schema 
                AND c.table_name = t.table_name
            WHERE c.table_schema ='{}' {} 
                AND t.table_type = 'BASE TABLE'
            ORDER BY c.table_schema, c.table_name, c.ordinal_position",
            &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let (table_schema, table_name) = (
                Self::get_str_with_null(&row, "table_schema")?,
                Self::get_str_with_null(&row, "table_name")?,
            );

            if self.filter_tb(&table_name) {
                continue;
            }

            let ordinal_position: i32 = row.try_get("ordinal_position")?;
            let is_identity = row.get("is_identity");
            let identity_generation = row.get("identity_generation");
            let generation_rule = Self::get_col_generation_rule(is_identity, identity_generation);
            let is_nullable = Self::get_str_with_null(&row, "is_nullable")?.to_lowercase() == "yes";
            let column_default = row
                .get::<Option<String>, _>("column_default")
                .map(ColumnDefault::Literal);
            let column = Column {
                column_name: Self::get_str_with_null(&row, "column_name")?,
                ordinal_position: ordinal_position as u32,
                column_default,
                is_nullable,
                generated: generation_rule,
                ..Default::default()
            };

            if let Some(table) = results.get_mut(&table_name) {
                table.columns.push(column);
            } else {
                results.insert(
                    table_name.clone(),
                    Table {
                        database_name: table_schema.clone(),
                        schema_name: table_schema,
                        table_name: table_name.clone(),
                        columns: vec![column],
                        ..Default::default()
                    },
                );
            }
        }

        // get column types
        for (table_name, table) in results.iter_mut() {
            let column_types = self.get_column_types(table_name).await?;
            for column in table.columns.iter_mut() {
                column.column_type = column_types.get(&column.column_name).unwrap().to_owned();
            }
        }

        Ok(results)
    }

    async fn get_column_types(&mut self, tb: &str) -> anyhow::Result<HashMap<String, String>> {
        let fetcher = PgStructCheckFetcher {
            conn_pool: self.conn_pool.clone(),
        };

        let oid = fetcher.get_oid(&self.schema, tb).await?;
        let sql = format!(
            "SELECT a.attname AS column_name, 
                pg_catalog.format_type(a.atttypid, a.atttypmod) AS column_type
            FROM pg_catalog.pg_attribute a
            WHERE a.attrelid = '{}' AND a.attnum > 0;",
            oid
        );

        let mut results = HashMap::new();
        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let column_name: String = Self::get_str_with_null(&row, "column_name")?;
            let column_type: String = Self::get_str_with_null(&row, "column_type")?;
            results.insert(column_name, column_type);
        }

        Ok(results)
    }

    async fn get_constraints(
        &mut self,
        tb: &str,
    ) -> anyhow::Result<HashMap<String, Vec<Constraint>>> {
        let mut results = HashMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND rel.relname = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT nsp.nspname,
                rel.relname,
                con.conname AS constraint_name,
                con.contype AS constraint_type,
                pg_get_constraintdef(con.oid) AS constraint_definition
            FROM pg_catalog.pg_constraint con
            JOIN pg_catalog.pg_class rel
                ON rel.oid = con.conrelid
            JOIN pg_catalog.pg_namespace nsp
                ON nsp.oid = connamespace
            WHERE nsp.nspname ='{}' {} 
            ORDER BY nsp.nspname,rel.relname",
            &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let table_name = Self::get_str_with_null(&row, "relname")?;
            let constraint_type = Self::get_with_null(&row, "constraint_type", ColType::Char)?;

            let constraint = Constraint {
                database_name: String::new(),
                schema_name: Self::get_str_with_null(&row, "nspname")?,
                table_name: table_name.clone(),
                constraint_name: Self::get_str_with_null(&row, "constraint_name")?,
                constraint_type: ConstraintType::from_str(&constraint_type, DbType::Pg),
                definition: Self::get_str_with_null(&row, "constraint_definition")?,
            };
            self.push_to_results(&mut results, &table_name, constraint);
        }

        Ok(results)
    }

    async fn get_indexes(&mut self, tb: &str) -> anyhow::Result<HashMap<String, Vec<Index>>> {
        let mut results = HashMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND tablename = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT schemaname,
                tablename,
                indexdef,
                COALESCE(tablespace, 'pg_default') AS tablespace, indexname
            FROM pg_indexes
            WHERE schemaname = '{}' {}",
            &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let table_name = Self::get_str_with_null(&row, "tablename")?;
            let definition = Self::get_str_with_null(&row, "indexdef")?;

            let index = Index {
                schema_name: Self::get_str_with_null(&row, "schemaname")?,
                table_name: table_name.clone(),
                index_name: Self::get_str_with_null(&row, "indexname")?,
                index_kind: self.get_index_kind(&definition),
                table_space: Self::get_str_with_null(&row, "tablespace")?,
                definition,
                ..Default::default()
            };
            self.push_to_results(&mut results, &table_name, index);
        }

        Ok(results)
    }

    async fn get_table_comments(
        &mut self,
        tb: &str,
    ) -> anyhow::Result<HashMap<String, Vec<Comment>>> {
        let mut results = HashMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND c.relname = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT n.nspname,
                c.relname,
                d.description
            FROM pg_class c
            LEFT JOIN pg_namespace n
                ON n.oid = c.relnamespace
            LEFT JOIN pg_description d
                ON c.oid = d.objoid  AND d.objsubid = 0
            WHERE n.nspname ='{}' {}
            AND d.description IS NOT null",
            &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let (schema_name, table_name): (String, String) = (
                Self::get_str_with_null(&row, "nspname")?,
                Self::get_str_with_null(&row, "relname")?,
            );

            let comment = Comment {
                comment_type: CommentType::Table,
                database_name: String::new(),
                schema_name,
                table_name: table_name.clone(),
                column_name: String::new(),
                comment: Self::get_str_with_null(&row, "description")?,
            };
            self.push_to_results(&mut results, &table_name, comment);
        }

        Ok(results)
    }

    async fn get_column_comments(
        &mut self,
        tb: &str,
    ) -> anyhow::Result<HashMap<String, Vec<Comment>>> {
        let mut results = HashMap::new();

        let tb_filter = if !tb.is_empty() {
            format!("AND c.relname = '{}'", tb)
        } else {
            String::new()
        };

        let sql = format!(
            "SELECT n.nspname,
                c.relname,
                col_description(a.attrelid, a.attnum) as comment,
                format_type(a.atttypid, a.atttypmod)as type,
                a.attname AS name,
                a.attnotnull AS notnull
            FROM pg_class c
            LEFT JOIN pg_attribute a
                ON a.attrelid =c.oid
            LEFT JOIN pg_namespace n
                ON n.oid = c.relnamespace
            WHERE n.nspname ='{}' {}
                AND a.attnum >0
                AND col_description(a.attrelid, a.attnum) is NOT null",
            &self.schema, tb_filter
        );

        let mut rows = sqlx::query(&sql).fetch(&self.conn_pool);
        while let Some(row) = rows.try_next().await? {
            let (schema_name, table_name, column_name) = (
                Self::get_str_with_null(&row, "nspname")?,
                Self::get_str_with_null(&row, "relname")?,
                Self::get_str_with_null(&row, "name")?,
            );

            let comment = Comment {
                comment_type: CommentType::Column,
                database_name: String::new(),
                schema_name,
                table_name: table_name.clone(),
                column_name,
                comment: Self::get_str_with_null(&row, "comment")?,
            };
            self.push_to_results(&mut results, &table_name, comment);
        }

        Ok(results)
    }

    fn get_index_kind(&self, definition: &str) -> IndexKind {
        if definition.starts_with("CREATE UNIQUE INDEX") {
            IndexKind::Unique
        } else {
            IndexKind::Unknown
        }
    }

    fn get_str_with_null(row: &PgRow, col_name: &str) -> anyhow::Result<String> {
        Self::get_with_null(row, col_name, ColType::Text)
    }

    fn get_with_null(row: &PgRow, col_name: &str, col_type: ColType) -> anyhow::Result<String> {
        let mut str_val = String::new();
        match col_type {
            ColType::Text => {
                let str_val_option: Option<String> = row.get(col_name);
                if let Some(s) = str_val_option {
                    str_val = s
                }
            }
            ColType::Char => {
                let char_val: i8 = row.get(col_name);
                str_val = char_val.to_string();
            }
        }
        Ok(str_val)
    }

    fn get_col_generation_rule(
        is_identity: Option<String>,
        identity_generation: Option<String>,
    ) -> Option<String> {
        if let Some(i) = is_identity {
            if i.to_lowercase() == "yes" && identity_generation.is_some() {
                return identity_generation;
            }
        }
        None
    }

    fn get_sequence_name_by_default_value(default_value: &str) -> (String, String) {
        // SELECT table_schema,
        //     table_name,
        //     column_name,
        //     column_default
        // FROM information_schema.columns
        // WHERE table_schema ='public' and table_name='sequence_test_4';

        // case 1: when search_path is the same with sequence schema, column_default be like:
        // nextval('"aaaaaaadefdfd.dsds::er3\ddd"'::regclass)

        // case 2: when search_path is not the same with sequence schema, column_default be like:
        // nextval('public."aaaaaaadefdfd.dsds::er3\ddd"'::regclass)
        // nextval('"ddddd.ddddddds**"."aaaaaaadefdfd.dsds::er3\ddd"'::regclass)

        let mut value = default_value.trim();
        if !value.starts_with("nextval(") {
            return (String::new(), String::new());
        }

        value = value
            .trim_start_matches("nextval(")
            .trim_start_matches('\'')
            .trim_end_matches(')')
            // ::regclass may not exists
            .trim_end_matches("::regclass")
            .trim_end_matches('\'');

        let escape_pair = SqlUtil::get_escape_pairs(&DbType::Pg)[0];
        if let Ok(tokens) = ConfigTokenParser::parse_config(value, &DbType::Pg, &['.']) {
            if tokens.len() == 1 {
                return (String::new(), SqlUtil::unescape(&tokens[0], &escape_pair));
            } else if tokens.len() == 2 {
                return (
                    SqlUtil::unescape(&tokens[0], &escape_pair),
                    SqlUtil::unescape(&tokens[1], &escape_pair),
                );
            }
        }
        (String::new(), String::new())
    }

    fn filter_tb(&mut self, tb: &str) -> bool {
        if let Some(filter) = &mut self.filter {
            return filter.filter_tb(&self.schema, tb);
        }
        false
    }

    fn push_to_results<T>(
        &mut self,
        results: &mut HashMap<String, Vec<T>>,
        table_name: &str,
        item: T,
    ) {
        if self.filter_tb(table_name) {
            return;
        }

        if let Some(exists) = results.get_mut(table_name) {
            exists.push(item);
        } else {
            results.insert(table_name.into(), vec![item]);
        }
    }

    fn get_result<T>(&self, results: &mut HashMap<String, Vec<T>>, table_name: &str) -> Vec<T> {
        results.remove(table_name).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::meta_fetcher::pg::pg_struct_fetcher::PgStructFetcher;

    #[test]
    fn get_sequence_name_by_default_value_test() {
        let default_values = [
            r#"('"aaaaaaadefdfd.dsds::er3\ddd"'::regclass)"#,
            r#"nextval('aaaaaaaaaa'::regclass)"#,
            r#"nextval('public.aaaaaaaaaa'::regclass)"#,
            r#"nextval('"aaaaaaadefdfd.dsds::er3\ddd"'::regclass)"#,
            r#"nextval('public."aaaaaaadefdfd.dsds::er3\ddd"'::regclass)"#,
            r#"nextval('"ddddd.ddddddds**"."aaaaaaadefdfd.dsds::er3\ddd"'::regclass)"#,
            r#"nextval('"aaaaaaadefdfd.dsds::er3\ddd"')"#,
            r#"nextval('public."aaaaaaadefdfd.dsds::er3\ddd"')"#,
            r#"nextval('"ddddd.ddddddds**"."aaaaaaadefdfd.dsds::er3\ddd"')"#,
        ];

        let expect_sequences = vec![
            ("", ""),
            ("", "aaaaaaaaaa"),
            ("public", "aaaaaaaaaa"),
            ("", r#"aaaaaaadefdfd.dsds::er3\ddd"#),
            ("public", r#"aaaaaaadefdfd.dsds::er3\ddd"#),
            ("ddddd.ddddddds**", r#"aaaaaaadefdfd.dsds::er3\ddd"#),
            ("", r#"aaaaaaadefdfd.dsds::er3\ddd"#),
            ("public", r#"aaaaaaadefdfd.dsds::er3\ddd"#),
            ("ddddd.ddddddds**", r#"aaaaaaadefdfd.dsds::er3\ddd"#),
        ];

        for i in 0..default_values.len() {
            let (schema, sequence) =
                PgStructFetcher::get_sequence_name_by_default_value(default_values[i]);
            assert_eq!((schema.as_str(), sequence.as_str()), expect_sequences[i]);
        }
    }
}

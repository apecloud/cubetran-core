use std::collections::HashMap;

use sqlx::{mysql::MySqlRow, postgres::PgRow};

use crate::adaptor::{
    mysql_col_value_convertor::MysqlColValueConvertor, pg_col_value_convertor::PgColValueConvertor,
};

use super::{
    col_value::ColValue, mysql::mysql_tb_meta::MysqlTbMeta, pg::pg_tb_meta::PgTbMeta,
    rdb_tb_meta::RdbTbMeta, row_type::RowType,
};

#[derive(Debug, Clone)]
pub struct RowData {
    // TODO, change to schema
    pub db: String,
    pub tb: String,
    pub row_type: RowType,
    pub before: Option<HashMap<String, ColValue>>,
    pub after: Option<HashMap<String, ColValue>>,
    pub current_position: String,
    // for mysql, this is the binlog position of the last commited transaction
    // for postgres, this is the sln of the last commited commited transaction
    pub checkpoint_position: String,
}

#[allow(dead_code)]
impl RowData {
    pub fn from_mysql_row(row: &MySqlRow, tb_meta: &MysqlTbMeta) -> Self {
        let mut after = HashMap::new();
        for (col, col_type) in &tb_meta.col_type_map {
            let col_val = MysqlColValueConvertor::from_query(row, col, col_type).unwrap();
            after.insert(col.to_string(), col_val);
        }
        Self::build_insert_row_data(after, &tb_meta.basic)
    }

    pub fn from_pg_row(row: &PgRow, tb_meta: &PgTbMeta) -> Self {
        let mut after = HashMap::new();
        for (col_name, col_type) in &tb_meta.col_type_map {
            let col_value = PgColValueConvertor::from_query(row, &col_name, &col_type).unwrap();
            after.insert(col_name.to_string(), col_value);
        }
        Self::build_insert_row_data(after, &tb_meta.basic)
    }

    pub fn build_insert_row_data(after: HashMap<String, ColValue>, tb_meta: &RdbTbMeta) -> Self {
        RowData {
            db: tb_meta.schema.clone(),
            tb: tb_meta.tb.clone(),
            before: None,
            after: Some(after),
            row_type: RowType::Insert,
            current_position: "".to_string(),
            checkpoint_position: "".to_string(),
        }
    }

    pub fn to_string(&self, tb_meta: &RdbTbMeta) -> String {
        let mut result = Vec::new();
        result.push(format!(
            "db: {}, tb: {}, row_type: {}",
            self.db,
            self.tb,
            self.row_type.to_str()
        ));
        result.push(format!(
            "before: {}",
            Self::format_values(&self.before, tb_meta)
        ));
        result.push(format!(
            "after: {}",
            Self::format_values(&self.after, tb_meta)
        ));
        result.join("\n")
    }

    pub fn get_hash_code(&self, tb_meta: &RdbTbMeta) -> u128 {
        let col_values = match self.row_type {
            RowType::Insert => self.after.as_ref().unwrap(),
            _ => self.before.as_ref().unwrap(),
        };

        // refer to: https://docs.oracle.com/javase/6/docs/api/java/util/List.html#hashCode%28%29
        let mut hash_code = 1u128;
        for col in tb_meta.id_cols.iter() {
            let col_hash_code = col_values.get(col).unwrap().hash_code();
            // col_hash_code is 0 if col_value is ColValue::None,
            // consider fowlling case,
            // create table a(id int, value int, unique key(id, value));
            // insert into a values(1, NULL);
            // delete from a where (id, value) in ((1, NULL));  // this won't work
            // delete from a where id=1 and value is NULL;  // this works
            // so here return 0 to stop merging to avoid batch deleting
            if col_hash_code == 0 {
                return 0;
            }
            hash_code = 31 * hash_code + col_hash_code as u128;
        }
        hash_code
    }

    fn format_values(values: &Option<HashMap<String, ColValue>>, tb_meta: &RdbTbMeta) -> String {
        if values.is_none() {
            return "none".to_string();
        }

        let mut result = Vec::new();
        let unwrap_values = values.as_ref().unwrap();
        for col in tb_meta.cols.iter() {
            if !unwrap_values.contains_key(col) {
                result.push(format!("{}: none", col));
                continue;
            }

            result.push(format!(
                "{}: {}",
                col,
                unwrap_values.get(col).unwrap().to_string()
            ));
        }
        return result.join(" , ");
    }
}
use std::sync::{atomic::AtomicBool, Arc, Mutex};

use concurrent_queue::ConcurrentQueue;
use dt_common::meta::db_enums::DbType;
use futures::TryStreamExt;
use sqlx::Row;

use crate::{
    common::syncer::Syncer,
    error::Error,
    extractor::{
        mysql::{
            mysql_cdc_extractor::MysqlCdcExtractor, mysql_check_extractor::MysqlCheckExtractor,
            mysql_snapshot_extractor::MysqlSnapshotExtractor,
        },
        pg::{
            pg_cdc_extractor::PgCdcExtractor, pg_check_extractor::PgCheckExtractor,
            pg_snapshot_extractor::PgSnapshotExtractor,
        },
        rdb_filter::RdbFilter,
    },
    meta::{
        mysql::mysql_meta_manager::MysqlMetaManager, pg::pg_meta_manager::PgMetaManager,
        row_data::RowData,
    },
};

use super::task_util::TaskUtil;

pub struct ExtractorUtil {}

const MYSQL_SYS_DBS: [&str; 4] = ["information_schema", "mysql", "performance_schema", "sys"];
const PG_SYS_SCHEMAS: [&str; 2] = ["pg_catalog", "information_schema"];

impl ExtractorUtil {
    pub async fn list_dbs(url: &str, db_type: &DbType) -> Result<Vec<String>, Error> {
        let mut dbs = match db_type {
            DbType::Mysql => Self::list_mysql_dbs(url).await?,
            DbType::Pg => Self::list_pg_schemas(url).await?,
        };
        dbs.sort();
        Ok(dbs)
    }

    pub async fn list_tbs(url: &str, db: &str, db_type: &DbType) -> Result<Vec<String>, Error> {
        let mut tbs = match db_type {
            DbType::Mysql => Self::list_mysql_tbs(url, db).await?,
            DbType::Pg => Self::list_pg_tbs(url, db).await?,
        };
        tbs.sort();
        Ok(tbs)
    }

    async fn list_pg_schemas(url: &str) -> Result<Vec<String>, Error> {
        let mut schemas = Vec::new();
        let conn_pool = TaskUtil::create_pg_conn_pool(url, 1, false).await?;

        let sql = "SELECT schema_name
            FROM information_schema.schemata
            WHERE catalog_name = current_database()";
        let mut rows = sqlx::query(&sql).fetch(&conn_pool);
        while let Some(row) = rows.try_next().await.unwrap() {
            let schema: String = row.try_get(0)?;
            if PG_SYS_SCHEMAS.contains(&schema.as_str()) {
                continue;
            }
            schemas.push(schema);
        }
        Ok(schemas)
    }

    async fn list_pg_tbs(url: &str, schema: &str) -> Result<Vec<String>, Error> {
        let mut tbs = Vec::new();
        let conn_pool = TaskUtil::create_pg_conn_pool(url, 1, false).await?;

        let sql = format!(
            "SELECT table_name 
            FROM information_schema.tables
            WHERE table_catalog = current_database() 
            AND table_schema = '{}'",
            schema
        );
        let mut rows = sqlx::query(&sql).fetch(&conn_pool);
        while let Some(row) = rows.try_next().await.unwrap() {
            let tb: String = row.try_get(0)?;
            tbs.push(tb);
        }
        Ok(tbs)
    }

    async fn list_mysql_dbs(url: &str) -> Result<Vec<String>, Error> {
        let mut dbs = Vec::new();
        let conn_pool = TaskUtil::create_mysql_conn_pool(url, 1, false).await?;

        let sql = format!("SHOW DATABASES");
        let mut rows = sqlx::query(&sql).fetch(&conn_pool);
        while let Some(row) = rows.try_next().await.unwrap() {
            let db: String = row.try_get(0)?;
            if MYSQL_SYS_DBS.contains(&db.as_str()) {
                continue;
            }
            dbs.push(db);
        }
        Ok(dbs)
    }

    async fn list_mysql_tbs(url: &str, db: &str) -> Result<Vec<String>, Error> {
        let mut tbs = Vec::new();
        let conn_pool = TaskUtil::create_mysql_conn_pool(url, 1, false).await?;

        let sql = format!("SHOW TABLES IN {}", db);
        let mut rows = sqlx::query(&sql).fetch(&conn_pool);
        while let Some(row) = rows.try_next().await.unwrap() {
            let tb: String = row.try_get(0)?;
            tbs.push(tb);
        }
        Ok(tbs)
    }

    pub async fn create_mysql_cdc_extractor<'a>(
        url: &str,
        binlog_filename: &str,
        binlog_position: u32,
        server_id: u64,
        buffer: &'a ConcurrentQueue<RowData>,
        filter: RdbFilter,
        log_level: &str,
        shut_down: &'a AtomicBool,
    ) -> Result<MysqlCdcExtractor<'a>, Error> {
        let enable_sqlx_log = TaskUtil::check_enable_sqlx_log(log_level);
        let conn_pool = TaskUtil::create_mysql_conn_pool(url, 2, enable_sqlx_log).await?;
        let meta_manager = MysqlMetaManager::new(conn_pool).init().await?;

        Ok(MysqlCdcExtractor {
            meta_manager,
            buffer,
            filter,
            url: url.to_string(),
            binlog_filename: binlog_filename.to_string(),
            binlog_position,
            server_id,
            shut_down: &shut_down,
        })
    }

    pub async fn create_pg_cdc_extractor<'a>(
        url: &str,
        slot_name: &str,
        start_lsn: &str,
        heartbeat_interval_secs: u64,
        buffer: &'a ConcurrentQueue<RowData>,
        filter: RdbFilter,
        log_level: &str,
        shut_down: &'a AtomicBool,
        syncer: Arc<Mutex<Syncer>>,
    ) -> Result<PgCdcExtractor<'a>, Error> {
        let enable_sqlx_log = TaskUtil::check_enable_sqlx_log(log_level);
        let conn_pool = TaskUtil::create_pg_conn_pool(url, 2, enable_sqlx_log).await?;
        let meta_manager = PgMetaManager::new(conn_pool.clone()).init().await?;

        Ok(PgCdcExtractor {
            meta_manager,
            buffer,
            filter,
            url: url.to_string(),
            slot_name: slot_name.to_string(),
            start_lsn: start_lsn.to_string(),
            shut_down: &shut_down,
            syncer,
            heartbeat_interval_secs,
        })
    }

    pub async fn create_mysql_snapshot_extractor<'a>(
        url: &str,
        db: &str,
        tb: &str,
        slice_size: usize,
        buffer: &'a ConcurrentQueue<RowData>,
        log_level: &str,
        shut_down: &'a AtomicBool,
    ) -> Result<MysqlSnapshotExtractor<'a>, Error> {
        let enable_sqlx_log = TaskUtil::check_enable_sqlx_log(log_level);
        // max_connections: 1 for extracting data from table, 1 for db-meta-manager
        let conn_pool = TaskUtil::create_mysql_conn_pool(url, 2, enable_sqlx_log).await?;
        let meta_manager = MysqlMetaManager::new(conn_pool.clone()).init().await?;

        Ok(MysqlSnapshotExtractor {
            conn_pool: conn_pool.clone(),
            meta_manager,
            buffer,
            db: db.to_string(),
            tb: tb.to_string(),
            slice_size,
            shut_down: &&shut_down,
        })
    }

    pub async fn create_mysql_check_extractor<'a>(
        url: &str,
        check_log_dir: &str,
        slice_size: usize,
        buffer: &'a ConcurrentQueue<RowData>,
        log_level: &str,
        shut_down: &'a AtomicBool,
    ) -> Result<MysqlCheckExtractor<'a>, Error> {
        let enable_sqlx_log = TaskUtil::check_enable_sqlx_log(log_level);
        let conn_pool = TaskUtil::create_mysql_conn_pool(url, 2, enable_sqlx_log).await?;
        let meta_manager = MysqlMetaManager::new(conn_pool.clone()).init().await?;

        Ok(MysqlCheckExtractor {
            conn_pool: conn_pool.clone(),
            meta_manager,
            buffer,
            check_log_dir: check_log_dir.to_string(),
            slice_size,
            shut_down: &&shut_down,
        })
    }

    pub async fn create_pg_check_extractor<'a>(
        url: &str,
        check_log_dir: &str,
        slice_size: usize,
        buffer: &'a ConcurrentQueue<RowData>,
        log_level: &str,
        shut_down: &'a AtomicBool,
    ) -> Result<PgCheckExtractor<'a>, Error> {
        let enable_sqlx_log = TaskUtil::check_enable_sqlx_log(log_level);
        let conn_pool = TaskUtil::create_pg_conn_pool(url, 2, enable_sqlx_log).await?;
        let meta_manager = PgMetaManager::new(conn_pool.clone()).init().await?;

        Ok(PgCheckExtractor {
            conn_pool: conn_pool.clone(),
            meta_manager,
            check_log_dir: check_log_dir.to_string(),
            buffer,
            slice_size,
            shut_down: &&shut_down,
        })
    }

    pub async fn create_pg_snapshot_extractor<'a>(
        url: &str,
        db: &str,
        tb: &str,
        slice_size: usize,
        buffer: &'a ConcurrentQueue<RowData>,
        log_level: &str,
        shut_down: &'a AtomicBool,
    ) -> Result<PgSnapshotExtractor<'a>, Error> {
        let enable_sqlx_log = TaskUtil::check_enable_sqlx_log(log_level);
        let conn_pool = TaskUtil::create_pg_conn_pool(url, 2, enable_sqlx_log).await?;
        let meta_manager = PgMetaManager::new(conn_pool.clone()).init().await?;

        Ok(PgSnapshotExtractor {
            conn_pool: conn_pool.clone(),
            meta_manager,
            buffer,
            slice_size,
            schema: db.to_string(),
            tb: tb.to_string(),
            shut_down: &&shut_down,
        })
    }
}

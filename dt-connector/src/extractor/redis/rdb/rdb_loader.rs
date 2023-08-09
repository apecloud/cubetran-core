use dt_common::{error::Error, log_info};
use dt_meta::redis::{redis_entry::RedisEntry, redis_object::RedisCmd};
use sqlx::types::chrono;

use crate::extractor::redis::RawByteReader;

use super::{entry_parser::entry_parser::EntryParser, reader::rdb_reader::RdbReader};

const K_FLAG_FUNCTION2: u8 = 245; // function library data
const K_FLAG_FUNCTION: u8 = 246; // old function library data for 7.0 rc1 and rc2
const K_FLAG_MODULE_AUX: u8 = 247; // Module auxiliary data.
const K_FLAG_IDLE: u8 = 0xf8; // LRU idle time.
const K_FLAG_FREQ: u8 = 0xf9; // LFU frequency.
const K_FLAG_AUX: u8 = 0xfa; // RDB aux field.
const K_FLAG_RESIZE_DB: u8 = 0xfb; // Hash table resize hint.
const K_FLAG_EXPIRE_MS: u8 = 0xfc; // Expire time in milliseconds.
const K_FLAG_EXPIRE: u8 = 0xfd; // Old expire time in seconds.
const K_FLAG_SELECT: u8 = 0xfe; // DB number of the following keys.
const K_EOF: u8 = 0xff; // End of the RDB file.

pub struct RdbLoader<'a> {
    pub reader: RdbReader<'a>,
    pub repl_stream_db_id: i64,
    pub now_db_id: i64,
    pub expire_ms: i64,
    pub idle: i64,
    pub freq: i64,

    pub is_end: bool,
}

impl RdbLoader<'_> {
    pub fn load_meta(&mut self) -> Result<String, Error> {
        // magic
        let mut buf = self.reader.read_raw(5)?;
        let magic = String::from_utf8(buf).unwrap();
        if magic != "REDIS" {
            return Err(Error::Unexpected {
                error: "invalid rdb format".to_string(),
            });
        }

        // version
        buf = self.reader.read_raw(4)?;
        let version = String::from_utf8(buf).unwrap();
        Ok(version)
    }

    pub fn load_entry(&mut self) -> Result<Option<RedisEntry>, Error> {
        let type_byte = self.reader.read_byte()?;

        match type_byte {
            K_FLAG_IDLE => {
                self.idle = self.reader.read_length()? as i64;
            }

            K_FLAG_FREQ => {
                self.freq = self.reader.read_u8()? as i64;
            }

            K_FLAG_AUX => {
                let key = String::from(self.reader.read_string()?);
                let value = self.reader.read_string()?;
                match key.as_str() {
                    "repl-stream-db" => {
                        let value = String::from(value);
                        self.repl_stream_db_id = value.parse::<i64>().unwrap();
                        log_info!("RDB repl-stream-db: {}", self.repl_stream_db_id);
                    }

                    "lua" => {
                        let mut cmd = RedisCmd::new();
                        cmd.add_str_arg("script");
                        cmd.add_str_arg("load");
                        cmd.add_redis_arg(&value);
                        log_info!("LUA script: {:?}", value);

                        let mut entry = RedisEntry::new();
                        entry.is_base = true;
                        entry.db_id = self.now_db_id;
                        entry.cmd = cmd;
                        return Ok(Some(entry));
                    }

                    _ => {
                        log_info!("RDB AUX fields. key=[{}], value=[{:?}]", key, value);
                    }
                }
            }

            K_FLAG_RESIZE_DB => {
                let db_size = self.reader.read_length()?;
                let expire_size = self.reader.read_length()?;
                log_info!(
                    "RDB resize db. db_size=[{}], expire_size=[{}]",
                    db_size,
                    expire_size
                )
            }

            K_FLAG_EXPIRE_MS => {
                let mut expire_ms = self.reader.read_u64()? as i64;
                expire_ms -= chrono::Utc::now().timestamp_millis();
                if expire_ms < 0 {
                    expire_ms = 1
                }
                self.expire_ms = expire_ms;
            }

            K_FLAG_EXPIRE => {
                let mut expire_ms = self.reader.read_u32()? as i64 * 1000;
                expire_ms -= chrono::Utc::now().timestamp_millis();
                if expire_ms < 0 {
                    expire_ms = 1
                }
                self.expire_ms = expire_ms;
            }

            K_FLAG_SELECT => {
                self.now_db_id = self.reader.read_length()? as i64;
            }

            K_EOF => {
                self.is_end = true;
                self.reader
                    .read_raw(self.reader.rdb_length - self.reader.position)?;
            }

            _ => {
                let key = self.reader.read_string()?;
                self.reader.copy_raw = true;
                let value = EntryParser::parse_object(&mut self.reader, type_byte, key.clone());
                self.reader.copy_raw = false;

                if let Err(error) = value {
                    panic!(
                        "parsing rdb failed, key: {:?}, error: {:?}",
                        String::from(key),
                        error
                    );
                } else {
                    let mut entry = RedisEntry::new();
                    entry.is_base = true;
                    entry.db_id = self.now_db_id;
                    entry.raw_bytes = self.reader.drain_raw_bytes();
                    entry.key = key;
                    entry.value = value.unwrap();
                    entry.value_type_byte = type_byte;
                    return Ok(Some(entry));
                }
            }
        }

        Ok(None)
    }
}
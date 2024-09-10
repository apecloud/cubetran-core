use std::io::Cursor;

use crate::{config::config_enums::DbType, error::Error, meta::time::dt_utc_time::DtNaiveTime};
use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt};
use chrono::{TimeZone, Utc};
use mysql_binlog_connector_rust::column::{
    column_value::ColumnValue, json::json_binary::JsonBinary,
};
use sqlx::{mysql::MySqlRow, types::BigDecimal, Row};

use crate::meta::{col_value::ColValue, mysql::mysql_col_type::MysqlColType};

pub struct MysqlColValueConvertor {}

impl MysqlColValueConvertor {
    pub fn parse_time(buf: Vec<u8>) -> anyhow::Result<ColValue> {
        // for: 13:14:15.456, buf: [12, 0, 0, 0, 0, 0, 13, 14, 15, 64, 245, 6, 0]
        // for: -838:59:59, buf: [8, 1, 34, 0, 0, 0, 22, 59, 59]

        // https://mariadb.com/kb/en/resultset-row/#timestamp-binary-encoding
        //  int<1> data length: 0 for special '00:00:00' value, 8 without fractional seconds, 12 with fractional seconds
        // if data length > 0
        // int<1> 0 for positive time, 1 for negative time
        // int<4> days
        // int<1> hours
        // int<1> minutes
        // int<1> seconds
        // if data length > 8
        // int<4> microseconds

        let mut cursor = Cursor::new(buf);
        let length = cursor.read_u8()? as usize;
        let time = if length == 0 {
            DtNaiveTime::default()
        } else {
            let is_negative = cursor.read_u8()? != 0;
            let day = cursor.read_u32::<LittleEndian>()?;
            let mut time = Self::parese_time_fields(&mut cursor, length - 5)?;
            time.hour += day * 24;
            time.is_negative = is_negative;
            time
        };
        Ok(ColValue::Time(time.to_string()))
    }

    pub fn parse_date(buf: Vec<u8>) -> anyhow::Result<ColValue> {
        let mut cursor = Cursor::new(buf);
        let length = cursor.read_u8()? as usize;
        let date = Self::parese_date_fields(&mut cursor, length)?;
        Ok(ColValue::Date(date))
    }

    pub fn parse_datetime(buf: Vec<u8>) -> anyhow::Result<ColValue> {
        let mut cursor = Cursor::new(buf);
        let datetime = Self::parse_date_time_fields(&mut cursor)?;
        Ok(ColValue::DateTime(datetime))
    }

    pub fn parse_timestamp(buf: Vec<u8>) -> anyhow::Result<ColValue> {
        let mut cursor = Cursor::new(buf);
        let datetime = Self::parse_date_time_fields(&mut cursor)?;
        Ok(ColValue::Timestamp(datetime))
    }

    fn parse_date_time_fields(cursor: &mut Cursor<Vec<u8>>) -> anyhow::Result<String> {
        let length = cursor.read_u8()? as usize;
        let date = Self::parese_date_fields(cursor, length)?;
        let time = Self::parese_time_fields(cursor, length - 4)?;
        Ok(format!("{} {}", date, time))
    }

    fn parese_date_fields(cursor: &mut Cursor<Vec<u8>>, length: usize) -> anyhow::Result<String> {
        let mut year = 0;
        let mut month = 0;
        let mut day = 0;
        if length >= 2 {
            year = cursor.read_u16::<LittleEndian>()?;
        }
        if length >= 3 {
            month = cursor.read_u8()?;
        }
        if length >= 4 {
            day = cursor.read_u8()?;
        }
        Ok(format!("{}-{:02}-{:02}", year, month, day))
    }

    #[allow(clippy::field_reassign_with_default)]
    fn parese_time_fields(
        cursor: &mut Cursor<Vec<u8>>,
        length: usize,
    ) -> anyhow::Result<DtNaiveTime> {
        let mut time = DtNaiveTime::default();
        time.hour = cursor.read_u8()? as u32;
        time.minute = cursor.read_u8()? as u32;
        time.second = cursor.read_u8()? as u32;
        if length >= 4 {
            let microsecond = cursor.read_uint::<LittleEndian>(length - 3)?;
            time.microsecond = microsecond as u32;
        }
        Ok(time)
    }

    pub fn from_binlog(col_type: &MysqlColType, value: ColumnValue) -> anyhow::Result<ColValue> {
        let col_value = match value {
            ColumnValue::Tiny(v) => {
                if *col_type == MysqlColType::UnsignedTiny {
                    ColValue::UnsignedTiny(v as u8)
                } else {
                    ColValue::Tiny(v)
                }
            }

            ColumnValue::Short(v) => {
                if *col_type == MysqlColType::UnsignedShort {
                    ColValue::UnsignedShort(v as u16)
                } else {
                    ColValue::Short(v)
                }
            }

            ColumnValue::Long(v) => {
                if *col_type == MysqlColType::UnsignedMedium {
                    ColValue::UnsignedLong((v as u32) << 8 >> 8)
                } else if *col_type == MysqlColType::UnsignedLong {
                    ColValue::UnsignedLong(v as u32)
                } else {
                    ColValue::Long(v)
                }
            }

            ColumnValue::LongLong(v) => {
                if *col_type == MysqlColType::UnsignedLongLong {
                    ColValue::UnsignedLongLong(v as u64)
                } else {
                    ColValue::LongLong(v)
                }
            }

            ColumnValue::Float(v) => ColValue::Float(v),
            ColumnValue::Double(v) => ColValue::Double(v),
            ColumnValue::Decimal(v) => ColValue::Decimal(v),
            ColumnValue::Time(v) => ColValue::Time(v),
            ColumnValue::Date(v) => ColValue::Date(v),
            ColumnValue::DateTime(v) => ColValue::DateTime(v),
            ColumnValue::Year(v) => ColValue::Year(v),

            ColumnValue::Timestamp(v) => {
                if let MysqlColType::Timestamp {
                    timezone_offset: timezone_diff_utc_seconds,
                } = *col_type
                {
                    // the value parsed from binlog is in millis with UTC
                    let dt = Utc.timestamp_nanos(v * 1000 + timezone_diff_utc_seconds * 1000000000);
                    ColValue::Timestamp(dt.to_string().replace(" UTC", ""))
                } else {
                    let dt = Utc.timestamp_nanos(v * 1000);
                    ColValue::Timestamp(dt.to_string().replace(" UTC", ""))
                }
            }

            // char, varchar, binary, varbinary
            ColumnValue::String(v) => {
                // when the type is binary(length), the value shoud be right-padded with '\0' to the specified length,
                // refer: https://dev.mysql.com/doc/refman/8.0/en/binary-varbinary.html
                match *col_type {
                    // binary
                    MysqlColType::Binary { length } => {
                        let final_v = if length as usize > v.len() {
                            let pad_v: Vec<u8> = vec![0; length as usize - v.len()];
                            [v, pad_v].concat()
                        } else {
                            v
                        };
                        ColValue::Blob(final_v)
                    }
                    // varbinary
                    MysqlColType::VarBinary { length: _ } => ColValue::Blob(v),
                    // char, varchar
                    _ => ColValue::RawString(v),
                }
            }

            // tinyblob, mediumblob, longblob, blob, tinytext, mediumtext, longtext, text
            ColumnValue::Blob(v) => match col_type {
                // tinytext, mediumtext, longtext, text
                MysqlColType::String {
                    length: _,
                    charset: _,
                } => ColValue::RawString(v),
                // tinyblob, mediumblob, longblob, blob
                _ => ColValue::Blob(v),
            },

            ColumnValue::Bit(v) => ColValue::Bit(v),

            ColumnValue::Set(mut v) => match col_type {
                MysqlColType::Set { items } => {
                    if v == 0 {
                        return Ok(ColValue::Set2(String::new()));
                    }
                    let mut matched_items = Vec::new();
                    let mut pos = 0;
                    while v > 0 {
                        let mut i = v & 0x01;
                        if i > 0 {
                            i <<= pos;
                            if let Some(item) = items.get(&i) {
                                matched_items.push(item.to_owned());
                            }
                        }
                        v >>= 1;
                        pos += 1;
                    }
                    ColValue::Set2(matched_items.join(","))
                }
                // should never happen
                _ => ColValue::Set(v),
            },

            ColumnValue::Enum(v) => match col_type {
                MysqlColType::Enum { items } => {
                    if let Some(item) = items.get(&v) {
                        ColValue::Enum2(item.to_owned())
                    } else {
                        // should never happen
                        ColValue::None
                    }
                }
                // should never happen
                _ => ColValue::Enum(v),
            },

            ColumnValue::Json(v) => {
                let v = JsonBinary::parse_as_string(&v)?;
                ColValue::Json2(v)
            }

            _ => ColValue::None,
        };

        Ok(col_value)
    }

    pub fn from_str(col_type: &MysqlColType, value_str: &str) -> anyhow::Result<ColValue> {
        let value_str = value_str.to_string();
        let col_value = match *col_type {
            MysqlColType::Tiny => match value_str.parse::<i8>() {
                Ok(value) => ColValue::Tiny(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::UnsignedTiny => match value_str.parse::<u8>() {
                Ok(value) => ColValue::UnsignedTiny(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::Short => match value_str.parse::<i16>() {
                Ok(value) => ColValue::Short(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::UnsignedShort => match value_str.parse::<u16>() {
                Ok(value) => ColValue::UnsignedShort(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::Medium | MysqlColType::Long => match value_str.parse::<i32>() {
                Ok(value) => ColValue::Long(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::UnsignedMedium | MysqlColType::UnsignedLong => {
                match value_str.parse::<u32>() {
                    Ok(value) => ColValue::UnsignedLong(value),
                    Err(_) => ColValue::None,
                }
            }
            MysqlColType::LongLong => match value_str.parse::<i64>() {
                Ok(value) => ColValue::LongLong(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::UnsignedLongLong => match value_str.parse::<u64>() {
                Ok(value) => ColValue::UnsignedLongLong(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::Float => match value_str.parse::<f32>() {
                Ok(value) => ColValue::Float(value),
                Err(_) => ColValue::None,
            },
            MysqlColType::Double => match value_str.parse::<f64>() {
                Ok(value) => ColValue::Double(value),
                Err(_) => ColValue::None,
            },

            MysqlColType::Decimal { .. } => ColValue::Decimal(value_str),
            MysqlColType::Time => ColValue::Time(value_str),
            MysqlColType::Date => ColValue::Date(value_str),
            MysqlColType::DateTime => ColValue::DateTime(value_str),

            MysqlColType::Timestamp { timezone_offset: _ } => ColValue::Timestamp(value_str),

            MysqlColType::Year => match value_str.parse::<u16>() {
                Ok(value) => ColValue::Year(value),
                Err(_) => ColValue::None,
            },

            MysqlColType::String {
                length: _,
                charset: _,
            } => ColValue::String(value_str),

            MysqlColType::Bit => match value_str.parse::<u64>() {
                Ok(value) => ColValue::Bit(value),
                Err(_) => ColValue::None,
            },

            MysqlColType::Set { .. } => ColValue::String(value_str),
            MysqlColType::Enum { .. } => ColValue::String(value_str),

            MysqlColType::Json => ColValue::Json2(value_str),

            _ => {
                bail! {Error::Unexpected(format!(
                    "unsupported column type: {:?}",
                    col_type
                )) }
            }
        };

        Ok(col_value)
    }

    pub fn from_query(
        row: &MySqlRow,
        col: &str,
        col_type: &MysqlColType,
    ) -> anyhow::Result<ColValue> {
        Self::from_query_mysql_compatible(row, col, col_type, &DbType::Mysql)
    }

    pub fn from_query_mysql_compatible(
        row: &MySqlRow,
        col: &str,
        col_type: &MysqlColType,
        db_type: &DbType,
    ) -> anyhow::Result<ColValue> {
        let value: Option<Vec<u8>> = row.get_unchecked(col);
        if value.is_none() {
            return Ok(ColValue::None);
        }

        match col_type {
            MysqlColType::Tiny => {
                let value: i8 = row.try_get(col)?;
                return Ok(ColValue::Tiny(value));
            }
            MysqlColType::UnsignedTiny => {
                let value: u8 = row.try_get(col)?;
                return Ok(ColValue::UnsignedTiny(value));
            }
            MysqlColType::Short => {
                let value: i16 = row.try_get(col)?;
                return Ok(ColValue::Short(value));
            }
            MysqlColType::UnsignedShort => {
                let value: u16 = row.try_get(col)?;
                return Ok(ColValue::UnsignedShort(value));
            }
            MysqlColType::Medium | MysqlColType::Long => {
                let value: i32 = row.try_get(col)?;
                return Ok(ColValue::Long(value));
            }
            MysqlColType::UnsignedMedium | MysqlColType::UnsignedLong => {
                let value: u32 = row.try_get(col)?;
                return Ok(ColValue::UnsignedLong(value));
            }
            MysqlColType::LongLong => {
                let value: i64 = row.try_get(col)?;
                return Ok(ColValue::LongLong(value));
            }
            MysqlColType::UnsignedLongLong => {
                let value: u64 = row.try_get(col)?;
                return Ok(ColValue::UnsignedLongLong(value));
            }
            MysqlColType::Float => {
                let value: f32 = row.try_get(col)?;
                return Ok(ColValue::Float(value));
            }
            MysqlColType::Double => {
                let value: f64 = row.try_get(col)?;
                return Ok(ColValue::Double(value));
            }
            MysqlColType::Decimal { .. } => {
                let value: BigDecimal = row.get_unchecked(col);
                return Ok(ColValue::Decimal(value.to_string()));
            }
            MysqlColType::Time => match db_type {
                DbType::Foxlake => {
                    let value: Vec<u8> = row.get_unchecked(col);
                    let str: String = String::from_utf8_lossy(&value).to_string();
                    return Ok(ColValue::Time(str));
                }
                _ => {
                    // do not use chrono::NaiveTime since it ignores year
                    // let value: chrono::NaiveTime = row.try_get(col)?;
                    let buf: Vec<u8> = row.get_unchecked(col);
                    return Self::parse_time(buf);
                }
            },
            MysqlColType::Date => match db_type {
                DbType::StarRocks | DbType::Foxlake => {
                    let value: Vec<u8> = row.get_unchecked(col);
                    let str: String = String::from_utf8_lossy(&value).to_string();
                    return Ok(ColValue::Date(str));
                }
                _ => {
                    let value: chrono::NaiveDate = row.try_get(col)?;
                    return Ok(ColValue::Date(value.format("%Y-%m-%d").to_string()));
                }
            },
            MysqlColType::DateTime => match db_type {
                DbType::StarRocks | DbType::Foxlake => {
                    let value: Vec<u8> = row.get_unchecked(col);
                    let str: String = String::from_utf8_lossy(&value).to_string();
                    return Ok(ColValue::DateTime(str));
                }
                _ => {
                    let value: chrono::NaiveDateTime = row.try_get(col)?;
                    return Ok(ColValue::DateTime(
                        value.format("%Y-%m-%d %H:%M:%S%.f").to_string(),
                    ));
                }
            },
            MysqlColType::Timestamp { timezone_offset: _ } => match db_type {
                DbType::Foxlake => {
                    let value: Vec<u8> = row.get_unchecked(col);
                    let str: String = String::from_utf8_lossy(&value).to_string();
                    return Ok(ColValue::Timestamp(str));
                }
                _ => {
                    // we always set session.time_zone='+00:00' for connection
                    let value: chrono::DateTime<Utc> = row.try_get(col)?;
                    return Ok(ColValue::Timestamp(
                        value.format("%Y-%m-%d %H:%M:%S%.f").to_string(),
                    ));
                }
            },
            MysqlColType::Year => {
                let value: u16 = row.get_unchecked(col);
                return Ok(ColValue::Year(value));
            }
            MysqlColType::String {
                length: _,
                charset: _,
            } => {
                let value: String = row.try_get(col)?;
                return Ok(ColValue::String(value));
            }
            MysqlColType::Binary { length: _ } => {
                let value: Vec<u8> = row.try_get(col)?;
                return Ok(ColValue::Blob(value));
            }
            MysqlColType::VarBinary { length: _ } => {
                let value: Vec<u8> = row.try_get(col)?;
                return Ok(ColValue::Blob(value));
            }
            MysqlColType::Blob => {
                let value: Vec<u8> = row.try_get(col)?;
                return Ok(ColValue::Blob(value));
            }
            MysqlColType::Bit => {
                let value: u64 = row.try_get(col)?;
                return Ok(ColValue::Bit(value));
            }
            MysqlColType::Set { .. } => {
                let value: String = row.try_get(col)?;
                return Ok(ColValue::Set2(value));
            }
            MysqlColType::Enum { .. } => {
                let value: String = row.try_get(col)?;
                return Ok(ColValue::Enum2(value));
            }
            MysqlColType::Json => {
                let value: serde_json::Value = row.try_get(col)?;
                // TODO, decimal will lose precision when insert into target mysql as string.
                // insert into json_table(id, json_col) values(1, "212765.700000000010000"); the result will be:
                // +-----+--------------------------+
                // | id | json_col                  |
                // |  1 | 212765.7                  |
                return Ok(ColValue::Json2(value.to_string()));
            }
            _ => {}
        }
        Ok(ColValue::None)
    }
}
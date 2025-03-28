test_db_1
CREATE DATABASE `test_db_1`

dst_test_db_2
CREATE DATABASE `dst_test_db_2`

test_db_1.full_column_type
CREATE TABLE `full_column_type` (
  `id` int(11) NOT NULL COMMENT "",
  `char_col` varchar(65533) NULL COMMENT "",
  `char_col_2` varchar(65533) NULL COMMENT "",
  `character_col` varchar(65533) NULL COMMENT "",
  `character_col_2` varchar(65533) NULL COMMENT "",
  `varchar_col` varchar(65533) NULL COMMENT "",
  `varchar_col_2` varchar(65533) NULL COMMENT "",
  `character_varying_col` varchar(65533) NULL COMMENT "",
  `character_varying_col_2` varchar(65533) NULL COMMENT "",
  `bpchar_col` varchar(65533) NULL COMMENT "",
  `bpchar_col_2` varchar(65533) NULL COMMENT "",
  `text_col` varchar(65533) NULL COMMENT "",
  `real_col` float NULL COMMENT "",
  `float4_col` float NULL COMMENT "",
  `double_precision_col` double NULL COMMENT "",
  `float8_col` double NULL COMMENT "",
  `numeric_col` decimal(38, 9) NULL COMMENT "",
  `numeric_col_2` decimal(38, 9) NULL COMMENT "",
  `decimal_col` decimal(38, 9) NULL COMMENT "",
  `decimal_col_2` decimal(38, 9) NULL COMMENT "",
  `smallint_col` smallint(6) NULL COMMENT "",
  `int2_col` smallint(6) NULL COMMENT "",
  `smallserial_col` smallint(6) NULL COMMENT "",
  `serial2_col` smallint(6) NULL COMMENT "",
  `integer_col` int(11) NULL COMMENT "",
  `int_col` int(11) NULL COMMENT "",
  `int4_col` int(11) NULL COMMENT "",
  `serial_col` int(11) NULL COMMENT "",
  `serial4_col` int(11) NULL COMMENT "",
  `bigint_col` bigint(20) NULL COMMENT "",
  `int8_col` bigint(20) NULL COMMENT "",
  `bigserial_col` bigint(20) NULL COMMENT "",
  `serial8_col` bigint(20) NULL COMMENT "",
  `bit_col` varchar(65533) NULL COMMENT "",
  `bit_col_2` varchar(65533) NULL COMMENT "",
  `bit_varying_col` varchar(65533) NULL COMMENT "",
  `bit_varying_col_2` varchar(65533) NULL COMMENT "",
  `varbit_col` varchar(65533) NULL COMMENT "",
  `varbit_col_2` varchar(65533) NULL COMMENT "",
  `time_col` varchar(255) NULL COMMENT "",
  `time_col_2` varchar(255) NULL COMMENT "",
  `time_col_3` varchar(255) NULL COMMENT "",
  `time_col_4` varchar(255) NULL COMMENT "",
  `timez_col` varchar(255) NULL COMMENT "",
  `timez_col_2` varchar(255) NULL COMMENT "",
  `timez_col_3` varchar(255) NULL COMMENT "",
  `timez_col_4` varchar(255) NULL COMMENT "",
  `timestamp_col` datetime NULL COMMENT "",
  `timestamp_col_2` datetime NULL COMMENT "",
  `timestamp_col_3` datetime NULL COMMENT "",
  `timestamp_col_4` datetime NULL COMMENT "",
  `timestampz_col` datetime NULL COMMENT "",
  `timestampz_col_2` datetime NULL COMMENT "",
  `timestampz_col_3` datetime NULL COMMENT "",
  `timestampz_col_4` datetime NULL COMMENT "",
  `date_col` date NULL COMMENT "",
  `bytea_col` varbinary NULL COMMENT "",
  `boolean_col` boolean NULL COMMENT "",
  `bool_col` boolean NULL COMMENT "",
  `json_col` json NULL COMMENT "",
  `jsonb_col` json NULL COMMENT "",
  `interval_col` varchar(255) NULL COMMENT "",
  `interval_col_2` varchar(255) NULL COMMENT "",
  `array_float4_col` varchar(65533) NULL COMMENT "",
  `array_float8_col` varchar(65533) NULL COMMENT "",
  `array_int2_col` varchar(65533) NULL COMMENT "",
  `array_int4_col` varchar(65533) NULL COMMENT "",
  `array_int8_col` varchar(65533) NULL COMMENT "",
  `array_int8_col_2` varchar(65533) NULL COMMENT "",
  `array_text_col` varchar(65533) NULL COMMENT "",
  `array_boolean_col` varchar(65533) NULL COMMENT "",
  `array_boolean_col_2` varchar(65533) NULL COMMENT "",
  `array_date_col` varchar(65533) NULL COMMENT "",
  `array_timestamp_col` varchar(65533) NULL COMMENT "",
  `array_timestamp_col_2` varchar(65533) NULL COMMENT "",
  `array_timestamptz_col` varchar(65533) NULL COMMENT "",
  `array_timestamptz_col_2` varchar(65533) NULL COMMENT "",
  `box_col` varchar(65533) NULL COMMENT "",
  `cidr_col` varchar(65533) NULL COMMENT "",
  `circle_col` varchar(65533) NULL COMMENT "",
  `inet_col` varchar(65533) NULL COMMENT "",
  `line_col` varchar(65533) NULL COMMENT "",
  `lseg_col` varchar(65533) NULL COMMENT "",
  `macaddr_col` varchar(65533) NULL COMMENT "",
  `macaddr8_col` varchar(65533) NULL COMMENT "",
  `money_col` varchar(65533) NULL COMMENT "",
  `path_col` varchar(65533) NULL COMMENT "",
  `pg_lsn_col` varchar(65533) NULL COMMENT "",
  `pg_snapshot_col` varchar(65533) NULL COMMENT "",
  `polygon_col` varchar(65533) NULL COMMENT "",
  `point_col` varchar(65533) NULL COMMENT "",
  `tsquery_col` varchar(65533) NULL COMMENT "",
  `tsvector_col` varchar(65533) NULL COMMENT "",
  `txid_snapshot_col` varchar(65533) NULL COMMENT "",
  `uuid_col` varchar(65533) NULL COMMENT "",
  `xml_col` varchar(65533) NULL COMMENT "",
  `_ape_dts_is_deleted` boolean NULL COMMENT "",
  `_ape_dts_timestamp` bigint(20) NULL COMMENT ""
) ENGINE=OLAP 
PRIMARY KEY(`id`)
DISTRIBUTED BY HASH(`id`)
PROPERTIES (
"replication_num" = "1",
"in_memory" = "false",
"enable_persistent_index" = "true",
"replicated_storage" = "true",
"compression" = "LZ4"
);

test_db_1.array_table
CREATE TABLE `array_table` (
  `pk` int(11) NOT NULL COMMENT "",
  `oid_array` varchar(65533) NULL COMMENT "",
  `numeric_array` varchar(65533) NULL COMMENT "",
  `varnumeric_array` varchar(65533) NULL COMMENT "",
  `inet_array` varchar(65533) NULL COMMENT "",
  `cidr_array` varchar(65533) NULL COMMENT "",
  `macaddr_array` varchar(65533) NULL COMMENT "",
  `tsrange_array` varchar(65533) NULL COMMENT "",
  `tstzrange_array` varchar(65533) NULL COMMENT "",
  `daterange_array` varchar(65533) NULL COMMENT "",
  `int4range_array` varchar(65533) NULL COMMENT "",
  `numerange_array` varchar(65533) NULL COMMENT "",
  `int8range_array` varchar(65533) NULL COMMENT "",
  `uuid_array` varchar(65533) NULL COMMENT "",
  `json_array` varchar(65533) NULL COMMENT "",
  `jsonb_array` varchar(65533) NULL COMMENT "",
  `int_array` varchar(65533) NULL COMMENT "",
  `bigint_array` varchar(65533) NULL COMMENT "",
  `date_array` varchar(65533) NULL COMMENT "",
  `text_array` varchar(65533) NULL COMMENT "",
  `char_array` varchar(65533) NULL COMMENT "",
  `varchar_array` varchar(65533) NULL COMMENT "",
  `_ape_dts_is_deleted` boolean NULL COMMENT "",
  `_ape_dts_timestamp` bigint(20) NULL COMMENT ""
) ENGINE=OLAP 
PRIMARY KEY(`pk`)
DISTRIBUTED BY HASH(`pk`)
PROPERTIES (
"replication_num" = "1",
"in_memory" = "false",
"enable_persistent_index" = "true",
"replicated_storage" = "true",
"compression" = "LZ4"
);

test_db_1.check_pk_cols_order
CREATE TABLE `check_pk_cols_order` (
  `pk_1` int(11) NOT NULL COMMENT "",
  `pk_2` int(11) NOT NULL COMMENT "",
  `pk_3` int(11) NOT NULL COMMENT "",
  `col_1` int(11) NULL COMMENT "",
  `col_2` int(11) NULL COMMENT "",
  `col_3` int(11) NULL COMMENT "",
  `col_4` int(11) NULL COMMENT "",
  `col_5` int(11) NULL COMMENT "",
  `_ape_dts_is_deleted` boolean NULL COMMENT "",
  `_ape_dts_timestamp` bigint(20) NULL COMMENT ""
) ENGINE=OLAP 
PRIMARY KEY(`pk_1`, `pk_2`, `pk_3`)
DISTRIBUTED BY HASH(`pk_1`)
PROPERTIES (
"replication_num" = "1",
"in_memory" = "false",
"enable_persistent_index" = "true",
"replicated_storage" = "true",
"compression" = "LZ4"
);


dst_test_db_2.router_test_1
CREATE TABLE `router_test_1` (
  `pk` int(11) NOT NULL COMMENT "",
  `col_1` int(11) NULL COMMENT "",
  `_ape_dts_is_deleted` boolean NULL COMMENT "",
  `_ape_dts_timestamp` bigint(20) NULL COMMENT ""
) ENGINE=OLAP 
PRIMARY KEY(`pk`)
DISTRIBUTED BY HASH(`pk`)
PROPERTIES (
"replication_num" = "1",
"in_memory" = "false",
"enable_persistent_index" = "true",
"replicated_storage" = "true",
"compression" = "LZ4"
);

dst_test_db_2.dst_router_test_2
CREATE TABLE `dst_router_test_2` (
  `pk` int(11) NOT NULL COMMENT "",
  `col_1` int(11) NULL COMMENT "",
  `_ape_dts_is_deleted` boolean NULL COMMENT "",
  `_ape_dts_timestamp` bigint(20) NULL COMMENT ""
) ENGINE=OLAP 
PRIMARY KEY(`pk`)
DISTRIBUTED BY HASH(`pk`)
PROPERTIES (
"replication_num" = "1",
"in_memory" = "false",
"enable_persistent_index" = "true",
"replicated_storage" = "true",
"compression" = "LZ4"
);
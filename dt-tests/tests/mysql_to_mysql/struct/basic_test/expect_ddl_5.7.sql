struct_it_mysql2mysql_1
CREATE DATABASE `struct_it_mysql2mysql_1` /*!40100 DEFAULT CHARACTER SET utf8 */

struct_it_mysql2mysql_1.full_column_type
CREATE TABLE `full_column_type` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `varchar_col` varchar(255) NOT NULL COMMENT 'varchar_col_comment',
  `char_col` char(10) DEFAULT NULL COMMENT 'char_col_comment',
  `tinytext_col` tinytext COMMENT 'tinytext_col_comment',
  `mediumtext_col` mediumtext COMMENT 'mediumtext_col_comment',
  `longtext_col` longtext COMMENT 'longtext_col_comment',
  `text_col` text COMMENT 'text_col_comment',
  `tinyint_col` tinyint(4) DEFAULT '0' COMMENT 'tinyint_col_comment',
  `smallint_col` smallint(6) DEFAULT NULL COMMENT 'smallint_col_comment',
  `mediumint_col` mediumint(9) DEFAULT NULL COMMENT 'mediumint_col_comment',
  `int_col` int(11) DEFAULT NULL COMMENT 'int_col_comment',
  `bigint_col` bigint(20) DEFAULT NULL COMMENT 'bigint_col_comment',
  `float_col` float(8,2) DEFAULT NULL COMMENT 'float_col_comment',
  `double_col` double(16,4) DEFAULT NULL COMMENT 'double_col_comment',
  `bit_col` bit(64) DEFAULT NULL COMMENT 'bit_col_comment',
  `decimal_col` decimal(10,2) DEFAULT NULL COMMENT 'decimal_col_comment',
  `date_col` date DEFAULT NULL COMMENT 'date_col_comment',
  `datetime_col` datetime DEFAULT NULL COMMENT 'datetime_col_comment',
  `datetime_col2` datetime(6) DEFAULT NULL COMMENT 'datetime_col2_comment',
  `timestamp_col` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT 'timestamp_col_comment',
  `timestamp_co2` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT 'timestamp_co2_comment',
  `time_col` time DEFAULT NULL COMMENT 'time_col_comment',
  `time_col2` time(2) DEFAULT NULL COMMENT 'time_col2_comment',
  `year_col` year(4) DEFAULT NULL COMMENT 'year_col_comment',
  `binary_col` binary(16) DEFAULT NULL COMMENT 'binary_col_comment',
  `varbinary_col` varbinary(255) DEFAULT NULL COMMENT 'varbinary_col_comment',
  `blob_col` blob COMMENT 'blob_col_comment',
  `tinyblob_col` tinyblob COMMENT 'tinyblob_col_comment',
  `mediumblob_col` mediumblob COMMENT 'mediumblob_col_comment',
  `longblob_col` longblob COMMENT 'longblob_col_comment',
  `enum_col` enum('value1','value2','value3') DEFAULT NULL COMMENT 'enum_col_comment',
  `set_col` set('option1','option2','option3') DEFAULT NULL COMMENT 'set_col_comment',
  `json_col` json DEFAULT NULL COMMENT 'json_col_comment',
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8

struct_it_mysql2mysql_1.full_index_type
CREATE TABLE `full_index_type` (
  `id` int(10) unsigned NOT NULL AUTO_INCREMENT,
  `f_1` int(11) DEFAULT NULL,
  `f_2` char(128) DEFAULT NULL,
  `f_3` varchar(128) DEFAULT NULL,
  `f_4` varchar(128) DEFAULT NULL,
  `f_5` varchar(128) DEFAULT NULL,
  `f_6` text,
  `f_7` text,
  `f_8` text,
  `f_9` point NOT NULL,
  `f_10` varchar(10) DEFAULT NULL,
  `f_11` varchar(10) DEFAULT NULL,
  `f_12` varchar(10) DEFAULT NULL,
  `f_13` varchar(10) DEFAULT NULL,
  PRIMARY KEY (`id`),
  UNIQUE KEY `idx_unique_1` (`f_1`,`f_2`,`f_3`),
  UNIQUE KEY `idx_unique_2` (`f_3`),
  SPATIAL KEY `idx_spatial_1` (`f_9`),
  FULLTEXT KEY `idx_full_text_1` (`f_6`,`f_7`,`f_8`),
  FULLTEXT KEY `idx_full_text_2` (`f_8`),
  KEY `idx_btree_text_1` (`f_10`),
  KEY `idx_btree_text_2` (`f_11`),
  KEY `idx_btree_text_3` (`f_13`,`f_12`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8

struct_it_mysql2mysql_1.constraint_table
CREATE TABLE `constraint_table` (
  `id` int(11) NOT NULL AUTO_INCREMENT,
  `username` varchar(50) NOT NULL,
  `password` varchar(50) NOT NULL,
  `email` varchar(100) NOT NULL,
  `age` int(11) DEFAULT NULL,
  `gender` enum('Male','Female','Other') NOT NULL,
  `created_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `username` (`username`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8
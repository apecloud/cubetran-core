DROP schema IF EXISTS precheck_it cascade;
CREATE schema precheck_it;

CREATE TABLE precheck_it.table_1(id integer, text varchar(10),primary key (id)); 

CREATE TABLE precheck_it.table_2(id integer, text varchar(10),primary key (id)); 

CREATE TABLE precheck_it.ignore_tb_1(id integer, text varchar(10),primary key (id)); 

CREATE TABLE precheck_it.ignore_tb_2(id integer, text varchar(10),primary key (id));     



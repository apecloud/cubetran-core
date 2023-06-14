#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use serial_test::serial;

    use crate::test_runner::test_base::TestBase;

    #[tokio::test]
    #[serial]
    async fn snapshot_basic_test() {
        TestBase::run_snapshot_test("mysql_to_mysql/snapshot/basic_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_on_duplicate_test() {
        TestBase::run_snapshot_test("mysql_to_mysql/snapshot/on_duplicate_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_wildchar_test() {
        TestBase::run_snapshot_test("mysql_to_mysql/snapshot/wildchar_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_charset_test() {
        TestBase::run_snapshot_test("mysql_to_mysql/snapshot/charset_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_special_character_in_name_test() {
        let mut dst_expected_counts = HashMap::new();
        dst_expected_counts.insert("`test_db_*.*`.`one_pk_no_uk_1_*.*`", 0);
        dst_expected_counts.insert("`test_db_*.*`.`one_pk_no_uk_2_*.*`", 0);
        dst_expected_counts.insert("`test_db_&.&`.`one_pk_no_uk_1_&.&`", 0);
        dst_expected_counts.insert("`test_db_&.&`.`one_pk_no_uk_2_&.&`", 0);
        dst_expected_counts.insert("`test_db_^.^`.`one_pk_no_uk_1_^.^`", 0);
        dst_expected_counts.insert("`test_db_^.^`.`one_pk_no_uk_2_^.^`", 2);
        dst_expected_counts.insert("`test_db_@.@`.`one_pk_no_uk_1_@.@`", 0);
        dst_expected_counts.insert("`test_db_@.@`.`one_pk_no_uk_2_@.@`", 2);
        dst_expected_counts.insert("`*.*_test_db`.`one_pk_no_uk_1_*.*`", 0);
        dst_expected_counts.insert("`*.*_test_db`.`one_pk_no_uk_2_*.*`", 2);
        dst_expected_counts.insert("`&.&_test_db`.`one_pk_no_uk_1_&.&`", 0);
        dst_expected_counts.insert("`&.&_test_db`.`one_pk_no_uk_2_&.&`", 2);
        dst_expected_counts.insert("`^.^_test_db`.`one_pk_no_uk_1_^.^`", 0);
        dst_expected_counts.insert("`^.^_test_db`.`one_pk_no_uk_2_^.^`", 0);
        dst_expected_counts.insert("`@.@_test_db`.`one_pk_no_uk_1_@.@`", 0);
        dst_expected_counts.insert("`@.@_test_db`.`one_pk_no_uk_2_@.@`", 0);

        TestBase::run_snapshot_test_and_check_dst_count(
            "mysql_to_mysql/snapshot/special_character_in_name_test",
            dst_expected_counts,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_resume_test() {
        let mut dst_expected_counts = HashMap::new();
        dst_expected_counts.insert("test_db_1.no_pk_no_uk", 9);
        // no_pk_one_uk has a uk with multiple cols, UNIQUE KEY uk_1 (f_1,f_2), resume_filter won't work
        dst_expected_counts.insert("test_db_1.no_pk_one_uk", 9);
        // resume_filter works
        dst_expected_counts.insert("test_db_1.one_pk_multi_uk", 4);
        dst_expected_counts.insert("test_db_1.one_pk_no_uk", 4);
        // with special characters in db && tb && col names
        dst_expected_counts.insert("test_db_@.resume_table_*$4", 1);

        TestBase::run_snapshot_test_and_check_dst_count(
            "mysql_to_mysql/snapshot/resume_test",
            dst_expected_counts,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_timezone_test() {
        println!("snapshot_timezone_test can be covered by test: cdc_basic_test, table: one_pk_no_uk, field: f_13 timestamp(6), the default_time_zone for source db is +08:00, the default_time_zone for target db is +07:00 ")
    }
}

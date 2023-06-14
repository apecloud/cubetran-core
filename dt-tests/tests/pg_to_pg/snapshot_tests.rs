#[cfg(test)]
mod test {

    use std::collections::HashMap;

    use serial_test::serial;

    use crate::test_runner::test_base::TestBase;

    #[tokio::test]
    #[serial]
    async fn snapshot_basic_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/basic_test").await;
    }

    /// dst table already has records with same primary keys of src table,
    /// src data should be synced to dst table by "ON CONFLICT (pk) DO UPDATE SET"
    #[tokio::test]
    #[serial]
    async fn snapshot_on_duplicate_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/on_duplicate_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_wildchar_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/wildchar_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_postgis_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/postgis_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_postgis_array_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/postgis_array_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_charset_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/charset_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_charset_euc_cn_test() {
        TestBase::run_snapshot_test("pg_to_pg/snapshot/charset_euc_cn_test").await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_resume_test() {
        let mut dst_expected_counts = HashMap::new();
        dst_expected_counts.insert("public.resume_table_1", 1);
        dst_expected_counts.insert("public.resume_table_2", 1);
        dst_expected_counts.insert("public.resume_table_3", 2);
        dst_expected_counts.insert("public.resume_table_*$4", 1);
        dst_expected_counts.insert(r#""test_db_*.*"."resume_table_*$5""#, 1);

        TestBase::run_snapshot_test_and_check_dst_count(
            "pg_to_pg/snapshot/resume_test",
            dst_expected_counts,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_special_character_in_name_test() {
        let mut dst_expected_counts = HashMap::new();
        dst_expected_counts.insert(r#""test_db_*.*"."one_pk_no_uk_1_*.*""#, 0);
        dst_expected_counts.insert(r#""test_db_*.*"."one_pk_no_uk_2_*.*""#, 0);
        dst_expected_counts.insert(r#""test_db_&.&"."one_pk_no_uk_1_&.&""#, 0);
        dst_expected_counts.insert(r#""test_db_&.&"."one_pk_no_uk_2_&.&""#, 0);
        dst_expected_counts.insert(r#""test_db_^.^"."one_pk_no_uk_1_^.^""#, 0);
        dst_expected_counts.insert(r#""test_db_^.^"."one_pk_no_uk_2_^.^""#, 2);
        dst_expected_counts.insert(r#""test_db_@.@"."one_pk_no_uk_1_@.@""#, 0);
        dst_expected_counts.insert(r#""test_db_@.@"."one_pk_no_uk_2_@.@""#, 2);
        dst_expected_counts.insert(r#""*.*_test_db"."one_pk_no_uk_1_*.*""#, 0);
        dst_expected_counts.insert(r#""*.*_test_db"."one_pk_no_uk_2_*.*""#, 2);
        dst_expected_counts.insert(r#""&.&_test_db"."one_pk_no_uk_1_&.&""#, 0);
        dst_expected_counts.insert(r#""&.&_test_db"."one_pk_no_uk_2_&.&""#, 2);
        dst_expected_counts.insert(r#""^.^_test_db"."one_pk_no_uk_1_^.^""#, 0);
        dst_expected_counts.insert(r#""^.^_test_db"."one_pk_no_uk_2_^.^""#, 0);
        dst_expected_counts.insert(r#""@.@_test_db"."one_pk_no_uk_1_@.@""#, 0);
        dst_expected_counts.insert(r#""@.@_test_db"."one_pk_no_uk_2_@.@""#, 0);

        TestBase::run_snapshot_test_and_check_dst_count(
            "pg_to_pg/snapshot/special_character_in_name_test",
            dst_expected_counts,
        )
        .await;
    }

    #[tokio::test]
    #[serial]
    async fn snapshot_timezone_test() {
        println!("snapshot_timezone_test can be covered by test: snapshot_basic_test, table: timezone_table, the default_time_zone for source db is +08:00, the default_time_zone for target db is +07:00")
    }
}

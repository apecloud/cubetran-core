use strum::{AsStaticStr, Display, EnumString, IntoStaticStr};

#[derive(Display, EnumString, IntoStaticStr, AsStaticStr)]
pub enum ExtractType {
    #[strum(serialize = "snapshot")]
    Snapshot,
    #[strum(serialize = "cdc")]
    Cdc,
    #[strum(serialize = "check_log")]
    CheckLog,
    #[strum(serialize = "struct")]
    Struct,
}

#[derive(EnumString, AsStaticStr)]
pub enum SinkType {
    #[strum(serialize = "write")]
    Write,
    #[strum(serialize = "check")]
    Check,
    #[strum(serialize = "struct")]
    Struct,
}

#[derive(EnumString, AsStaticStr)]
pub enum ParallelType {
    #[strum(serialize = "snapshot")]
    Snapshot,
    #[strum(serialize = "rdb_partition")]
    RdbPartition,
    #[strum(serialize = "rdb_merge")]
    RdbMerge,
    #[strum(serialize = "rdb_check")]
    RdbCheck,
}
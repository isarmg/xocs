//! Installation metadata descriptors retained for the current blog contract.
pub const PLATFORM_GENERATION: u32 = 1;
pub const PLATFORM_SCHEMA_REVISION: u32 = 1;
pub const PLATFORM_METADATA_DDL: &str = include_str!("../../schemas/platform/v1.sql");
pub const PLATFORM_METADATA_INSERT_SQL: &str = "INSERT INTO _common_platform_metadata(singleton,platform_generation,platform_schema_revision,profile,created_at_micros) VALUES(1,?,?,?,?)";
#[allow(dead_code)]
pub const PLATFORM_METADATA_READ_SQL: &str = "SELECT typeof(singleton), singleton, typeof(platform_generation), platform_generation, typeof(platform_schema_revision), platform_schema_revision, typeof(profile), profile, typeof(created_at_micros), created_at_micros FROM _common_platform_metadata ORDER BY singleton LIMIT 2";
pub const PLATFORM_METADATA_CHECK_SQL: &str = "SELECT COUNT(*)=1 AND COALESCE(MIN(typeof(singleton)='integer' AND singleton=1 AND typeof(platform_generation)='integer' AND platform_generation=? AND typeof(platform_schema_revision)='integer' AND platform_schema_revision=? AND typeof(profile)='text' AND profile=? AND typeof(created_at_micros)='integer' AND created_at_micros>=0),0) FROM _common_platform_metadata";

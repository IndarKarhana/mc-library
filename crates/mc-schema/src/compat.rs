use crate::SUPPORTED_SCHEMA_VERSION;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityReport {
    pub supported: bool,
    pub required_version: String,
    pub found_version: String,
    pub reason: Option<String>,
}

pub fn check_schema_compatibility(version: &str) -> CompatibilityReport {
    let required = parse_major_minor(SUPPORTED_SCHEMA_VERSION);
    let found = parse_major_minor(version);

    match (required, found) {
        (Some((req_major, req_minor)), Some((found_major, found_minor))) => {
            if found_major != req_major {
                return CompatibilityReport {
                    supported: false,
                    required_version: SUPPORTED_SCHEMA_VERSION.to_string(),
                    found_version: version.to_string(),
                    reason: Some("schema major version mismatch".to_string()),
                };
            }

            if found_minor > req_minor {
                return CompatibilityReport {
                    supported: false,
                    required_version: SUPPORTED_SCHEMA_VERSION.to_string(),
                    found_version: version.to_string(),
                    reason: Some("schema minor version is newer than runtime support".to_string()),
                };
            }

            CompatibilityReport {
                supported: true,
                required_version: SUPPORTED_SCHEMA_VERSION.to_string(),
                found_version: version.to_string(),
                reason: None,
            }
        }
        _ => CompatibilityReport {
            supported: false,
            required_version: SUPPORTED_SCHEMA_VERSION.to_string(),
            found_version: version.to_string(),
            reason: Some("invalid schema version format; expected <major>.<minor>".to_string()),
        },
    }
}

fn parse_major_minor(version: &str) -> Option<(u64, u64)> {
    let mut parts = version.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next()?.parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor))
}

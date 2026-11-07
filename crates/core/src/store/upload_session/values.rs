//! String encoding for the storage policy type column.

use cloudreve_sdk_api::models::explorer::PolicyType;

use crate::store::CatalogError;

pub(crate) fn policy_type_to_str(value: &PolicyType) -> &'static str {
    match value {
        PolicyType::Local => "local",
        PolicyType::Remote => "remote",
        PolicyType::Oss => "oss",
        PolicyType::Qiniu => "qiniu",
        PolicyType::Onedrive => "onedrive",
        PolicyType::Cos => "cos",
        PolicyType::Upyun => "upyun",
        PolicyType::S3 => "s3",
        PolicyType::Ks3 => "ks3",
        PolicyType::Obs => "obs",
    }
}

pub(crate) fn policy_type_from_str(value: &str) -> Result<PolicyType, CatalogError> {
    let policy_type = match value {
        "local" => PolicyType::Local,
        "remote" => PolicyType::Remote,
        "oss" => PolicyType::Oss,
        "qiniu" => PolicyType::Qiniu,
        "onedrive" => PolicyType::Onedrive,
        "cos" => PolicyType::Cos,
        "upyun" => PolicyType::Upyun,
        "s3" => PolicyType::S3,
        "ks3" => PolicyType::Ks3,
        "obs" => PolicyType::Obs,
        other => {
            return Err(CatalogError::DecodeFailed {
                cause: format!("无法识别的存储策略类型: {other}"),
            });
        }
    };
    Ok(policy_type)
}

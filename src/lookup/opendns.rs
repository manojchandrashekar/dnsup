use super::dig::execute;
use crate::{util::vlog, AppConfig, UserConfig};

pub fn validate(user_config: &mut UserConfig, app_config: &AppConfig) {
    user_config.ip = Some(execute(
        "+short".to_string(),
        "myip.opendns.com".to_string(),
        "resolver1.opendns.com".to_string(),
    ));
    vlog(
        format!("IP: {}", user_config.ip.as_ref().unwrap()).as_str(),
        app_config,
    )
}

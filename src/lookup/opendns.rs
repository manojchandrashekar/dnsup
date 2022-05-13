use super::dig::execute;
use crate::UserConfig;

pub fn validate(user_config: &mut UserConfig) {
    user_config.ip = execute(
        "+short".to_string(),
        "myip.opendns.com".to_string(),
        "resolver1.opendns.com".to_string(),
    );
    log::info!("IP: {}", user_config.ip);
}

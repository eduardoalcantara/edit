//! Persistência e modelo de variáveis de ambiente (Usuário / Sistema).

mod conf;
mod store;

pub use conf::{parse_conf, serialize_conf, validate_var_name, EnvVar};
pub use store::{
    install_system_conf_elevated, load_scope, save_scope, system_conf_path, user_conf_path,
    EnvScope, EnvStoreError,
};

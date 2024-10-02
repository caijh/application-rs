use std::fmt::Display;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DbConnection {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub name: String,
    pub kind: String,
    pub args: Option<String>,
}

impl Display for DbConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut db_url = self.kind.to_string()
            + "://"
            + &self.user
            + ":"
            + &self.password
            + "@"
            + &self.host
            + ":"
            + &self.port.to_string()
            + "/"
            + &self.name;

        if let Some(args) = &self.args {
            db_url = db_url + "?" + args;
        }
        write!(f, "{}", db_url)
    }
}

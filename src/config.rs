#[derive(Debug, Clone)]
pub(crate) struct Config {
    enable_cli: bool,
    enable_ws: bool,
    http_host: String,
    http_port: u16,
    db_path: String,
    // TODO: Add entry to disable developer documentation endpoints
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_cli: false,
            enable_ws: true,
            http_host: "0.0.0.0".to_string(),
            http_port: 8080,
            db_path: "./db/data.db".to_string(),
        }
    }
}

impl Config {
    pub(crate) fn is_cli_enabled(&self) -> bool {
        self.enable_cli
    }

    pub(crate) fn is_ws_enabled(&self) -> bool {
        self.enable_ws
    }

    pub(crate) fn http_host(&self) -> String {
        self.http_host.clone()
    }

    pub(crate) fn http_port(&self) -> u16 {
        self.http_port
    }

    pub(crate) fn db_path(&self) -> String {
        self.db_path.clone()
    }
}

#[derive(Debug, Default)]
pub(crate) struct ConfigBuilder {
    enable_cli: Option<bool>,
    enable_ws: Option<bool>,
    http_host: Option<String>,
    http_port: Option<u16>,
    db_path: Option<String>,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn enable_cli(mut self, enable: bool) -> Self {
        self.enable_cli = Some(enable);

        self
    }

    pub(crate) fn enable_ws(mut self, enable: bool) -> Self {
        self.enable_ws = Some(enable);
        self
    }

    pub(crate) fn http_host(mut self, host: &str) -> Self {
        self.http_host = Some(host.to_string());
        self
    }

    pub(crate) fn http_port(mut self, port: u16) -> Self {
        self.http_port = Some(port);
        self
    }

    pub(crate) fn db_path(mut self, path: &str) -> Self {
        self.db_path = Some(path.to_string());
        self
    }

    pub(crate) fn build(self) -> Config {
        let mut the_config = Config::default();

        the_config.enable_cli = self.enable_cli.unwrap_or(the_config.enable_cli);
        the_config.enable_ws = self.enable_ws.unwrap_or(the_config.enable_ws);
        the_config.http_host = self.http_host.unwrap_or(the_config.http_host);
        the_config.http_port = self.http_port.unwrap_or(the_config.http_port);

        the_config
    }
}

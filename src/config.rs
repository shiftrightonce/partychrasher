#![allow(dead_code)]

#[derive(Debug, Clone)]
pub(crate) struct Config {
    enable_cli: bool,
    enable_ws: bool,
    enable_web: bool,
    http_host: String,
    http_port: u16,
    db_path: String,
    static_path: String,
    audio_format: String,
    video_format: String,
    photo_format: String,
    // TODO: Add entry to disable developer documentation endpoints
}

impl Default for Config {
    fn default() -> Self {
        Self {
            enable_cli: false,
            enable_ws: true,
            enable_web: true,
            http_host: if let Ok(host) = std::env::var("PARTY_HTTP_HOST") {
                host
            } else {
                "127.0.0.1".to_string()
            },
            http_port: if let Ok(port) = std::env::var("PARTY_HTTP_PORT") {
                port.parse().unwrap_or(8080)
            } else {
                8080
            },
            db_path: if let Ok(db_path) = std::env::var("PARTY_DB_LOCATION") {
                db_path
            } else {
                "./db".to_string()
            },
            static_path: if let Ok(static_path) = std::env::var("PARTY_STATIC_LOCATION") {
                static_path
            } else {
                "./static".to_string()
            },
            audio_format: if let Ok(format) = std::env::var("PARTY_AUDIO_FORMAT") {
                format
            } else {
                "mp3,aac,m4a,wav,ogg,wma,webm,flac".to_string()
            },

            video_format: if let Ok(format) = std::env::var("PARTY_VIDEO_FORMAT") {
                format
            } else {
                "mp4".to_string()
            },
            photo_format: if let Ok(format) = std::env::var("PARTY_PHOTO_FORMAT") {
                format
            } else {
                "jpg,png,gif".to_string()
            },
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

    pub(crate) fn is_web_enabled(&self) -> bool {
        self.enable_web
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
    pub(crate) fn static_path(&self) -> String {
        self.static_path.clone()
    }

    pub(crate) fn artwork_path(&self) -> String {
        format!("{}/{}", self.static_path, "artwork")
    }

    pub(crate) fn audio_format(&self) -> Vec<&str> {
        self.audio_format.split(',').collect::<Vec<&str>>()
    }
    pub(crate) fn video_format(&self) -> Vec<&str> {
        self.video_format.split(',').collect::<Vec<&str>>()
    }

    pub(crate) fn photo_format(&self) -> Vec<&str> {
        self.photo_format.split(',').collect::<Vec<&str>>()
    }
}

#[derive(Debug, Default)]
pub(crate) struct ConfigBuilder {
    enable_cli: Option<bool>,
    enable_ws: Option<bool>,
    enable_web: Option<bool>,
    http_host: Option<String>,
    http_port: Option<u16>,
    db_path: Option<String>,
    static_path: Option<String>,
    audio_format: Option<String>,
    video_format: Option<String>,
    photo_format: Option<String>,
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

    pub(crate) fn enable_web(mut self, enable: bool) -> Self {
        self.enable_web = Some(enable);
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

    pub(crate) fn set_static_path(mut self, path: &str) -> Self {
        self.static_path = Some(path.to_string());
        self
    }
    pub(crate) fn set_audio_formats(mut self, formats: &[&str]) -> Self {
        self.audio_format = Some(
            formats
                .iter()
                .fold(String::new(), |acc, add| format!("{}{}", acc, add)),
        );
        self
    }
    pub(crate) fn set_video_formats(mut self, formats: &[&str]) -> Self {
        self.video_format = Some(
            formats
                .iter()
                .fold(String::new(), |acc, add| format!("{}{}", acc, add)),
        );
        self
    }
    pub(crate) fn set_photo_formats(mut self, formats: &[&str]) -> Self {
        self.photo_format = Some(
            formats
                .iter()
                .fold(String::new(), |acc, add| format!("{}{}", acc, add)),
        );
        self
    }

    pub(crate) fn build(self) -> Config {
        let mut the_config = Config::default();

        the_config.enable_cli = self.enable_cli.unwrap_or(the_config.enable_cli);
        the_config.enable_ws = self.enable_ws.unwrap_or(the_config.enable_ws);
        the_config.enable_web = self.enable_web.unwrap_or(the_config.enable_web);
        the_config.db_path = self.db_path.unwrap_or(the_config.db_path);
        the_config.http_host = self.http_host.unwrap_or(the_config.http_host);
        the_config.http_port = self.http_port.unwrap_or(the_config.http_port);
        the_config.static_path = self.static_path.unwrap_or(the_config.static_path);
        the_config.audio_format = self.audio_format.unwrap_or(the_config.audio_format);
        the_config.video_format = self.video_format.unwrap_or(the_config.video_format);
        the_config.photo_format = self.photo_format.unwrap_or(the_config.photo_format);

        the_config
    }
}

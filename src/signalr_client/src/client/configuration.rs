#[derive(Clone)]
pub(crate) enum Authentication {
    None,
    Basic {
        user: String,
        password: Option<String>,
    },
    Bearer {
        token: String,
    },
} 

pub struct ConnectionConfiguration {
    _secure: bool,
    _domain: String,
    _hub: String,
    _port: Option<i32>,
    _authentication: Authentication,
    _query_params: Vec<(String, String)>,
}

impl ConnectionConfiguration {
    pub(crate) fn new(domain: String, hub: String) -> Self {
        ConnectionConfiguration {
            _authentication: Authentication::None,
            _domain: domain,
            _secure: true,
            _hub: hub,
            _port: None,
            _query_params: Vec::new(),
        }
    }

    /// Sets the port for the connection.
    ///
    /// # Arguments
    ///
    /// * `port` - An integer specifying the port number to use for the connection.
    ///
    /// # Returns
    ///
    /// * `&ConnectionConfiguration` - Returns a reference to the updated connection configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SignalRClient::connect_with("localhost", "test", |c| {
    ///     c.with_port(5220);
    /// }).await.unwrap();
    /// ```
    pub fn with_port(&mut self, port: i32) -> &ConnectionConfiguration {
        self._port = Some(port);

        self
    }

    /// Sets the hub name for the connection.
    ///
    /// # Arguments
    ///
    /// * `hub` - A `String` specifying the name of the hub to connect to.
    ///
    /// # Returns
    ///
    /// * `&ConnectionConfiguration` - Returns a reference to the updated connection configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SignalRClient::connect_with("localhost", "test", |c| {
    ///     c.with_hub("myHub".to_string());
    /// }).await.unwrap();
    /// ```
    pub fn with_hub(&mut self, hub: String) -> &ConnectionConfiguration {
        self._hub = hub;

        self
    }

    /// Configures the connection to use a secure (HTTPS) protocol.
    ///
    /// # Returns
    ///
    /// * `&ConnectionConfiguration` - Returns a reference to the updated connection configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SignalRClient::connect_with("localhost", "test", |c| {
    ///     c.secure();
    /// }).await.unwrap();
    /// ```    
    pub fn secure(&mut self) -> &ConnectionConfiguration {
        self._secure = true;

        self
    }

    /// Configures the connection to use an unsecure (HTTP) protocol.
    ///
    /// # Returns
    ///
    /// * `&ConnectionConfiguration` - Returns a reference to the updated connection configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SignalRClient::connect_with("localhost", "test", |c| {
    ///     c.unsecure();
    /// }).await.unwrap();
    /// ```    
    pub fn unsecure(&mut self) -> &ConnectionConfiguration {
        self._secure = false;

        self
    }

    /// Configures the connection to use basic authentication.
    ///
    /// # Arguments
    ///
    /// * `user` - A `String` specifying the username for authentication.
    /// * `password` - An `Option<String>` specifying the password for authentication. If `None`, no password is used.
    ///
    /// # Returns
    ///
    /// * `&ConnectionConfiguration` - Returns a reference to the updated connection configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SignalRClient::connect_with("localhost", "test", |c| {
    ///     c.authenticate_basic("username".to_string(), Some("password".to_string()));
    /// }).await.unwrap();
    /// ```    
    pub fn authenticate_basic(&mut self, user: String, password: Option<String>) -> &ConnectionConfiguration {
        self._authentication = Authentication::Basic { user: user, password: password };

        self
    }

    /// Configures the connection to use bearer token authentication.
    ///
    /// # Arguments
    ///
    /// * `token` - A `String` specifying the bearer token for authentication.
    ///
    /// # Returns
    ///
    /// * `&ConnectionConfiguration` - Returns a reference to the updated connection configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = SignalRClient::connect_with("localhost", "test", |c| {
    ///     c.authenticate_bearer("your_bearer_token".to_string());
    /// }).await.unwrap();
    /// ```    
    pub fn authenticate_bearer(&mut self, token: String) -> &ConnectionConfiguration {
        self._authentication = Authentication::Bearer { token: token };

        self
    }

    pub fn with_query_param(&mut self, key: String, value: String) -> &ConnectionConfiguration {
        self._query_params.push((key, value));
        self
    }

    pub fn with_access_token(&mut self, token: String) -> &ConnectionConfiguration {
        self.with_query_param("access_token".to_string(), token)
    }

    pub(crate) fn get_web_url(&self) -> String {
        let base_url = format!("{}://{}/{}", self.get_http_schema(), self.get_domain(), self._hub);
        if self._query_params.is_empty() {
            base_url
        } else {
            let params = self._query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", base_url, params)
        }
    }

    pub(crate) fn get_socket_url(&self) -> String {
        let base_url = format!("{}://{}/{}", self.get_socket_schema(), self.get_domain(), self._hub);
        if self._query_params.is_empty() {
            base_url
        } else {
            let params = self._query_params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            format!("{}?{}", base_url, params)
        }
    }

    pub(crate) fn get_negotiate_url(&self) -> String {
        let mut url = self.get_web_url();
        // Nếu đã có query params, thêm negotiate với &, ngược lại thêm với ?
        if url.contains('?') {
            url = format!("{}&negotiate", url);
        } else {
            url = format!("{}/negotiate", url);
        }
        url
    }

    pub(crate) fn get_authentication(&self) -> Authentication {
        self._authentication.clone()
    }

    fn get_http_schema(&self) -> String {
        if self._secure {
            "https".to_string()
        } else {
            "http".to_string()
        }
    }

    fn get_socket_schema(&self) -> String {
        if self._secure {
            "wss".to_string()
        } else {
            "ws".to_string()
        }
    }

    fn get_domain(&self) -> String {
        match self._port {
            Some(port) => format!("{}:{}", self._domain, port),
            None => self._domain.clone()
        }
    }
}
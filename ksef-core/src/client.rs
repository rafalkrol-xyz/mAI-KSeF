//! KSeF API client.
//!
//! STATUS: scaffolding. The environment split, token handling, and session
//! lifecycle are modeled here, but the actual HTTP wire calls are not yet
//! implemented — they must be built against the KSeF TEST environment and
//! validated with a full submit + UPO round-trip before production use.
//!
//! The session flow for token-based auth is roughly:
//!   1. request an auth challenge
//!   2. authenticate with the KSeF token -> session token
//!   3. open a session, submit the FA(3) XML (encrypted)
//!   4. poll submission status
//!   5. fetch the UPO (official confirmation) once accepted

/// Which KSeF environment to talk to.
///
/// Defaults to [`Environment::Test`] everywhere so a real invoice is never
/// issued by accident — production requires an explicit opt-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Environment {
    /// KSeF test environment. Safe for experiments; invoices are not legal.
    #[default] Test,
    /// KSeF production. Invoices issued here are legally binding.
    Production,
}

impl Environment {
    /// Base URL for the environment's API.
    pub fn base_url(self) -> &'static str {
        match self {
            Environment::Test => "https://ksef-test.mf.gov.pl/api/v2",
            Environment::Production => "https://api.ksef.mf.gov.pl/v2",
        }
    }

    pub fn is_production(self) -> bool {
        matches!(self, Environment::Production)
    }
}

/// A KSeF authentication token (from the portal or token API).
///
/// Wrapped so it is never accidentally printed; [`std::fmt::Debug`] is redacted.
#[derive(Clone)]
pub struct Token(String);

impl Token {
    pub fn new(raw: impl Into<String>) -> Self {
        Token(raw.into())
    }

    /// Read the token from the `KSEF_TOKEN` environment variable.
    pub fn from_env() -> anyhow::Result<Self> {
        let raw = std::env::var("KSEF_TOKEN")
            .map_err(|_| anyhow::anyhow!("KSEF_TOKEN environment variable is not set"))?;
        if raw.trim().is_empty() {
            anyhow::bail!("KSEF_TOKEN is set but empty");
        }
        Ok(Token(raw))
    }

    /// Borrow the raw token. Use only when handing it to the API.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Token(***redacted***)")
    }
}

/// A KSeF client bound to an environment and token.
#[derive(Debug)]
pub struct Client {
    env: Environment,
    #[allow(dead_code)]
    token: Token,
}

impl Client {
    pub fn new(env: Environment, token: Token) -> Self {
        Client { env, token }
    }

    pub fn environment(&self) -> Environment {
        self.env
    }

    /// Verify connectivity and that the token authenticates.
    ///
    /// TODO: implement against KSeF TEST (auth challenge -> session token).
    pub fn verify_auth(&self) -> anyhow::Result<()> {
        anyhow::bail!(
            "KSeF auth is not implemented yet; next milestone is the TEST-environment session flow"
        )
    }

    /// Submit FA(3) XML and return a submission reference.
    ///
    /// TODO: implement session open + encrypted submit against KSeF TEST.
    pub fn submit(&self, _fa3_xml: &str) -> anyhow::Result<String> {
        anyhow::bail!(
            "KSeF submit is not implemented yet; validate FA(3) XML on TEST before wiring this up"
        )
    }
}

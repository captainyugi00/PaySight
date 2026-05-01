use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

/// Errors surfaced by the PaySight detection engine.
#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid target URL `{input}`: {source}")]
    InvalidTarget {
        input: String,
        #[source]
        source: url::ParseError,
    },

    #[error("invalid proxy URL `{input}`: {source}")]
    InvalidProxy {
        input: String,
        #[source]
        source: url::ParseError,
    },

    #[error("proxy pool is empty — at least one proxy is required when proxies are configured")]
    EmptyProxyPool,

    #[error("failed to build HTTP client: {0}")]
    ClientBuild(String),

    #[error("network error while fetching `{url}`: {source}")]
    Network {
        url: String,
        #[source]
        source: wreq::Error,
    },

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error: {0}")]
    Config(String),
}

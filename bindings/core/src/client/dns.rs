//! DNS resolution via the hickory-resolver crate.

use std::{net::SocketAddr, sync::LazyLock};

use hickory_resolver::{
    TokioResolver,
    config::{LookupIpStrategy, ResolverConfig},
    lookup_ip::LookupIpIntoIter,
    name_server::TokioConnectionProvider,
};
use wreq::dns::{Addrs, Name, Resolve, Resolving};

/// Wrapper around a [`TokioResolver`], which implements the `Resolve` trait.
#[derive(Debug, Clone)]
pub struct HickoryDnsResolver {
    /// Shared, lazily-initialized Tokio-based DNS resolver.
    resolver: &'static LazyLock<TokioResolver>,
}

impl HickoryDnsResolver {
    /// Create a new resolver with the default configuration.
    pub fn new() -> HickoryDnsResolver {
        static RESOLVER: LazyLock<TokioResolver> = LazyLock::new(|| {
            let mut builder = match TokioResolver::builder_tokio() {
                Ok(resolver) => resolver,
                Err(err) => {
                    eprintln!("error reading DNS system conf: {}, using defaults", err);
                    TokioResolver::builder_with_config(
                        ResolverConfig::default(),
                        TokioConnectionProvider::default(),
                    )
                }
            };
            builder.options_mut().ip_strategy = LookupIpStrategy::Ipv4AndIpv6;
            builder.build()
        });

        HickoryDnsResolver {
            resolver: &RESOLVER,
        }
    }
}

struct SocketAddrs {
    iter: LookupIpIntoIter,
}

impl Resolve for HickoryDnsResolver {
    fn resolve(&self, name: Name) -> Resolving {
        let resolver = self.clone();
        Box::pin(async move {
            let lookup = resolver.resolver.lookup_ip(name.as_str()).await?;
            let addrs: Addrs = Box::new(SocketAddrs {
                iter: lookup.into_iter(),
            });
            Ok(addrs)
        })
    }
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ip_addr| SocketAddr::new(ip_addr, 0))
    }
}

#![allow(clippy::expect_used)]

use originweave_core::Origin;

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn proxy(value: &str) -> ProxyServer {
    ProxyServer::parse(value).expect("test proxy must parse")
}
use originweave_destination::{
    MAX_PAC_ORIGIN_COUNT, MAX_PROXY_SERVER_COUNT, ProxyRoute, ProxyRouteError, ProxyRouteKind,
    ProxyRoutePolicy, ProxyServer,
};

#[test]
fn all_32_proxy_authorities_are_retained_and_the_33rd_is_rejected() -> Result<(), ProxyRouteError> {
    assert_eq!(MAX_PROXY_SERVER_COUNT, 32);
    let target = origin("https://target.example");
    let proxies = (0..32)
        .map(|index| proxy(&format!("http://proxy-{index}.example:8080")))
        .collect::<Vec<_>>();
    let policy = ProxyRoutePolicy::new(false, proxies.clone(), Vec::new())?;
    assert!(!policy.allows_direct());
    for proxy_server in &proxies {
        let evidence = policy.authorize(
            &target,
            &ProxyRoute::ExplicitProxy {
                proxy_server: proxy_server.clone(),
            },
        )?;
        assert_eq!(evidence.route_kind(), ProxyRouteKind::ExplicitProxy);
        assert_eq!(evidence.target_origin(), &target);
        assert_eq!(evidence.proxy_server(), Some(proxy_server));
        assert_eq!(evidence.pac_origin(), None);
    }
    let unlisted = proxy("http://proxy-32.example:8080");
    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::ExplicitProxy {
                proxy_server: unlisted.clone(),
            },
        ),
        Err(ProxyRouteError::ProxyServerDenied {
            server: unlisted.clone(),
        })
    );
    let mut oversized = proxies;
    oversized.push(unlisted);
    assert_eq!(
        ProxyRoutePolicy::new(false, oversized, Vec::new()),
        Err(ProxyRouteError::TooManyProxyServers {
            count: 33,
            maximum: 32,
        })
    );
    Ok(())
}

#[test]
fn all_16_pac_authorities_are_retained_and_the_17th_is_rejected() -> Result<(), ProxyRouteError> {
    assert_eq!(MAX_PAC_ORIGIN_COUNT, 16);
    let target = origin("https://target.example");
    let pacs = (0..16)
        .map(|index| origin(&format!("https://pac-{index}.example")))
        .collect::<Vec<_>>();
    let policy = ProxyRoutePolicy::new(true, Vec::new(), pacs.clone())?;
    assert!(policy.allows_direct());
    for pac_origin in &pacs {
        let evidence = policy.authorize(
            &target,
            &ProxyRoute::PacDirect {
                pac_origin: pac_origin.clone(),
            },
        )?;
        assert_eq!(evidence.route_kind(), ProxyRouteKind::PacDirect);
        assert_eq!(evidence.target_origin(), &target);
        assert_eq!(evidence.proxy_server(), None);
        assert_eq!(evidence.pac_origin(), Some(pac_origin));
    }
    let unlisted = origin("https://pac-16.example");
    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::PacDirect {
                pac_origin: unlisted.clone(),
            },
        ),
        Err(ProxyRouteError::PacOriginDenied {
            origin: unlisted.clone(),
        })
    );
    let mut oversized = pacs;
    oversized.push(unlisted);
    assert_eq!(
        ProxyRoutePolicy::new(true, Vec::new(), oversized),
        Err(ProxyRouteError::TooManyPacOrigins {
            count: 17,
            maximum: 16,
        })
    );
    Ok(())
}

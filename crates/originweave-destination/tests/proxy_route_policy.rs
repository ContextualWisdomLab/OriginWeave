#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_destination::{
    MAX_PAC_ORIGIN_COUNT, MAX_PROXY_SERVER_COUNT, ProxyRoute, ProxyRouteError, ProxyRouteKind,
    ProxyRoutePolicy, ProxyServer,
};

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

fn proxy(value: &str) -> ProxyServer {
    ProxyServer::parse(value).expect("test proxy server must parse")
}

#[test]
fn direct_only_default_rejects_proxy_and_pac_routes() {
    let target = origin("https://target.example");
    let proxy = proxy("http://proxy.example:8080");
    let pac = origin("https://config.example");
    let policy = ProxyRoutePolicy::default();
    assert_eq!(policy, ProxyRoutePolicy::direct_only());

    let direct = policy
        .authorize(&target, &ProxyRoute::Direct)
        .expect("direct route must be authorized by default");
    assert_eq!(direct.route_kind(), ProxyRouteKind::Direct);
    assert_eq!(direct.target_origin(), &target);
    assert_eq!(direct.proxy_server(), None);
    assert_eq!(direct.pac_origin(), None);

    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::ExplicitProxy {
                proxy_server: proxy.clone(),
            },
        ),
        Err(ProxyRouteError::ProxyServerDenied { server: proxy })
    );
    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::PacDirect {
                pac_origin: pac.clone(),
            },
        ),
        Err(ProxyRouteError::PacOriginDenied { origin: pac })
    );

    let deny_all =
        ProxyRoutePolicy::new(false, Vec::new(), Vec::new()).expect("empty policy must be bounded");
    assert_eq!(
        deny_all.authorize(&target, &ProxyRoute::Direct),
        Err(ProxyRouteError::DirectRouteDenied)
    );
}

#[test]
fn explicit_proxy_authority_uses_canonical_server_identity() {
    let target = origin("https://target.example");
    let configured_proxy = proxy("http://proxy.example:8080");
    let route_proxy = proxy("HTTP://PROXY.EXAMPLE:8080");
    let policy = ProxyRoutePolicy::new(false, vec![configured_proxy], Vec::new())
        .expect("bounded proxy policy must be valid");

    let evidence = policy
        .authorize(
            &target,
            &ProxyRoute::ExplicitProxy {
                proxy_server: route_proxy.clone(),
            },
        )
        .expect("canonical proxy server must be authorized");
    assert_eq!(evidence.route_kind(), ProxyRouteKind::ExplicitProxy);
    assert_eq!(evidence.proxy_server(), Some(&route_proxy));
    assert_eq!(evidence.pac_origin(), None);
    assert_eq!(route_proxy.as_str(), "http://proxy.example:8080");
    assert!(!policy.allows_direct());
}

#[test]
fn pac_selected_proxy_requires_both_authority_boundaries() {
    let target = origin("https://target.example");
    let proxy = proxy("socks5://proxy.example:1080");
    let pac = origin("https://config.example");
    let other_proxy = proxy("https://other-proxy.example:8443");
    let other_pac = origin("https://other-config.example");
    let policy = ProxyRoutePolicy::new(false, vec![proxy.clone()], vec![pac.clone()])
        .expect("bounded PAC policy must be valid");

    let evidence = policy
        .authorize(
            &target,
            &ProxyRoute::PacProxy {
                pac_origin: pac.clone(),
                proxy_server: proxy.clone(),
            },
        )
        .expect("PAC source and selected proxy server are separately authorized");
    assert_eq!(evidence.route_kind(), ProxyRouteKind::PacProxy);
    assert_eq!(evidence.proxy_server(), Some(&proxy));
    assert_eq!(evidence.pac_origin(), Some(&pac));

    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::PacProxy {
                pac_origin: pac.clone(),
                proxy_server: other_proxy.clone(),
            },
        ),
        Err(ProxyRouteError::ProxyServerDenied {
            server: other_proxy,
        })
    );
    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::PacProxy {
                pac_origin: other_pac.clone(),
                proxy_server: proxy,
            },
        ),
        Err(ProxyRouteError::PacOriginDenied { origin: other_pac })
    );
}

#[test]
fn pac_selected_direct_requires_pac_and_direct_authority() {
    let target = origin("https://target.example");
    let pac = origin("https://config.example");
    let allowed = ProxyRoutePolicy::new(true, Vec::new(), vec![pac.clone()])
        .expect("bounded PAC policy must be valid");
    let denied = ProxyRoutePolicy::new(false, Vec::new(), vec![pac.clone()])
        .expect("bounded PAC policy must be valid");

    let evidence = allowed
        .authorize(
            &target,
            &ProxyRoute::PacDirect {
                pac_origin: pac.clone(),
            },
        )
        .expect("PAC DIRECT needs both authorities");
    assert_eq!(evidence.route_kind(), ProxyRouteKind::PacDirect);
    assert_eq!(evidence.pac_origin(), Some(&pac));
    assert_eq!(
        denied.authorize(&target, &ProxyRoute::PacDirect { pac_origin: pac }),
        Err(ProxyRouteError::DirectRouteDenied)
    );
}

#[test]
fn policy_rejects_unbounded_authority_sets_before_authorization() {
    let proxies = (0..=MAX_PROXY_SERVER_COUNT)
        .map(|index| proxy(&format!("http://proxy-{index}.example:8080")))
        .collect::<Vec<_>>();
    let pacs = (0..=MAX_PAC_ORIGIN_COUNT)
        .map(|index| origin(&format!("https://pac-{index}.example")))
        .collect::<Vec<_>>();

    assert_eq!(
        ProxyRoutePolicy::new(false, proxies, Vec::new()),
        Err(ProxyRouteError::TooManyProxyServers {
            count: MAX_PROXY_SERVER_COUNT + 1,
            maximum: MAX_PROXY_SERVER_COUNT,
        })
    );
    assert_eq!(
        ProxyRoutePolicy::new(false, Vec::new(), pacs),
        Err(ProxyRouteError::TooManyPacOrigins {
            count: MAX_PAC_ORIGIN_COUNT + 1,
            maximum: MAX_PAC_ORIGIN_COUNT,
        })
    );
}

#[test]
fn proxy_route_errors_have_deterministic_standard_error_contracts() {
    let server = proxy("http://proxy.example:8080");
    let pac_origin = origin("https://proxy.example");
    let cases = [
        (
            ProxyRouteError::DirectRouteDenied,
            "direct proxy route is not authorized".to_owned(),
        ),
        (
            ProxyRouteError::ProxyServerDenied {
                server: server.clone(),
            },
            "proxy server is not authorized: http://proxy.example:8080".to_owned(),
        ),
        (
            ProxyRouteError::PacOriginDenied { origin: pac_origin },
            "PAC origin is not authorized: https://proxy.example".to_owned(),
        ),
        (
            ProxyRouteError::TooManyProxyServers {
                count: MAX_PROXY_SERVER_COUNT + 1,
                maximum: MAX_PROXY_SERVER_COUNT,
            },
            format!(
                "proxy server policy has {} entries; maximum is {}",
                MAX_PROXY_SERVER_COUNT + 1,
                MAX_PROXY_SERVER_COUNT
            ),
        ),
        (
            ProxyRouteError::TooManyPacOrigins {
                count: MAX_PAC_ORIGIN_COUNT + 1,
                maximum: MAX_PAC_ORIGIN_COUNT,
            },
            format!(
                "PAC origin policy has {} entries; maximum is {}",
                MAX_PAC_ORIGIN_COUNT + 1,
                MAX_PAC_ORIGIN_COUNT
            ),
        ),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
        let standard: &dyn std::error::Error = &error;
        assert!(standard.source().is_none());
    }
}
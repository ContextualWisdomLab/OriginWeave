#![allow(clippy::expect_used)]

use originweave_core::Origin;
use originweave_destination::{
    MAX_PAC_ORIGIN_COUNT, MAX_PROXY_ORIGIN_COUNT, ProxyRoute, ProxyRouteError, ProxyRouteKind,
    ProxyRoutePolicy,
};

fn origin(value: &str) -> Origin {
    Origin::parse(value).expect("test origin must parse")
}

#[test]
fn direct_only_default_rejects_proxy_and_pac_routes() {
    let target = origin("https://target.example");
    let proxy = origin("https://proxy.example:443");
    let pac = origin("https://config.example");
    let policy = ProxyRoutePolicy::default();

    let direct = policy
        .authorize(&target, &ProxyRoute::Direct)
        .expect("direct route must be authorized by default");
    assert_eq!(direct.route_kind(), ProxyRouteKind::Direct);
    assert_eq!(direct.target_origin(), &target);
    assert_eq!(direct.proxy_origin(), None);
    assert_eq!(direct.pac_origin(), None);

    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::ExplicitProxy {
                proxy_origin: proxy.clone(),
            },
        ),
        Err(ProxyRouteError::ProxyOriginDenied { origin: proxy })
    );
    assert_eq!(
        policy.authorize(&target, &ProxyRoute::PacDirect { pac_origin: pac.clone() }),
        Err(ProxyRouteError::PacOriginDenied { origin: pac })
    );
}

#[test]
fn explicit_proxy_authority_uses_canonical_origins() {
    let target = origin("https://target.example");
    let configured_proxy = origin("https://proxy.example:443");
    let route_proxy = origin("HTTPS://PROXY.EXAMPLE");
    let policy = ProxyRoutePolicy::new(false, [configured_proxy], [])
        .expect("bounded proxy policy must be valid");

    let evidence = policy
        .authorize(
            &target,
            &ProxyRoute::ExplicitProxy {
                proxy_origin: route_proxy.clone(),
            },
        )
        .expect("canonical proxy must be authorized");
    assert_eq!(evidence.route_kind(), ProxyRouteKind::ExplicitProxy);
    assert_eq!(evidence.proxy_origin(), Some(&route_proxy));
    assert_eq!(evidence.pac_origin(), None);
    assert_eq!(route_proxy.as_str(), "https://proxy.example");
    assert!(!policy.allows_direct());
}

#[test]
fn pac_selected_proxy_requires_both_authority_boundaries() {
    let target = origin("https://target.example");
    let proxy = origin("https://proxy.example");
    let pac = origin("https://config.example");
    let other_proxy = origin("https://other-proxy.example");
    let other_pac = origin("https://other-config.example");
    let policy = ProxyRoutePolicy::new(false, [proxy.clone()], [pac.clone()])
        .expect("bounded PAC policy must be valid");

    let evidence = policy
        .authorize(
            &target,
            &ProxyRoute::PacProxy {
                pac_origin: pac.clone(),
                proxy_origin: proxy.clone(),
            },
        )
        .expect("PAC source and selected proxy are separately authorized");
    assert_eq!(evidence.route_kind(), ProxyRouteKind::PacProxy);
    assert_eq!(evidence.proxy_origin(), Some(&proxy));
    assert_eq!(evidence.pac_origin(), Some(&pac));

    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::PacProxy {
                pac_origin: pac.clone(),
                proxy_origin: other_proxy.clone(),
            },
        ),
        Err(ProxyRouteError::ProxyOriginDenied {
            origin: other_proxy,
        })
    );
    assert_eq!(
        policy.authorize(
            &target,
            &ProxyRoute::PacProxy {
                pac_origin: other_pac.clone(),
                proxy_origin: proxy,
            },
        ),
        Err(ProxyRouteError::PacOriginDenied { origin: other_pac })
    );
}

#[test]
fn pac_selected_direct_requires_pac_and_direct_authority() {
    let target = origin("https://target.example");
    let pac = origin("https://config.example");
    let allowed = ProxyRoutePolicy::new(true, [], [pac.clone()])
        .expect("bounded PAC policy must be valid");
    let denied = ProxyRoutePolicy::new(false, [], [pac.clone()])
        .expect("bounded PAC policy must be valid");

    let evidence = allowed
        .authorize(&target, &ProxyRoute::PacDirect { pac_origin: pac.clone() })
        .expect("PAC DIRECT needs both authorities");
    assert_eq!(evidence.route_kind(), ProxyRouteKind::PacDirect);
    assert_eq!(evidence.pac_origin(), Some(&pac));
    assert_eq!(
        denied.authorize(&target, &ProxyRoute::PacDirect { pac_origin: pac }),
        Err(ProxyRouteError::DirectRouteDenied)
    );
}

#[test]
fn policy_rejects_unbounded_origin_sets_before_authorization() {
    let proxies = (0..=MAX_PROXY_ORIGIN_COUNT)
        .map(|index| origin(&format!("https://proxy-{index}.example")))
        .collect::<Vec<_>>();
    let pacs = (0..=MAX_PAC_ORIGIN_COUNT)
        .map(|index| origin(&format!("https://pac-{index}.example")))
        .collect::<Vec<_>>();

    assert_eq!(
        ProxyRoutePolicy::new(false, proxies, []),
        Err(ProxyRouteError::TooManyProxyOrigins {
            count: MAX_PROXY_ORIGIN_COUNT + 1,
            maximum: MAX_PROXY_ORIGIN_COUNT,
        })
    );
    assert_eq!(
        ProxyRoutePolicy::new(false, [], pacs),
        Err(ProxyRouteError::TooManyPacOrigins {
            count: MAX_PAC_ORIGIN_COUNT + 1,
            maximum: MAX_PAC_ORIGIN_COUNT,
        })
    );
}

#[test]
fn proxy_route_errors_have_deterministic_standard_error_contracts() {
    let error = ProxyRouteError::DirectRouteDenied;
    assert_eq!(error.to_string(), "direct proxy route is not authorized");
    let standard: &dyn std::error::Error = &error;
    assert!(standard.source().is_none());
}

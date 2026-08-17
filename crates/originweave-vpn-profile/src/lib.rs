//! Secret-safe normalization for bounded WireGuard and provider-neutral IKEv2 profiles.
//!
//! This crate parses configuration text into typed connectivity intent only. It never
//! executes profile hooks, mutates host routing, installs a tunnel, negotiates IKE/IPsec,
//! or retains raw private credentials in a normalized profile. Raw secrets are borrowed
//! only long enough to hand them to a caller-provided [`VpnSecretImporter`], which returns
//! an opaque [`SecretReference`].

#![forbid(unsafe_code)]

/// Maximum UTF-8 byte length accepted for one VPN profile.
pub const MAX_PROFILE_BYTES: usize = 65_536;
/// Maximum UTF-8 byte length accepted for one opaque secret reference.
pub const MAX_SECRET_REFERENCE_BYTES: usize = 512;
/// Maximum UTF-8 byte length accepted for one raw secret at the importer boundary.
pub const MAX_SECRET_BYTES: usize = 4_096;
/// Maximum number of peers accepted in one WireGuard profile.
pub const MAX_PEERS: usize = 64;
/// Maximum number of entries accepted in one comma-separated profile list.
pub const MAX_LIST_ITEMS: usize = 256;
const MAX_IKE_IDENTITY_BYTES: usize = 253;

/// A bounded opaque reference to a secret stored outside the normalized profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecretReference(String);

impl SecretReference {
    /// Create a bounded non-empty opaque reference free of whitespace and control formatting.
    pub fn new(reference: impl Into<String>) -> Result<Self, ProfileError> {
        let reference = reference.into();
        if reference.is_empty()
            || reference.len() > MAX_SECRET_REFERENCE_BYTES
            || reference.chars().any(|character| {
                character.is_whitespace()
                    || character.is_control()
                    || matches!(
                        character,
                        '\u{00ad}'
                            | '\u{061c}'
                            | '\u{200b}'..='\u{200f}'
                            | '\u{202a}'..='\u{202e}'
                            | '\u{2060}'..='\u{206f}'
                            | '\u{feff}'
                    )
            })
        {
            return Err(ProfileError::InvalidSecretReference);
        }
        Ok(Self(reference))
    }

    /// Borrow the opaque reference without exposing any imported secret value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The only raw-secret vocabulary that can cross the trusted importer boundary.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum VpnSecret<'a> {
    /// WireGuard interface `PrivateKey`.
    WireGuardPrivateKey(&'a str),
    /// WireGuard peer `PresharedKey`.
    WireGuardPresharedKey(&'a str),
    /// IKEv2 preshared key.
    Ikev2PresharedKey(&'a str),
    /// IKEv2 EAP password.
    Ikev2Password(&'a str),
}

/// Trusted boundary that moves raw profile credentials into a secret store.
///
/// Profile entry points complete a side-effect-free validation pass before calling the
/// real importer. During the import pass, OriginWeave journals every successful opaque
/// reference. If a later import fails, every journaled reference is offered back to the
/// importer for disposal in reverse order before the failure is returned. Importers that
/// persist secrets must override [`VpnSecretImporter::discard_secret`]; the default fails
/// closed so missing cleanup authority cannot be mistaken for successful rollback.
pub trait VpnSecretImporter {
    /// Import one borrowed secret and return only an opaque reference.
    fn import_secret(&mut self, secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError>;

    /// Discard one previously imported opaque reference during failed-profile rollback.
    ///
    /// The default intentionally fails closed. Persistent importers must implement
    /// disposal or provide equivalent transactional behavior behind this hook.
    fn discard_secret(&mut self, _reference: &SecretReference) -> Result<(), ProfileError> {
        Err(ProfileError::SecretCleanupFailed)
    }
}

/// Normalized VPN profile intent for a privileged platform adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VpnProfile {
    /// WireGuard connectivity intent.
    WireGuard(WireGuardProfile),
    /// IKEv2 connectivity intent.
    Ikev2(Ikev2Profile),
}

/// Secret-safe WireGuard interface profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireGuardProfile {
    /// Interface addresses expressed exactly as validated IP/CIDR strings.
    pub addresses: Vec<String>,
    /// Optional DNS server addresses from the profile.
    pub dns_servers: Vec<String>,
    /// Optional interface MTU.
    pub mtu: Option<u16>,
    /// Optional local UDP listen port.
    pub listen_port: Option<u16>,
    /// Opaque reference replacing the raw interface private key.
    pub private_key: SecretReference,
    /// Bounded peers in profile order.
    pub peers: Vec<WireGuardPeer>,
}

/// Secret-safe WireGuard peer intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WireGuardPeer {
    /// Peer public key.
    pub public_key: String,
    /// Optional opaque reference replacing a peer preshared key.
    pub preshared_key: Option<SecretReference>,
    /// Optional endpoint host and UDP port, left for destination policy authorization.
    pub endpoint: Option<String>,
    /// Allowed IP prefixes for the peer.
    pub allowed_ips: Vec<String>,
    /// Optional WireGuard persistent keepalive interval in seconds.
    pub persistent_keepalive_seconds: Option<u16>,
}

/// IKEv2 authentication intent with secret values replaced by opaque references.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Ikev2Authentication {
    /// Preshared-key authentication.
    PresharedKey(SecretReference),
    /// EAP username plus an opaque password reference.
    Eap {
        /// Provider-defined username.
        username: String,
        /// Opaque reference replacing the raw password.
        password: SecretReference,
    },
}

/// Provider-neutral IKEv2 profile intent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ikev2Profile {
    /// Remote VPN gateway host or IP, subject to OriginWeave destination authorization.
    pub server: String,
    /// Optional expected remote IKE identity.
    pub remote_id: Option<String>,
    /// Optional local IKE identity.
    pub local_id: Option<String>,
    /// Secret-safe authentication intent.
    pub authentication: Ikev2Authentication,
    /// Modern allow-listed IKE/ESP proposal identifier.
    pub proposal: String,
    /// Bounded traffic selectors represented as validated IP/CIDR strings.
    pub traffic_selectors: Vec<String>,
    /// Whether a platform adapter may negotiate MOBIKE.
    pub mobike: bool,
    /// Whether a platform adapter may negotiate IKEv2 fragmentation.
    pub fragmentation: bool,
    /// Dead-peer detection interval in seconds.
    pub dpd_seconds: u32,
    /// Rekey interval in seconds.
    pub rekey_seconds: u32,
}

/// Fail-closed profile parsing and secret-import errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProfileError {
    /// Input exceeds the bounded profile size.
    ProfileTooLarge,
    /// Input has no recognized profile section.
    UnsupportedProfile,
    /// A line is malformed or appears in an invalid section.
    MalformedLine,
    /// A key is unknown or would grant host execution/routing authority.
    UnsupportedAuthority,
    /// A singleton field was repeated.
    DuplicateField,
    /// A required field is absent.
    MissingField,
    /// A bounded value is empty or syntactically invalid.
    InvalidValue,
    /// Too many peers or list items were supplied.
    TooManyItems,
    /// A raw secret is empty or exceeds its ingestion bound.
    InvalidSecret,
    /// A trusted secret store returned an invalid opaque reference.
    InvalidSecretReference,
    /// The trusted secret importer rejected a secret.
    SecretImportFailed,
    /// Rollback of one or more already imported secret references failed.
    SecretCleanupFailed,
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ProfileTooLarge => "profile exceeds the bounded size",
            Self::UnsupportedProfile => "profile format is unsupported",
            Self::MalformedLine => "profile contains a malformed line",
            Self::UnsupportedAuthority => "profile requests unsupported authority",
            Self::DuplicateField => "profile field is duplicated",
            Self::MissingField => "profile is missing a required field",
            Self::InvalidValue => "profile field value is invalid",
            Self::TooManyItems => "profile contains too many items",
            Self::InvalidSecret => "profile secret is invalid",
            Self::InvalidSecretReference => "secret reference is invalid",
            Self::SecretImportFailed => "secret import failed",
            Self::SecretCleanupFailed => "secret cleanup failed",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ProfileError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum WireGuardSection {
    Interface,
    Peer,
}

#[derive(Default)]
struct WireGuardPeerBuilder {
    public_key: Option<String>,
    preshared_key: Option<SecretReference>,
    endpoint: Option<String>,
    allowed_ips: Option<Vec<String>>,
    persistent_keepalive_seconds: Option<u16>,
}

impl WireGuardPeerBuilder {
    fn finish(self) -> Result<WireGuardPeer, ProfileError> {
        Ok(WireGuardPeer {
            public_key: self.public_key.ok_or(ProfileError::MissingField)?,
            preshared_key: self.preshared_key,
            endpoint: self.endpoint,
            allowed_ips: self.allowed_ips.ok_or(ProfileError::MissingField)?,
            persistent_keepalive_seconds: self.persistent_keepalive_seconds,
        })
    }
}

struct ValidationImporter;

impl VpnSecretImporter for ValidationImporter {
    fn import_secret(&mut self, _secret: VpnSecret<'_>) -> Result<SecretReference, ProfileError> {
        Ok(SecretReference("secret://validation".to_owned()))
    }
}

fn bounded_profile(profile: &str) -> Result<(), ProfileError> {
    if profile.is_empty() {
        return Err(ProfileError::UnsupportedProfile);
    }
    if profile.len() > MAX_PROFILE_BYTES {
        return Err(ProfileError::ProfileTooLarge);
    }
    Ok(())
}

fn bounded_value(value: &str) -> Result<&str, ProfileError> {
    let value = value.trim();
    if value.is_empty() {
        return Err(ProfileError::InvalidValue);
    }
    Ok(value)
}

fn validate_wireguard_key(value: &str) -> Result<&str, ProfileError> {
    let bytes = value.as_bytes();
    if bytes.len() != 44 || bytes[43] != b'=' {
        return Err(ProfileError::InvalidValue);
    }
    if !bytes[..43]
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/'))
    {
        return Err(ProfileError::InvalidValue);
    }
    if !matches!(bytes[42], b'A' | b'Q' | b'g' | b'w') {
        return Err(ProfileError::InvalidValue);
    }
    Ok(value)
}

fn import_bounded_secret(
    importer: &mut dyn VpnSecretImporter,
    imported: &mut Vec<SecretReference>,
    secret: VpnSecret<'_>,
) -> Result<SecretReference, ProfileError> {
    let raw = match secret {
        VpnSecret::WireGuardPrivateKey(raw)
        | VpnSecret::WireGuardPresharedKey(raw)
        | VpnSecret::Ikev2PresharedKey(raw)
        | VpnSecret::Ikev2Password(raw) => raw,
    };
    if raw.len() > MAX_SECRET_BYTES {
        return Err(ProfileError::InvalidSecret);
    }
    if matches!(
        secret,
        VpnSecret::WireGuardPrivateKey(_) | VpnSecret::WireGuardPresharedKey(_)
    ) {
        validate_wireguard_key(raw)?;
    }
    let reference = importer
        .import_secret(secret)
        .map_err(|_| ProfileError::SecretImportFailed)?;
    imported.push(reference.clone());
    Ok(reference)
}

fn rollback_imports(
    importer: &mut dyn VpnSecretImporter,
    imported: &[SecretReference],
    error: ProfileError,
) -> ProfileError {
    let mut cleanup_failed = false;
    for reference in imported.iter().rev() {
        if importer.discard_secret(reference).is_err() {
            cleanup_failed = true;
        }
    }
    if cleanup_failed {
        ProfileError::SecretCleanupFailed
    } else {
        error
    }
}

fn split_bounded_list(value: &str) -> Result<Vec<String>, ProfileError> {
    let mut items = Vec::new();
    for item in value.split(',') {
        if items.len() == MAX_LIST_ITEMS {
            return Err(ProfileError::TooManyItems);
        }
        items.push(bounded_value(item)?.to_owned());
    }
    Ok(items)
}

fn validate_ip_network(value: &str) -> Result<(), ProfileError> {
    let (address, prefix_text) = value.split_once('/').ok_or(ProfileError::InvalidValue)?;
    let address = address
        .parse::<std::net::IpAddr>()
        .map_err(|_| ProfileError::InvalidValue)?;
    let prefix = prefix_text
        .parse::<u8>()
        .map_err(|_| ProfileError::InvalidValue)?;
    if prefix_text != prefix.to_string() {
        return Err(ProfileError::InvalidValue);
    }
    let maximum_prefix = match address {
        std::net::IpAddr::V4(_) => 32,
        std::net::IpAddr::V6(_) => 128,
    };
    if prefix > maximum_prefix {
        return Err(ProfileError::InvalidValue);
    }
    Ok(())
}

fn validate_ip_address(value: &str) -> Result<(), ProfileError> {
    value
        .parse::<std::net::IpAddr>()
        .map(|_| ())
        .map_err(|_| ProfileError::InvalidValue)
}

fn split_network_list(value: &str) -> Result<Vec<String>, ProfileError> {
    let items = split_bounded_list(value)?;
    for item in &items {
        validate_ip_network(item)?;
    }
    Ok(items)
}

fn split_wireguard_allowed_ips(value: &str) -> Result<Vec<String>, ProfileError> {
    split_bounded_list(value)?
        .into_iter()
        .map(|item| {
            if item.contains('/') {
                validate_ip_network(&item)?;
                return Ok(item);
            }
            let address = item
                .parse::<std::net::IpAddr>()
                .map_err(|_| ProfileError::InvalidValue)?;
            let prefix = match address {
                std::net::IpAddr::V4(_) => 32,
                std::net::IpAddr::V6(_) => 128,
            };
            Ok(format!("{address}/{prefix}"))
        })
        .collect()
}

fn split_ip_address_list(value: &str) -> Result<Vec<String>, ProfileError> {
    let items = split_bounded_list(value)?;
    for item in &items {
        validate_ip_address(item)?;
    }
    Ok(items)
}

fn validate_dns_hostname(value: &str) -> bool {
    value.len() <= 253
        && value.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

fn validate_gateway_host(value: &str) -> Result<&str, ProfileError> {
    let value = bounded_value(value)?;
    if value
        .chars()
        .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(ProfileError::InvalidValue);
    }
    if value.parse::<std::net::IpAddr>().is_ok() || validate_dns_hostname(value) {
        return Ok(value);
    }
    Err(ProfileError::InvalidValue)
}

fn validate_ike_identity(value: &str) -> Result<&str, ProfileError> {
    if value.len() > MAX_IKE_IDENTITY_BYTES || value.chars().any(char::is_control) {
        return Err(ProfileError::InvalidValue);
    }
    Ok(value)
}

fn parse_nonzero_udp_port(value: &str) -> Result<u16, ProfileError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ProfileError::InvalidValue);
    }
    let port = value
        .parse::<u16>()
        .map_err(|_| ProfileError::InvalidValue)?;
    if port == 0 {
        return Err(ProfileError::InvalidValue);
    }
    Ok(port)
}

fn validate_wireguard_endpoint(value: &str) -> Result<&str, ProfileError> {
    if let Some(bracketed) = value.strip_prefix('[') {
        let (host, port) = bracketed
            .split_once("]:")
            .ok_or(ProfileError::InvalidValue)?;
        host.parse::<std::net::Ipv6Addr>()
            .map_err(|_| ProfileError::InvalidValue)?;
        parse_nonzero_udp_port(port)?;
        return Ok(value);
    }

    let (host, port) = value.rsplit_once(':').ok_or(ProfileError::InvalidValue)?;
    if host.contains(':') {
        return Err(ProfileError::InvalidValue);
    }
    validate_gateway_host(host)?;
    parse_nonzero_udp_port(port)?;
    Ok(value)
}

fn parse_u16(value: &str) -> Result<u16, ProfileError> {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ProfileError::InvalidValue);
    }
    value.parse::<u16>().map_err(|_| ProfileError::InvalidValue)
}

fn parse_u32(value: &str) -> Result<u32, ProfileError> {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ProfileError::InvalidValue);
    }
    value.parse::<u32>().map_err(|_| ProfileError::InvalidValue)
}

fn parse_boolean(value: &str) -> Result<bool, ProfileError> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(ProfileError::InvalidValue),
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> Result<(), ProfileError> {
    if slot.is_some() {
        return Err(ProfileError::DuplicateField);
    }
    *slot = Some(value);
    Ok(())
}

fn extend_bounded_list(
    slot: &mut Option<Vec<String>>,
    mut values: Vec<String>,
) -> Result<(), ProfileError> {
    let current = slot.get_or_insert_default();
    if current.len() + values.len() > MAX_LIST_ITEMS {
        return Err(ProfileError::TooManyItems);
    }
    current.append(&mut values);
    Ok(())
}

fn profile_lines(profile: &str) -> impl Iterator<Item = &str> {
    profile
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn parse_assignment(line: &str) -> Result<(&str, &str), ProfileError> {
    let (key, value) = line.split_once('=').ok_or(ProfileError::MalformedLine)?;
    Ok((bounded_value(key)?, bounded_value(value)?))
}

/// Import a standard WireGuard/wg-quick-style profile without granting hook or route authority.
///
/// `PrivateKey` and `PresharedKey` values cross only [`VpnSecretImporter`]. `PreUp`,
/// `PostUp`, `PreDown`, `PostDown`, `SaveConfig`, and `Table` are explicitly rejected.
pub fn import_wireguard_profile(
    profile: &str,
    importer: &mut dyn VpnSecretImporter,
) -> Result<WireGuardProfile, ProfileError> {
    let mut validator = ValidationImporter;
    let mut validation_imports = Vec::new();
    import_wireguard_profile_once(profile, &mut validator, &mut validation_imports)?;

    let mut imported = Vec::new();
    match import_wireguard_profile_once(profile, importer, &mut imported) {
        Ok(profile) => Ok(profile),
        Err(error) => Err(rollback_imports(importer, &imported, error)),
    }
}

fn import_wireguard_profile_once(
    profile: &str,
    importer: &mut dyn VpnSecretImporter,
    imported: &mut Vec<SecretReference>,
) -> Result<WireGuardProfile, ProfileError> {
    bounded_profile(profile)?;

    let mut section: Option<WireGuardSection> = None;
    let mut addresses: Option<Vec<String>> = None;
    let mut dns_servers: Option<Vec<String>> = None;
    let mut mtu: Option<u16> = None;
    let mut listen_port: Option<u16> = None;
    let mut private_key: Option<SecretReference> = None;
    let mut peers = Vec::new();
    let mut peer: Option<WireGuardPeerBuilder> = None;

    for line in profile_lines(profile) {
        if line == "[Interface]" {
            if section.is_some() {
                return Err(ProfileError::DuplicateField);
            }
            section = Some(WireGuardSection::Interface);
            continue;
        }
        if line == "[Peer]" {
            if section.is_none() {
                return Err(ProfileError::MalformedLine);
            }
            if let Some(previous) = peer.take() {
                if peers.len() == MAX_PEERS {
                    return Err(ProfileError::TooManyItems);
                }
                peers.push(previous.finish()?);
            }
            section = Some(WireGuardSection::Peer);
            peer = Some(WireGuardPeerBuilder::default());
            continue;
        }

        let (key, value) = parse_assignment(line)?;
        match section {
            Some(WireGuardSection::Interface) => match key {
                "Address" => {
                    extend_bounded_list(&mut addresses, split_wireguard_allowed_ips(value)?)?;
                }
                "DNS" => {
                    extend_bounded_list(&mut dns_servers, split_ip_address_list(value)?)?;
                }
                "MTU" => set_once(&mut mtu, parse_u16(value)?)?,
                "ListenPort" => set_once(&mut listen_port, parse_u16(value)?)?,
                "PrivateKey" => {
                    let secret = import_bounded_secret(
                        importer,
                        imported,
                        VpnSecret::WireGuardPrivateKey(value),
                    )?;
                    set_once(&mut private_key, secret)?;
                }
                "PreUp" | "PostUp" | "PreDown" | "PostDown" | "SaveConfig" | "Table" => {
                    return Err(ProfileError::UnsupportedAuthority);
                }
                _ => return Err(ProfileError::UnsupportedAuthority),
            },
            Some(WireGuardSection::Peer) => {
                let current = peer.get_or_insert_default();
                match key {
                    "PublicKey" => {
                        set_once(
                            &mut current.public_key,
                            validate_wireguard_key(value)?.to_owned(),
                        )?;
                    }
                    "PresharedKey" => {
                        let secret = import_bounded_secret(
                            importer,
                            imported,
                            VpnSecret::WireGuardPresharedKey(value),
                        )?;
                        set_once(&mut current.preshared_key, secret)?;
                    }
                    "Endpoint" => {
                        set_once(
                            &mut current.endpoint,
                            validate_wireguard_endpoint(value)?.to_owned(),
                        )?;
                    }
                    "AllowedIPs" => {
                        set_once(
                            &mut current.allowed_ips,
                            split_wireguard_allowed_ips(value)?,
                        )?;
                    }
                    "PersistentKeepalive" => {
                        set_once(&mut current.persistent_keepalive_seconds, parse_u16(value)?)?;
                    }
                    _ => return Err(ProfileError::UnsupportedAuthority),
                }
            }
            None => return Err(ProfileError::MalformedLine),
        }
    }

    if let Some(previous) = peer {
        if peers.len() == MAX_PEERS {
            return Err(ProfileError::TooManyItems);
        }
        peers.push(previous.finish()?);
    }

    if section.is_none() {
        return Err(ProfileError::UnsupportedProfile);
    }
    Ok(WireGuardProfile {
        addresses: addresses.ok_or(ProfileError::MissingField)?,
        dns_servers: dns_servers.unwrap_or_default(),
        mtu,
        listen_port,
        private_key: private_key.ok_or(ProfileError::MissingField)?,
        peers,
    })
}

const ALLOWED_IKEV2_PROPOSALS: [&str; 3] = [
    "aes256gcm16-prfsha384-ecp384",
    "aes256gcm16-prfsha256-ecp256",
    "aes256-sha256-modp2048",
];

/// Parse the crate's provider-neutral `[IKEv2]` profile format.
///
/// The accepted keys are `Server`, `RemoteId`, `LocalId`, `Auth`, `Username`, `Psk`,
/// `Password`, `Proposal`, `TrafficSelectors`, `Mobike`, `Fragmentation`, `DpdSeconds`,
/// and `RekeySeconds`. The function validates a modern proposal allow-list and replaces
/// raw PSK/password material with opaque references before returning.
pub fn parse_ikev2_profile(
    profile: &str,
    importer: &mut dyn VpnSecretImporter,
) -> Result<Ikev2Profile, ProfileError> {
    let mut validator = ValidationImporter;
    let mut validation_imports = Vec::new();
    parse_ikev2_profile_once(profile, &mut validator, &mut validation_imports)?;

    let mut imported = Vec::new();
    match parse_ikev2_profile_once(profile, importer, &mut imported) {
        Ok(profile) => Ok(profile),
        Err(error) => Err(rollback_imports(importer, &imported, error)),
    }
}

fn parse_ikev2_profile_once(
    profile: &str,
    importer: &mut dyn VpnSecretImporter,
    imported: &mut Vec<SecretReference>,
) -> Result<Ikev2Profile, ProfileError> {
    bounded_profile(profile)?;

    let mut saw_section = false;
    let mut server: Option<String> = None;
    let mut remote_id: Option<String> = None;
    let mut local_id: Option<String> = None;
    let mut auth_kind: Option<String> = None;
    let mut username: Option<String> = None;
    let mut psk: Option<SecretReference> = None;
    let mut password: Option<SecretReference> = None;
    let mut proposal: Option<String> = None;
    let mut traffic_selectors: Option<Vec<String>> = None;
    let mut mobike: Option<bool> = None;
    let mut fragmentation: Option<bool> = None;
    let mut dpd_seconds: Option<u32> = None;
    let mut rekey_seconds: Option<u32> = None;

    for line in profile_lines(profile) {
        if line == "[IKEv2]" {
            if saw_section {
                return Err(ProfileError::DuplicateField);
            }
            saw_section = true;
            continue;
        }
        if !saw_section || line.starts_with('[') {
            return Err(ProfileError::MalformedLine);
        }
        let (key, value) = parse_assignment(line)?;
        match key {
            "Server" => set_once(&mut server, validate_gateway_host(value)?.to_owned())?,
            "RemoteId" => set_once(&mut remote_id, validate_ike_identity(value)?.to_owned())?,
            "LocalId" => set_once(&mut local_id, validate_ike_identity(value)?.to_owned())?,
            "Auth" => set_once(&mut auth_kind, value.to_owned())?,
            "Username" => set_once(&mut username, validate_ike_identity(value)?.to_owned())?,
            "Psk" => {
                let secret =
                    import_bounded_secret(importer, imported, VpnSecret::Ikev2PresharedKey(value))?;
                set_once(&mut psk, secret)?;
            }
            "Password" => {
                let secret =
                    import_bounded_secret(importer, imported, VpnSecret::Ikev2Password(value))?;
                set_once(&mut password, secret)?;
            }
            "Proposal" => {
                let candidate = value;
                if !ALLOWED_IKEV2_PROPOSALS.contains(&candidate) {
                    return Err(ProfileError::InvalidValue);
                }
                set_once(&mut proposal, candidate.to_owned())?;
            }
            "TrafficSelectors" => {
                set_once(&mut traffic_selectors, split_network_list(value)?)?;
            }
            "Mobike" => set_once(&mut mobike, parse_boolean(value)?)?,
            "Fragmentation" => set_once(&mut fragmentation, parse_boolean(value)?)?,
            "DpdSeconds" => set_once(&mut dpd_seconds, parse_u32(value)?)?,
            "RekeySeconds" => set_once(&mut rekey_seconds, parse_u32(value)?)?,
            _ => return Err(ProfileError::UnsupportedAuthority),
        }
    }

    if !saw_section {
        return Err(ProfileError::UnsupportedProfile);
    }
    let auth_kind = auth_kind.ok_or(ProfileError::MissingField)?;
    let authentication = match auth_kind.as_str() {
        "psk" => {
            if username.is_some() || password.is_some() {
                return Err(ProfileError::InvalidValue);
            }
            Ikev2Authentication::PresharedKey(psk.ok_or(ProfileError::MissingField)?)
        }
        "eap" => {
            if psk.is_some() {
                return Err(ProfileError::InvalidValue);
            }
            Ikev2Authentication::Eap {
                username: username.ok_or(ProfileError::MissingField)?,
                password: password.ok_or(ProfileError::MissingField)?,
            }
        }
        _ => return Err(ProfileError::InvalidValue),
    };

    let dpd_seconds = dpd_seconds.unwrap_or(30);
    let rekey_seconds = rekey_seconds.unwrap_or(3_600);
    if dpd_seconds == 0 || rekey_seconds < 300 || dpd_seconds >= rekey_seconds {
        return Err(ProfileError::InvalidValue);
    }

    Ok(Ikev2Profile {
        server: server.ok_or(ProfileError::MissingField)?,
        remote_id,
        local_id,
        authentication,
        proposal: proposal.ok_or(ProfileError::MissingField)?,
        traffic_selectors: traffic_selectors.ok_or(ProfileError::MissingField)?,
        mobike: mobike.unwrap_or(false),
        fragmentation: fragmentation.unwrap_or(false),
        dpd_seconds,
        rekey_seconds,
    })
}

/// Detect the bounded profile family and normalize it through the same secret importer.
pub fn parse_vpn_profile(
    profile: &str,
    importer: &mut dyn VpnSecretImporter,
) -> Result<VpnProfile, ProfileError> {
    bounded_profile(profile)?;
    let first = profile_lines(profile)
        .next()
        .ok_or(ProfileError::UnsupportedProfile)?;
    match first {
        "[Interface]" => import_wireguard_profile(profile, importer).map(VpnProfile::WireGuard),
        "[IKEv2]" => parse_ikev2_profile(profile, importer).map(VpnProfile::Ikev2),
        _ => Err(ProfileError::UnsupportedProfile),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_WIREGUARD_KEY: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";

    #[derive(Default)]
    struct RecordingImporter {
        kinds: Vec<&'static str>,
        fail: bool,
    }

    impl VpnSecretImporter for RecordingImporter {
        fn import_secret(
            &mut self,
            secret: VpnSecret<'_>,
        ) -> Result<SecretReference, ProfileError> {
            if self.fail {
                return Err(ProfileError::InvalidSecret);
            }
            let kind = match secret {
                VpnSecret::WireGuardPrivateKey(_) => "wg-private",
                VpnSecret::WireGuardPresharedKey(_) => "wg-psk",
                VpnSecret::Ikev2PresharedKey(_) => "ike-psk",
                VpnSecret::Ikev2Password(_) => "ike-password",
            };
            self.kinds.push(kind);
            Ok(SecretReference(format!(
                "secret://{kind}/{}",
                self.kinds.len()
            )))
        }
    }

    fn wireguard_profile() -> String {
        format!(
            "# controlled fixture\n[Interface]\nAddress = 10.0.0.2/32, fd00::2/128\nDNS = 1.1.1.1\nMTU = 1420\nListenPort = 51820\nPrivateKey = {VALID_WIREGUARD_KEY}\n\n[Peer]\nPublicKey = {VALID_WIREGUARD_KEY}\nPresharedKey = {VALID_WIREGUARD_KEY}\nEndpoint = vpn.example:51820\nAllowedIPs = 0.0.0.0/0, ::/0\nPersistentKeepalive = 25\n"
        )
    }

    fn ikev2_psk_profile() -> &'static str {
        "[IKEv2]\nServer = vpn.example\nRemoteId = vpn.example\nLocalId = client@example\nAuth = psk\nPsk = raw-psk\nProposal = aes256gcm16-prfsha384-ecp384\nTrafficSelectors = 10.0.0.0/8, fd00::/8\nMobike = true\nFragmentation = false\nDpdSeconds = 30\nRekeySeconds = 3600\n"
    }

    fn ikev2_eap_profile() -> &'static str {
        "[IKEv2]\nServer=vpn.example\nAuth=eap\nUsername=alice\nPassword=raw-password\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=0.0.0.0/0\n"
    }

    #[test]
    fn secret_reference_is_bounded_and_borrowable() {
        assert_eq!(
            SecretReference::new("secret://one").map(|value| value.as_str().to_owned()),
            Ok("secret://one".to_owned())
        );
        assert_eq!(
            SecretReference::new(""),
            Err(ProfileError::InvalidSecretReference)
        );
        assert_eq!(
            SecretReference::new("x".repeat(MAX_SECRET_REFERENCE_BYTES + 1)),
            Err(ProfileError::InvalidSecretReference)
        );
    }

    #[test]
    fn wireguard_profile_imports_secrets_and_preserves_connectivity_intent() {
        let mut importer = RecordingImporter::default();
        let profile = wireguard_profile();
        let result = import_wireguard_profile(&profile, &mut importer);
        assert_eq!(importer.kinds, vec!["wg-private", "wg-psk"]);
        assert_eq!(
            result.map(|profile| (profile.mtu, profile.listen_port, profile.peers.len())),
            Ok((Some(1420), Some(51820), 1))
        );
    }

    #[test]
    fn wireguard_rejects_host_authority_and_structural_ambiguity() {
        for key in [
            "PreUp",
            "PostUp",
            "PreDown",
            "PostDown",
            "SaveConfig",
            "Table",
            "Unknown",
        ] {
            let profile = format!(
                "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n{key}=x\n"
            );
            let mut importer = RecordingImporter::default();
            assert_eq!(
                import_wireguard_profile(&profile, &mut importer),
                Err(ProfileError::UnsupportedAuthority)
            );
        }
        for profile in [
            "PrivateKey=k",
            "[Peer]\nPublicKey=k\nAllowedIPs=0.0.0.0/0",
            "[Interface]\n[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k",
            "[Interface]\nbroken",
        ] {
            let mut importer = RecordingImporter::default();
            assert!(import_wireguard_profile(profile, &mut importer).is_err());
        }
    }

    #[test]
    fn wireguard_requires_interface_and_peer_fields_and_rejects_duplicates() {
        for profile in [
            "[Interface]\nPrivateKey=k",
            "[Interface]\nAddress=10.0.0.2/32",
            "[Interface]\nAddress=10.0.0.2/32\nAddress=10.0.0.3/32\nPrivateKey=k",
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nAllowedIPs=10.0.0.0/8",
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p",
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8\nPublicKey=q",
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8\nNope=x",
        ] {
            let mut importer = RecordingImporter::default();
            assert!(import_wireguard_profile(profile, &mut importer).is_err());
        }
    }

    #[test]
    fn wireguard_rejects_invalid_numbers_lists_limits_and_secret_failures() {
        for profile in [
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\nMTU=nope",
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\nListenPort=nope",
            "[Interface]\nAddress=10.0.0.2/32,,10.0.0.3/32\nPrivateKey=k",
            "[Interface]\nAddress=10.0.0.2/32\nPrivateKey=k\n[Peer]\nPublicKey=p\nAllowedIPs=10.0.0.0/8\nPersistentKeepalive=nope",
        ] {
            let mut importer = RecordingImporter::default();
            assert_eq!(
                import_wireguard_profile(profile, &mut importer),
                Err(ProfileError::InvalidValue)
            );
        }
        let list = std::iter::repeat_n("10.0.0.2/32", MAX_LIST_ITEMS + 1)
            .collect::<Vec<_>>()
            .join(",");
        let profile = format!("[Interface]\nAddress={list}\nPrivateKey={VALID_WIREGUARD_KEY}");
        assert_eq!(
            import_wireguard_profile(&profile, &mut RecordingImporter::default()),
            Err(ProfileError::TooManyItems)
        );
        let secret = "x".repeat(MAX_SECRET_BYTES + 1);
        let profile = format!("[Interface]\nAddress=10.0.0.2/32\nPrivateKey={secret}");
        assert_eq!(
            import_wireguard_profile(&profile, &mut RecordingImporter::default()),
            Err(ProfileError::InvalidSecret)
        );
        let mut importer = RecordingImporter {
            kinds: Vec::new(),
            fail: true,
        };
        let profile = format!("[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}");
        assert_eq!(
            import_wireguard_profile(&profile, &mut importer),
            Err(ProfileError::SecretImportFailed)
        );
    }

    #[test]
    fn wireguard_enforces_peer_bound() {
        let mut profile =
            format!("[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n");
        for _ in 0..=MAX_PEERS {
            profile.push_str("[Peer]\n");
            profile.push_str(&format!("PublicKey={VALID_WIREGUARD_KEY}\n"));
            profile.push_str("AllowedIPs=10.0.0.0/8\n");
        }
        assert_eq!(
            import_wireguard_profile(&profile, &mut RecordingImporter::default()),
            Err(ProfileError::TooManyItems)
        );
    }

    #[test]
    fn ikev2_psk_and_eap_profiles_are_secret_safe() {
        let mut psk_importer = RecordingImporter::default();
        let psk = parse_ikev2_profile(ikev2_psk_profile(), &mut psk_importer);
        assert_eq!(psk_importer.kinds, vec!["ike-psk"]);
        assert_eq!(
            psk.map(|profile| {
                (
                    profile.mobike,
                    profile.fragmentation,
                    profile.dpd_seconds,
                    profile.rekey_seconds,
                    profile.authentication,
                )
            }),
            Ok((
                true,
                false,
                30,
                3600,
                Ikev2Authentication::PresharedKey(SecretReference("secret://ike-psk/1".to_owned())),
            ))
        );

        let mut eap_importer = RecordingImporter::default();
        let eap = parse_ikev2_profile(ikev2_eap_profile(), &mut eap_importer);
        assert_eq!(eap_importer.kinds, vec!["ike-password"]);
        assert_eq!(
            eap.map(|profile| {
                (
                    profile.mobike,
                    profile.fragmentation,
                    profile.dpd_seconds,
                    profile.rekey_seconds,
                    profile.authentication,
                )
            }),
            Ok((
                false,
                false,
                30,
                3600,
                Ikev2Authentication::Eap {
                    username: "alice".to_owned(),
                    password: SecretReference("secret://ike-password/1".to_owned()),
                },
            ))
        );
    }

    #[test]
    fn ikev2_rejects_unknown_authority_bad_proposals_and_bad_booleans() {
        for profile in [
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=des-md5-modp768\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nMobike=yes",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nExec=x",
        ] {
            assert!(parse_ikev2_profile(profile, &mut RecordingImporter::default()).is_err());
        }
    }

    #[test]
    fn ikev2_rejects_auth_conflicts_missing_fields_duplicates_and_bad_timers() {
        for profile in [
            "[IKEv2]\nServer=s\nAuth=unknown\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nUsername=u\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nPassword=p\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=psk\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=eap\nPassword=p\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=eap\nUsername=u\nProposal=aes256gcm16-prfsha256-ecp256\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nServer=t\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=0",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nRekeySeconds=299",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=400\nRekeySeconds=400",
            "[IKEv2]\nServer=s\nAuth=psk\nPsk=k\nProposal=aes256gcm16-prfsha384-ecp384\nTrafficSelectors=10.0.0.0/8\nDpdSeconds=nope",
        ] {
            assert!(parse_ikev2_profile(profile, &mut RecordingImporter::default()).is_err());
        }
    }

    #[test]
    fn ikev2_rejects_structure_and_import_failures() {
        for profile in [
            "Server=s",
            "[IKEv2]\n[IKEv2]",
            "[Other]\nServer=s",
            "[IKEv2]\nbroken",
            "[IKEv2]\nServer=",
        ] {
            assert!(parse_ikev2_profile(profile, &mut RecordingImporter::default()).is_err());
        }
        let mut importer = RecordingImporter {
            kinds: Vec::new(),
            fail: true,
        };
        assert_eq!(
            parse_ikev2_profile(ikev2_psk_profile(), &mut importer),
            Err(ProfileError::SecretImportFailed)
        );
    }

    #[test]
    fn top_level_detection_dispatches_and_rejects_unknown_profiles() {
        assert_eq!(
            parse_vpn_profile("[Other]\nA=B", &mut RecordingImporter::default()),
            Err(ProfileError::UnsupportedProfile)
        );
        assert_eq!(
            parse_vpn_profile("", &mut RecordingImporter::default()),
            Err(ProfileError::UnsupportedProfile)
        );
        assert_eq!(
            parse_vpn_profile("# only comment", &mut RecordingImporter::default()),
            Err(ProfileError::UnsupportedProfile)
        );
        assert_eq!(
            parse_vpn_profile(
                &"x".repeat(MAX_PROFILE_BYTES + 1),
                &mut RecordingImporter::default()
            ),
            Err(ProfileError::ProfileTooLarge)
        );
    }

    #[test]
    fn wireguard_post_key_validation_fail_closed_edges_are_covered() {
        for (profile, expected) in [
            (
                format!(
                    "[Interface]\nAddress=10.0.0.2/32\nListenPort=51820\nListenPort=51821\nPrivateKey={VALID_WIREGUARD_KEY}\n"
                ),
                ProfileError::DuplicateField,
            ),
            (
                format!(
                    "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs=10.0.0.0/8\nPersistentKeepalive=10\nPersistentKeepalive=20\n"
                ),
                ProfileError::DuplicateField,
            ),
            (
                format!(
                    "[Interface]\nAddress=10.0.0.2/32\nListenPort=not-a-number\nPrivateKey={VALID_WIREGUARD_KEY}\n"
                ),
                ProfileError::InvalidValue,
            ),
            (
                format!(
                    "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPersistentKeepalive=not-a-number\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs=10.0.0.0/8\n"
                ),
                ProfileError::InvalidValue,
            ),
            (
                format!(
                    "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nPublicKey={VALID_WIREGUARD_KEY}\nAllowedIPs=10.0.0.0/8\nUnknown=x\n"
                ),
                ProfileError::UnsupportedAuthority,
            ),
            (
                format!(
                    "[Interface]\nAddress=10.0.0.2/32\nPrivateKey={VALID_WIREGUARD_KEY}\n[Peer]\nAllowedIPs=10.0.0.0/8\n"
                ),
                ProfileError::MissingField,
            ),
            (
                format!("[Interface]\nPrivateKey={VALID_WIREGUARD_KEY}\n"),
                ProfileError::MissingField,
            ),
        ] {
            let mut importer = RecordingImporter::default();
            assert_eq!(
                import_wireguard_profile(&profile, &mut importer),
                Err(expected)
            );
            assert!(importer.kinds.is_empty());
        }
    }
}

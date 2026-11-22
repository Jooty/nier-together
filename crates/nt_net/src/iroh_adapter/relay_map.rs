use iroh::{RelayConfig, RelayMap, RelayMode, RelayUrl};

pub const TYPHON_RELAY_URL: &str = "https://nt-relay.ctrldevil.com";

pub fn relay_map_with_typhon() -> RelayMap {
    let map = RelayMode::Default.relay_map();
    if let Ok(url) = TYPHON_RELAY_URL.parse::<RelayUrl>() {
        map.insert(url.clone(), RelayConfig::new(url, None).into());
    }
    map
}

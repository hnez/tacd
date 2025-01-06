//! This code was generated by `zbus-xmlgen` `4.1.0` from DBus introspection data.
//!
//! By manually running
//!
//! zbus-xmlgen system org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/IP4Config/<ID>
//!
//! For all <ID>s on the LXA TAC and manually combining the results.

use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.NetworkManager.IP4Config",
    default_service = "org.freedesktop.NetworkManager"
)]
trait IP4Config {
    /// AddressData property
    #[zbus(property)]
    fn address_data(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// Addresses property
    #[zbus(property)]
    fn addresses(&self) -> zbus::Result<Vec<Vec<u32>>>;

    /// DnsOptions property
    #[zbus(property)]
    fn dns_options(&self) -> zbus::Result<Vec<String>>;

    /// DnsPriority property
    #[zbus(property)]
    fn dns_priority(&self) -> zbus::Result<i32>;

    /// Domains property
    #[zbus(property)]
    fn domains(&self) -> zbus::Result<Vec<String>>;

    /// Gateway property
    #[zbus(property)]
    fn gateway(&self) -> zbus::Result<String>;

    /// NameserverData property
    #[zbus(property)]
    fn nameserver_data(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// Nameservers property
    #[zbus(property)]
    fn nameservers(&self) -> zbus::Result<Vec<u32>>;

    /// RouteData property
    #[zbus(property)]
    fn route_data(
        &self,
    ) -> zbus::Result<Vec<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>>;

    /// Routes property
    #[zbus(property)]
    fn routes(&self) -> zbus::Result<Vec<Vec<u32>>>;

    /// Searches property
    #[zbus(property)]
    fn searches(&self) -> zbus::Result<Vec<String>>;

    /// WinsServerData property
    #[zbus(property)]
    fn wins_server_data(&self) -> zbus::Result<Vec<String>>;

    /// WinsServers property
    #[zbus(property)]
    fn wins_servers(&self) -> zbus::Result<Vec<u32>>;
}

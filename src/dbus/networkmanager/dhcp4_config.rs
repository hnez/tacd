//! This code was generated by `zbus-xmlgen` `3.1.1` from DBus introspection data.
//!
//! By manually running
//!
//! zbus-xmlgen --system org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/DHCP4Config/<ID>
//!
//! For all <ID>s on the LXA TAC and manually combining the results.

use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.freedesktop.NetworkManager.DHCP4Config",
    default_service = "org.freedesktop.NetworkManager"
)]
trait DHCP4Config {
    /// Options property
    #[dbus_proxy(property)]
    fn options(
        &self,
    ) -> zbus::Result<std::collections::HashMap<String, zbus::zvariant::OwnedValue>>;
}

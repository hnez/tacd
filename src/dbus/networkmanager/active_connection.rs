//! This code was generated by `zbus-xmlgen` `4.1.0` from DBus introspection data.
//!
//! By manually running
//!
//! zbus-xmlgen system org.freedesktop.NetworkManager /org/freedesktop/NetworkManager/ActiveConnection/<ID>
//!
//! For all <ID>s on the LXA TAC and manually combining the results.

use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.NetworkManager.Connection.Active",
    default_service = "org.freedesktop.NetworkManager"
)]
trait Active {
    /// StateChanged signal
    #[zbus(signal)]
    fn state_changed(&self, state: u32, reason: u32) -> zbus::Result<()>;

    /// Connection property
    #[zbus(property)]
    fn connection(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Controller property
    #[zbus(property)]
    fn controller(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Default property
    #[zbus(property)]
    fn default(&self) -> zbus::Result<bool>;

    /// Default6 property
    #[zbus(property)]
    fn default6(&self) -> zbus::Result<bool>;

    /// Devices property
    #[zbus(property)]
    fn devices(&self) -> zbus::Result<Vec<zbus::zvariant::OwnedObjectPath>>;

    /// Dhcp4Config property
    #[zbus(property)]
    fn dhcp4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Dhcp6Config property
    #[zbus(property)]
    fn dhcp6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Id property
    #[zbus(property)]
    fn id(&self) -> zbus::Result<String>;

    /// Ip4Config property
    #[zbus(property)]
    fn ip4_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Ip6Config property
    #[zbus(property)]
    fn ip6_config(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// Master property
    #[zbus(property)]
    fn master(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// SpecificObject property
    #[zbus(property)]
    fn specific_object(&self) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;

    /// State property
    #[zbus(property)]
    fn state(&self) -> zbus::Result<u32>;

    /// StateFlags property
    #[zbus(property)]
    fn state_flags(&self) -> zbus::Result<u32>;

    /// Type property
    #[zbus(property)]
    fn type_(&self) -> zbus::Result<String>;

    /// Uuid property
    #[zbus(property)]
    fn uuid(&self) -> zbus::Result<String>;

    /// Vpn property
    #[zbus(property)]
    fn vpn(&self) -> zbus::Result<bool>;
}

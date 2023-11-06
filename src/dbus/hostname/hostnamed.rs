//! This code was generated by `zbus-xmlgen` `3.1.1` from DBus introspection data.
//!
//! By running `zbus-xmlgen --system org.freedesktop.hostname1 /org/freedesktop/hostname1`
//! on the LXA TAC.

use zbus::dbus_proxy;

#[dbus_proxy(
    interface = "org.freedesktop.hostname1",
    default_service = "org.freedesktop.hostname1",
    default_path = "/org/freedesktop/hostname1"
)]
trait Hostname {
    /// Describe method
    fn describe(&self) -> zbus::Result<String>;

    /// GetHardwareSerial method
    fn get_hardware_serial(&self) -> zbus::Result<String>;

    /// GetProductUUID method
    #[dbus_proxy(name = "GetProductUUID")]
    fn get_product_uuid(&self, interactive: bool) -> zbus::Result<Vec<u8>>;

    /// SetChassis method
    fn set_chassis(&self, chassis: &str, interactive: bool) -> zbus::Result<()>;

    /// SetDeployment method
    fn set_deployment(&self, deployment: &str, interactive: bool) -> zbus::Result<()>;

    /// SetHostname method
    fn set_hostname(&self, hostname: &str, interactive: bool) -> zbus::Result<()>;

    /// SetIconName method
    fn set_icon_name(&self, icon: &str, interactive: bool) -> zbus::Result<()>;

    /// SetLocation method
    fn set_location(&self, location: &str, interactive: bool) -> zbus::Result<()>;

    /// SetPrettyHostname method
    fn set_pretty_hostname(&self, hostname: &str, interactive: bool) -> zbus::Result<()>;

    /// SetStaticHostname method
    fn set_static_hostname(&self, hostname: &str, interactive: bool) -> zbus::Result<()>;

    /// Chassis property
    #[dbus_proxy(property)]
    fn chassis(&self) -> zbus::Result<String>;

    /// DefaultHostname property
    #[dbus_proxy(property)]
    fn default_hostname(&self) -> zbus::Result<String>;

    /// Deployment property
    #[dbus_proxy(property)]
    fn deployment(&self) -> zbus::Result<String>;

    /// FirmwareVersion property
    #[dbus_proxy(property)]
    fn firmware_version(&self) -> zbus::Result<String>;

    /// HardwareModel property
    #[dbus_proxy(property)]
    fn hardware_model(&self) -> zbus::Result<String>;

    /// HardwareVendor property
    #[dbus_proxy(property)]
    fn hardware_vendor(&self) -> zbus::Result<String>;

    /// HomeURL property
    #[dbus_proxy(property, name = "HomeURL")]
    fn home_url(&self) -> zbus::Result<String>;

    /// Hostname property
    #[dbus_proxy(property)]
    fn hostname(&self) -> zbus::Result<String>;

    /// HostnameSource property
    #[dbus_proxy(property)]
    fn hostname_source(&self) -> zbus::Result<String>;

    /// IconName property
    #[dbus_proxy(property)]
    fn icon_name(&self) -> zbus::Result<String>;

    /// KernelName property
    #[dbus_proxy(property)]
    fn kernel_name(&self) -> zbus::Result<String>;

    /// KernelRelease property
    #[dbus_proxy(property)]
    fn kernel_release(&self) -> zbus::Result<String>;

    /// KernelVersion property
    #[dbus_proxy(property)]
    fn kernel_version(&self) -> zbus::Result<String>;

    /// Location property
    #[dbus_proxy(property)]
    fn location(&self) -> zbus::Result<String>;

    /// OperatingSystemCPEName property
    #[dbus_proxy(property, name = "OperatingSystemCPEName")]
    fn operating_system_cpename(&self) -> zbus::Result<String>;

    /// OperatingSystemPrettyName property
    #[dbus_proxy(property)]
    fn operating_system_pretty_name(&self) -> zbus::Result<String>;

    /// PrettyHostname property
    #[dbus_proxy(property)]
    fn pretty_hostname(&self) -> zbus::Result<String>;

    /// StaticHostname property
    #[dbus_proxy(property)]
    fn static_hostname(&self) -> zbus::Result<String>;
}

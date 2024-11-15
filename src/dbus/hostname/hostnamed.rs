//! This code was generated by `zbus-xmlgen` `4.1.0` from DBus introspection data.
//!
//! By running `zbus-xmlgen system org.freedesktop.hostname1 /org/freedesktop/hostname1`
//! on the LXA TAC.

use zbus::proxy;

#[proxy(
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
    #[zbus(name = "GetProductUUID")]
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

    /// BootID property
    #[zbus(property, name = "BootID")]
    fn boot_id(&self) -> zbus::Result<Vec<u8>>;

    /// Chassis property
    #[zbus(property)]
    fn chassis(&self) -> zbus::Result<String>;

    /// DefaultHostname property
    #[zbus(property)]
    fn default_hostname(&self) -> zbus::Result<String>;

    /// Deployment property
    #[zbus(property)]
    fn deployment(&self) -> zbus::Result<String>;

    /// FirmwareDate property
    #[zbus(property)]
    fn firmware_date(&self) -> zbus::Result<u64>;

    /// FirmwareVendor property
    #[zbus(property)]
    fn firmware_vendor(&self) -> zbus::Result<String>;

    /// FirmwareVersion property
    #[zbus(property)]
    fn firmware_version(&self) -> zbus::Result<String>;

    /// HardwareModel property
    #[zbus(property)]
    fn hardware_model(&self) -> zbus::Result<String>;

    /// HardwareVendor property
    #[zbus(property)]
    fn hardware_vendor(&self) -> zbus::Result<String>;

    /// HomeURL property
    #[zbus(property, name = "HomeURL")]
    fn home_url(&self) -> zbus::Result<String>;

    /// Hostname property
    #[zbus(property)]
    fn hostname(&self) -> zbus::Result<String>;

    /// HostnameSource property
    #[zbus(property)]
    fn hostname_source(&self) -> zbus::Result<String>;

    /// IconName property
    #[zbus(property)]
    fn icon_name(&self) -> zbus::Result<String>;

    /// KernelName property
    #[zbus(property)]
    fn kernel_name(&self) -> zbus::Result<String>;

    /// KernelRelease property
    #[zbus(property)]
    fn kernel_release(&self) -> zbus::Result<String>;

    /// KernelVersion property
    #[zbus(property)]
    fn kernel_version(&self) -> zbus::Result<String>;

    /// Location property
    #[zbus(property)]
    fn location(&self) -> zbus::Result<String>;

    /// MachineID property
    #[zbus(property, name = "MachineID")]
    fn machine_id(&self) -> zbus::Result<Vec<u8>>;

    /// OperatingSystemCPEName property
    #[zbus(property, name = "OperatingSystemCPEName")]
    fn operating_system_cpename(&self) -> zbus::Result<String>;

    /// OperatingSystemPrettyName property
    #[zbus(property)]
    fn operating_system_pretty_name(&self) -> zbus::Result<String>;

    /// OperatingSystemSupportEnd property
    #[zbus(property)]
    fn operating_system_support_end(&self) -> zbus::Result<u64>;

    /// PrettyHostname property
    #[zbus(property)]
    fn pretty_hostname(&self) -> zbus::Result<String>;

    /// StaticHostname property
    #[zbus(property)]
    fn static_hostname(&self) -> zbus::Result<String>;
}

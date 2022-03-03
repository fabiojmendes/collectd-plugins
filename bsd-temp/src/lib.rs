use collectd_plugin as cld;
use collectd_plugin::{
    bindings::{
        oconfig_item_t, plugin_register_complex_config, plugin_register_init,
        plugin_register_shutdown,
    },
    internal, ConfigItem, LogLevel, Plugin, PluginCapabilities, PluginManager, PluginRegistration,
    Value, ValueListBuilder,
};
use std::{error, ffi::CString};
use sysctl::{Ctl, CtlValue, Sysctl};

struct BSDTemp {
    ctls: Vec<(String, Ctl)>,
}

impl PluginManager for BSDTemp {
    fn name() -> &'static str {
        "sensors"
    }

    fn plugins(
        config: Option<&[ConfigItem<'_>]>,
    ) -> Result<PluginRegistration, Box<dyn error::Error>> {
        let mut ctls = Vec::new();
        for item in config.unwrap() {
            let value = item.values.get(0).ok_or("No name found in config")?;
            let name = match value {
                cld::ConfigValue::String(str) => str,
                _ => return Err("Error parsing name")?,
            };
            let value = item.values.get(1).ok_or("No label found in config")?;
            let label = match value {
                cld::ConfigValue::String(str) => str,
                _ => return Err("Error parsing label")?,
            };
            let line = format!("bsd_temp config: {:?} {:?}", name, label);
            cld::collectd_log(LogLevel::Info, &line);
            ctls.push((String::from(*label), Ctl::new(name)?));
        }
        let plugin = Box::new(BSDTemp { ctls });
        Ok(PluginRegistration::Single(plugin))
    }
}

impl Plugin for BSDTemp {
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::READ
    }

    fn read_values(&self) -> Result<(), Box<dyn error::Error>> {
        for (label, ctl) in &self.ctls {
            let val_enum = ctl.value()?;
            if let CtlValue::Temperature(val) = val_enum {
                let values = vec![Value::Gauge(val.celsius() as f64)];
                ValueListBuilder::new(Self::name(), "temperature")
                    .plugin_instance(&label[..])
                    .values(&values)
                    .submit()?;
            }
        }
        Ok(())
    }
}

// cld::collectd_plugin!(BSDTemp);

// Let's us know if we've seen our config section before
static CONFIG_SEEN: ::std::sync::atomic::AtomicBool = ::std::sync::atomic::AtomicBool::new(false);

#[no_mangle]
pub extern "C" fn module_register() {
    cld::collectd_log(LogLevel::Info, "register BSDTemp plugin");
    internal::register_panic_handler();
    let name = CString::new("bsd_temp").unwrap();
    unsafe {
        plugin_register_complex_config(name.as_ptr(), Some(collectd_plugin_complex_config));
        plugin_register_init(name.as_ptr(), Some(collectd_plugin_init));
        plugin_register_shutdown(name.as_ptr(), Some(collectd_plugin_shutdown));
    }
}

extern "C" fn collectd_plugin_init() -> ::std::os::raw::c_int {
    internal::plugin_init::<BSDTemp>(&CONFIG_SEEN)
}

extern "C" fn collectd_plugin_shutdown() -> ::std::os::raw::c_int {
    internal::plugin_shutdown::<BSDTemp>()
}

unsafe extern "C" fn collectd_plugin_complex_config(
    config: *mut oconfig_item_t,
) -> ::std::os::raw::c_int {
    internal::plugin_complex_config::<BSDTemp>(&CONFIG_SEEN, config)
}

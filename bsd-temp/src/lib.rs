use collectd_plugin::{
    collectd_plugin, ConfigItem, Plugin, PluginCapabilities, PluginManager, PluginRegistration,
    Value, ValueListBuilder,
};
use std::error;
use sysctl::{Ctl, CtlValue, Sysctl};

struct BSDTemp {
    core: Ctl,
    ccd: Ctl,
}

impl PluginManager for BSDTemp {
    fn name() -> &'static str {
        "sensors"
    }

    fn plugins(
        _config: Option<&[ConfigItem<'_>]>,
    ) -> Result<PluginRegistration, Box<dyn error::Error>> {
        let core = Ctl::new("dev.amdtemp.0.core0.sensor0")?;
        let ccd = Ctl::new("dev.amdtemp.0.ccd0")?;
        let plugin = Box::new(BSDTemp { core, ccd });
        Ok(PluginRegistration::Single(plugin))
    }
}

impl Plugin for BSDTemp {
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::READ
    }

    fn read_values(&self) -> Result<(), Box<dyn error::Error>> {
        let val_enum = self.core.value()?;
        if let CtlValue::Temperature(val) = val_enum {
            let values = vec![Value::Gauge(val.celsius() as f64)];
            ValueListBuilder::new(Self::name(), "temperature")
                .plugin_instance("core")
                .values(&values)
                .submit()?;
        }
        let val_enum = self.ccd.value()?;
        if let CtlValue::Temperature(val) = val_enum {
            let values = vec![Value::Gauge(val.celsius() as f64)];
            ValueListBuilder::new(Self::name(), "temperature")
                .plugin_instance("ccd")
                .values(&values)
                .submit()?;
        }
        Ok(())
    }
}

collectd_plugin!(BSDTemp);

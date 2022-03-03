use collectd_plugin as cld;
use collectd_plugin::{
    ConfigItem, ConfigValue, LogLevel, Plugin, PluginCapabilities, PluginManager,
    PluginRegistration, Value, ValueListBuilder,
};
use std::error;
use sysctl::{Ctl, CtlValue, Sysctl};

#[derive(Debug)]
struct BSDTemp {
    ctls: Vec<(String, Ctl)>,
}

fn parse_config_item<'a>(item: &'a ConfigItem) -> Option<(&'a str, &'a str)> {
    let &name = match item.values.get(0)? {
        ConfigValue::String(str) => str,
        _ => return None,
    };
    let &label = match item.values.get(1)? {
        ConfigValue::String(str) => str,
        _ => return None,
    };
    Some((name, label))
}

impl PluginManager for BSDTemp {
    fn name() -> &'static str {
        "bsd_temp"
    }

    fn plugins(
        config: Option<&[ConfigItem<'_>]>,
    ) -> Result<PluginRegistration, Box<dyn error::Error>> {
        let mut ctls = Vec::new();
        for item in config.ok_or("no config found")? {
            let (name, label) =
                parse_config_item(item).ok_or(format!("error parsing config item: {:?}", item))?;
            let line = format!("bsd_temp config: {:?} {:?}", name, label);
            cld::collectd_log(LogLevel::Debug, &line);
            ctls.push((String::from(label), Ctl::new(name)?));
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

cld::collectd_plugin!(BSDTemp);

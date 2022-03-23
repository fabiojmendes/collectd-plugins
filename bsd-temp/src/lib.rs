use collectd_plugin as cld;
use collectd_plugin::{
    ConfigItem, ConfigValue, LogLevel, Plugin, PluginCapabilities, PluginManager,
    PluginRegistration, Value, ValueListBuilder,
};
use std::error;
use sysctl::{Ctl, CtlValue, Sysctl};

/// Here is what our collectd config can look like:
///
/// ```
/// LoadPlugin bsd_temp
/// <Plugin bsd_temp>
///   <Ctl "sysctl.node.name" "optionalLabel">
///   <Ctl "sysctl.node.name">
/// </Plugin>
/// ```
#[derive(Debug)]
struct BSDTemp {
    ctls: Vec<(String, Ctl)>,
}

fn parse_config(item: &ConfigItem) -> Result<(String, Ctl), Box<dyn error::Error>> {
    let name = match item.values.first() {
        Some(ConfigValue::String(str)) => *str,
        _ => return Err(format!("syntax error {:?}", item).into()),
    };
    let label = match item.values.get(1) {
        Some(ConfigValue::String(str)) => str,
        _ => name,
    };
    cld::collectd_log(
        LogLevel::Debug,
        &format!("bsd_temp config: {:?} {:?}", name, label),
    );
    Ok((String::from(label), Ctl::new(name)?))
}

impl PluginManager for BSDTemp {
    fn name() -> &'static str {
        "bsd_temp"
    }

    fn plugins(
        config: Option<&[ConfigItem<'_>]>,
    ) -> Result<PluginRegistration, Box<dyn error::Error>> {
        let ctls: Vec<_> = config
            .ok_or("no config provided")?
            .iter()
            .map(parse_config)
            .collect::<Result<_, _>>()?;
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
            if let CtlValue::Temperature(val) = ctl.value()? {
                let values = [Value::Gauge(val.celsius() as f64)];
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

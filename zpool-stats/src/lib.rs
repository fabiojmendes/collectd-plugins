use libzetta::zpool::{ZpoolEngine, ZpoolOpen3};
use std::error;

use collectd_plugin::{
    collectd_plugin, ConfigItem, Plugin, PluginCapabilities, PluginManager,
    PluginManagerCapabilities, PluginRegistration, Value, ValueListBuilder,
};

struct ZpoolStats {
    engine: ZpoolOpen3,
}

impl PluginManager for ZpoolStats {
    fn name() -> &'static str {
        "zfs_pool"
    }

    fn capabilities() -> PluginManagerCapabilities {
        PluginManagerCapabilities::INIT
    }

    fn initialize() -> Result<(), Box<dyn error::Error>> {
        // Collectd bleeds SIGCHLD into the plugins
        // https://www.mail-archive.com/collectd@verplant.org/msg03931.html
        unsafe {
            libc::signal(libc::SIGCHLD, libc::SIG_DFL);
        }
        Ok(())
    }

    fn plugins(
        _config: Option<&[ConfigItem<'_>]>,
    ) -> Result<PluginRegistration, Box<dyn error::Error>> {
        let engine = ZpoolOpen3::default();
        let plugin = Box::new(ZpoolStats { engine });
        Ok(PluginRegistration::Single(plugin))
    }
}

impl Plugin for ZpoolStats {
    fn capabilities(&self) -> PluginCapabilities {
        PluginCapabilities::READ
    }

    fn read_values(&self) -> Result<(), Box<dyn error::Error>> {
        for pool in &self.engine.all()? {
            let name = pool.name();
            let health_label = format!("{:?}", pool.health());
            let health_code = pool.health().clone() as u8;
            let health = [Value::Gauge(health_code as f64)];
            ValueListBuilder::new(Self::name(), "health")
                .plugin_instance(&name[..])
                .type_instance(&health_label[..])
                .values(&health)
                .submit()?;
            if let Ok(props) = self.engine.read_properties(pool.name()) {
                let alloc = [Value::Gauge(*props.alloc() as f64)];
                ValueListBuilder::new(Self::name(), "bytes")
                    .plugin_instance(&name[..])
                    .type_instance("allocated")
                    .values(&alloc)
                    .submit()?;
                let free = [Value::Gauge(*props.free() as f64)];
                ValueListBuilder::new(Self::name(), "bytes")
                    .plugin_instance(&name[..])
                    .type_instance("free")
                    .values(&free)
                    .submit()?;
                let size = [Value::Gauge(*props.size() as f64)];
                ValueListBuilder::new(Self::name(), "bytes")
                    .plugin_instance(&name[..])
                    .type_instance("size")
                    .values(&size)
                    .submit()?;
                let fragmentation = [Value::Gauge(*props.fragmentation() as f64)];
                ValueListBuilder::new(Self::name(), "percent")
                    .plugin_instance(&name[..])
                    .type_instance("fragmentation")
                    .values(&fragmentation)
                    .submit()?;
                let capacity = [Value::Gauge(*props.capacity() as f64)];
                ValueListBuilder::new(Self::name(), "percent")
                    .plugin_instance(&name[..])
                    .type_instance("capacity")
                    .values(&capacity)
                    .submit()?;
            }
        }

        Ok(())
    }
}

collectd_plugin!(ZpoolStats);

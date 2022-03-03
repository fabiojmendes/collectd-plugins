TARGET = target/debug
LIBS = $(TARGET)/libbsd_temp.so $(TARGET)/libzpool_stats.so
PLUGINS = plugins/bsd_temp.so plugins/zpool_stats.so

all: run

build:
	cargo build

$(LIBS): build

plugins/zpool_stats.so: target/debug/libzpool_stats.so
	cp ${.ALLSRC} ${.TARGET}

plugins/bsd_temp.so: target/debug/libbsd_temp.so
	cp ${.ALLSRC} ${.TARGET}

run: $(PLUGINS)
	collectd -f -C debug.conf


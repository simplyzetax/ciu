ESP_TARGET := xtensa-esp32-espidf
FIRMWARES := bike goggles helmet
ESPFLASH ?= espflash
PORT ?=
MONITOR_BAUD ?= 115200
ESPFLASH_PORT = $(if $(PORT),--port $(PORT),)

deploy-goggles monitor-goggles deploy-monitor-goggles: PORT = /dev/cu.usbserial-10
deploy-helmet monitor-helmet deploy-monitor-helmet: PORT = /dev/cu.usbserial-210

.PHONY: check check-core check-esp32 check-bike check-goggles check-helmet \
	$(addprefix build-,$(FIRMWARES)) \
	$(addprefix deploy-,$(FIRMWARES)) \
	$(addprefix monitor-,$(FIRMWARES)) \
	$(addprefix deploy-monitor-,$(FIRMWARES))

# Check the entire workspace for ESP32 (also the default for `make`).
check:
	cargo check-esp

# Core can be checked natively on the host.
check-core:
	cargo check -p ciu-core

check-esp32 check-bike check-goggles check-helmet: check-%:
	cargo check -p ciu-$* --target $(ESP_TARGET)

$(addprefix build-,$(FIRMWARES)): build-%:
	cargo build -p ciu-$* --target $(ESP_TARGET)

$(addprefix deploy-,$(FIRMWARES)): deploy-%: build-%
	$(ESPFLASH) flash $(ESPFLASH_PORT) target/$(ESP_TARGET)/debug/ciu-$*

$(addprefix monitor-,$(FIRMWARES)): monitor-%: build-%
	$(ESPFLASH) monitor $(ESPFLASH_PORT) --monitor-baud $(MONITOR_BAUD) \
		--elf target/$(ESP_TARGET)/debug/ciu-$*

$(addprefix deploy-monitor-,$(FIRMWARES)): deploy-monitor-%: build-%
	$(ESPFLASH) flash $(ESPFLASH_PORT) --monitor --monitor-baud $(MONITOR_BAUD) \
		target/$(ESP_TARGET)/debug/ciu-$*

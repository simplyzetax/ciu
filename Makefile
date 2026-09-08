.PHONY: check check-core check-esp32 check-bike check-goggles check-helmet

# Check the entire workspace for ESP32 (also the default for `make`).
check:
	cargo check-esp

# Core can be checked natively on the host.
check-core:
	cargo check -p ciu-core

check-esp32 check-bike check-goggles check-helmet: check-%:
	cargo check -p ciu-$* --target xtensa-esp32-espidf

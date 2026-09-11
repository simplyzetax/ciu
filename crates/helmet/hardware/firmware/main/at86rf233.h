#pragma once

#include <stdint.h>
#include "esp_err.h"

esp_err_t at86rf233_init(void);
esp_err_t at86rf233_reset(void);
esp_err_t at86rf233_read_register(uint8_t address, uint8_t *value);

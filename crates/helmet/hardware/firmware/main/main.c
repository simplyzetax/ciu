#include "at86rf233.h"

#include "esp_check.h"
#include "esp_log.h"

#define RG_VERSION_NUM 0x1d
#define RG_PART_NUM 0x1c

static const char *TAG = "bringup";

void app_main(void)
{
    uint8_t part = 0;
    uint8_t version = 0;

    ESP_ERROR_CHECK(at86rf233_init());
    ESP_ERROR_CHECK(at86rf233_reset());
    ESP_ERROR_CHECK(at86rf233_read_register(RG_PART_NUM, &part));
    ESP_ERROR_CHECK(at86rf233_read_register(RG_VERSION_NUM, &version));

    if (part != 0x0b || (version != 0x01 && version != 0x02)) {
        ESP_LOGE(TAG, "unexpected identity: PART_NUM=0x%02x VERSION_NUM=0x%02x", part, version);
        ESP_ERROR_CHECK(ESP_ERR_INVALID_RESPONSE);
    }
    ESP_LOGI(TAG, "AT86RF233 detected: PART_NUM=0x%02x VERSION_NUM=0x%02x", part, version);
}

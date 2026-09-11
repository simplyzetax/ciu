#include "at86rf233.h"
#include "pins.h"

#include "driver/gpio.h"
#include "driver/spi_master.h"
#include "esp_check.h"
#include "esp_rom_sys.h"

static spi_device_handle_t radio;

esp_err_t at86rf233_init(void)
{
    const gpio_config_t outputs = {
        .pin_bit_mask = (1ULL << PIN_RESET) | (1ULL << PIN_SLP_TR),
        .mode = GPIO_MODE_OUTPUT,
    };
    ESP_RETURN_ON_ERROR(gpio_config(&outputs), "at86rf233", "gpio_config");
    ESP_RETURN_ON_ERROR(gpio_set_level(PIN_SLP_TR, 0), "at86rf233", "SLP_TR low");

    const spi_bus_config_t bus = {
        .mosi_io_num = PIN_MOSI,
        .miso_io_num = PIN_MISO,
        .sclk_io_num = PIN_SCLK,
        .quadwp_io_num = -1,
        .quadhd_io_num = -1,
    };
    ESP_RETURN_ON_ERROR(spi_bus_initialize(SPI2_HOST, &bus, SPI_DMA_DISABLED), "at86rf233", "SPI bus");
    const spi_device_interface_config_t device = {
        .clock_speed_hz = 1000000,
        .mode = 0,
        .spics_io_num = PIN_SEL,
        .queue_size = 1,
    };
    return spi_bus_add_device(SPI2_HOST, &device, &radio);
}

esp_err_t at86rf233_reset(void)
{
    ESP_RETURN_ON_ERROR(gpio_set_level(PIN_RESET, 0), "at86rf233", "RESET low");
    esp_rom_delay_us(10);
    ESP_RETURN_ON_ERROR(gpio_set_level(PIN_RESET, 1), "at86rf233", "RESET high");
    esp_rom_delay_us(1000);
    return ESP_OK;
}

esp_err_t at86rf233_read_register(uint8_t address, uint8_t *value)
{
    if (!radio || !value || address > 0x3f) return ESP_ERR_INVALID_ARG;
    spi_transaction_t transaction = {
        .flags = SPI_TRANS_USE_TXDATA | SPI_TRANS_USE_RXDATA,
        .length = 16,
        .tx_data = { (uint8_t)(0x80 | address), 0 },
    };
    ESP_RETURN_ON_ERROR(spi_device_transmit(radio, &transaction), "at86rf233", "register read");
    *value = transaction.rx_data[1];
    return ESP_OK;
}

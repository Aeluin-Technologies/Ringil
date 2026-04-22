{
  services.udev.extraRules = ''
    # Jetson UART (généralement ttyTHS0, ttyTHS1, etc.)
    KERNEL=="ttyTHS[0-9]*", GROUP="dialout", MODE="0660"
    # Bus I2C pour des capteurs additionnels
    KERNEL=="i2c-[0-9]*", GROUP="i2c", MODE="0660"
    # Bus SPI
    KERNEL=="spidev*", GROUP="spi", MODE="0660"
  '';

  users.groups.i2c = {};
  users.groups.spi = {};
  users.groups.dialout = {};
}

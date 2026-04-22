{
  services.udev.extraRules = ''
    KERNEL=="ttyTHS[0-9]*", GROUP="dialout", MODE="0660"
    KERNEL=="i2c-[0-9]*", GROUP="i2c", MODE="0660"
    KERNEL=="spidev*", GROUP="spi", MODE="0660"
  '';

  users.groups.i2c = {};
  users.groups.spi = {};
  users.groups.dialout = {};
}

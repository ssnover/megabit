MEMORY {
    /*
        Basic usage for the nRF52840_xxAA
        FLASH : ORIGIN = 0x00000000, LENGTH = 1024K
        RAM : ORIGIN = 0x20000000, LENGTH = 256K
    */

    /*
        Parameters for usage with bossac (Arduino Nano 33 BLE)
    */
    FLASH : ORIGIN = 0x10000, LENGTH = (1024K - 0x10000)
    RAM : ORIGIN = 0x20000000, LENGTH = 256K

    /*
        If using nRF52840 with nrf-softdevice S140 7.3.0, use these
        FLASH : ORIGIN = 0x00027000, LENGTH = 868K
        RAM : ORIGIN = 0x20020000, LENGTH = 128K
    */
}
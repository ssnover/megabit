MEMORY {
    ROM (rx) : ORIGIN = 0x00000000, LENGTH = 16K

    FLASH_BOOT (rx) : ORIGIN = 0x10000000, LENGTH = 32K
    FLASH_APP  (rx) : ORIGIN = 0x10008000, LENGTH = 992K + 992K
    NVS        (rw) : ORIGIN = 0x101f8000, LENGTH = 32K

    /* Pick one of the two options for RAM layout     */

    /* OPTION A: Use all RAM banks as one big block   */
    /* Reasonable, unless you are doing something     */
    /* really particular with DMA or other concurrent */
    /* access that would benefit from striping        */
    RAM   : ORIGIN = 0x20000000, LENGTH = 264K

    /* OPTION B: Keep the unstriped sections separate */
    /* RAM: ORIGIN = 0x20000000, LENGTH = 256K        */
    /* SCRATCH_A: ORIGIN = 0x20040000, LENGTH = 4K    */
    /* SCRATCH_B: ORIGIN = 0x20041000, LENGTH = 4K    */
}
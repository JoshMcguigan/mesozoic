MEMORY
{
  FLASH : ORIGIN = 0x00026000, LENGTH = 512K - 152K
  /*
  Portions of RAM are reserved for:
  * Persisting debug information
  * NRF SoftDevice
  */
  RAM : ORIGIN = 0x20000000 + 16128, LENGTH = 64K - 1K - 16128
  PANIC_DUMP: ORIGIN = 0x2000FC00, LENGTH = 1K
}

_panic_dump_start = ORIGIN(PANIC_DUMP);
_panic_dump_end   = ORIGIN(PANIC_DUMP) + LENGTH(PANIC_DUMP);

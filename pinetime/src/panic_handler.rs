//! This module implements panic handling in a way that combines portions of
//! [panic-probe](https://github.com/knurling-rs/defmt/tree/main/firmware/panic-probe)
//! with large portions of [panic-persist](https://github.com/jamesmunns/panic-persist/tree/master).

use core::{
    cmp::min,
    fmt::{self, Write},
    mem::size_of,
    panic::PanicInfo,
    ptr::{self, addr_of_mut},
    slice,
    str::{from_utf8, from_utf8_unchecked},
};

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    cortex_m::interrupt::disable();

    defmt::error!("{}", defmt::Display2Format(info));

    let ram_write_result = writeln!(Ram { offset: 0 }, "{}", info);
    if ram_write_result.is_err() {
        defmt::error!("panic handler failed to persist panic info to RAM");
    }

    cortex_m::peripheral::SCB::sys_reset();
}

pub fn get_message() -> Option<&'static str> {
    let bytes = get_panic_message_bytes()?;

    match from_utf8(bytes) {
        Ok(stir) => Some(stir),
        // Safety: We just validated these bytes as UTF-8.
        Err(utf_err) => Some(unsafe { from_utf8_unchecked(&bytes[..utf_err.valid_up_to()]) }),
    }
}

struct Ram {
    offset: usize,
}

/// Internal Write implementation to output the formatted panic string into RAM
impl fmt::Write for Ram {
    fn write_str(&mut self, s: &str) -> Result<(), fmt::Error> {
        // Obtain panic region start and end from linker symbol _panic_dump_start and _panic_dump_end
        extern "C" {
            static mut _panic_dump_start: u8;
            static mut _panic_dump_end: u8;
        }

        // Get the data about the string that is being written now
        let data = s.as_bytes();
        let len = data.len();

        // Obtain info about the panic dump region
        let start_ptr = unsafe { addr_of_mut!(_panic_dump_start) as *mut u8 };
        let end_ptr = unsafe { addr_of_mut!(_panic_dump_end) as *mut u8 };
        let max_len = end_ptr as usize - start_ptr as usize;
        let max_len_str = max_len - size_of::<usize>() - size_of::<usize>();

        // If we have written the full length of the region, we can't write any
        // more. This could happen with multiple writes with this implementation
        if self.offset >= max_len_str {
            return Ok(());
        }

        // We should write the size of the string, or the amount of space
        // we have remaining, whichever is less
        let str_len = min(max_len_str - self.offset, len);

        unsafe {
            // Write the magic word for later detection
            start_ptr.cast::<usize>().write_unaligned(0x0FACADE0);

            // For now, skip writing the length...

            // Write the string to RAM
            ptr::copy(
                data.as_ptr() as *mut u8,
                start_ptr.offset(8).offset(self.offset as isize),
                str_len,
            );

            // Increment the offset so later writes will be appended
            self.offset += str_len;

            // ... and now write the current offset (or total size) to the size location
            start_ptr
                .offset(4)
                .cast::<usize>()
                .write_unaligned(self.offset);
        };

        Ok(())
    }
}

/// Get the panic message from the last boot, if any.
/// This method may possibly not return valid UTF-8 if the message
/// was truncated before the end of a full UTF-8 character. Care must
/// be taken before treating this as a proper &str.
///
/// If a message existed, this function will only return the value once
/// (subsequent calls will return None)
fn get_panic_message_bytes() -> Option<&'static [u8]> {
    // Obtain panic region start and end from linker symbol _panic_dump_start and _panic_dump_end
    extern "C" {
        static mut _panic_dump_start: u8;
        static mut _panic_dump_end: u8;
    }

    let start_ptr = unsafe { addr_of_mut!(_panic_dump_start) as *mut u8 };

    if 0x0FACADE0 != unsafe { ptr::read_unaligned(start_ptr.cast::<usize>()) } {
        return None;
    }

    // Clear the magic word to prevent this message from "sticking"
    // across multiple boots
    unsafe {
        start_ptr.cast::<usize>().write_unaligned(0x00000000);
    }

    // Obtain info about the panic dump region
    let end_ptr = unsafe { addr_of_mut!(_panic_dump_end) as *mut u8 };
    let max_len = end_ptr as usize - start_ptr as usize;
    let max_len_str = max_len - size_of::<usize>() - size_of::<usize>();

    let len = unsafe { ptr::read_unaligned(start_ptr.offset(4).cast::<usize>()) };

    if len > max_len_str {
        return None;
    }

    // TODO: This is prooooooooobably undefined behavior
    let byte_slice = unsafe { slice::from_raw_parts(start_ptr.offset(8), len) };

    Some(byte_slice)
}

use crate::uefi::types::{EfiHandle, EfiStatus, Result};
use core::fmt;
use core::marker::PhantomPinned;

#[repr(C)]
pub struct EfiSimpleTextOutputProtocol {
    reset: EfiHandle,
    output_string:
        extern "win64" fn(this: *const EfiSimpleTextOutputProtocol, str: *const u16) -> EfiStatus,
    test_string: EfiHandle,
    query_mode: EfiHandle,
    set_mode: EfiHandle,
    set_attribute: EfiHandle,
    clear_screen: extern "win64" fn(this: *const EfiSimpleTextOutputProtocol) -> EfiStatus,
    _pinned: PhantomPinned,
}

impl EfiSimpleTextOutputProtocol {
    pub fn clear_screen(&self) -> Result<()> {
        (self.clear_screen)(self).into_result()
    }
}

pub struct EfiSimpleTextOutputProtocolWriter<'a> {
    pub protocol: &'a EfiSimpleTextOutputProtocol,
    //
    _pinned: PhantomPinned,
}

impl<'a> EfiSimpleTextOutputProtocolWriter<'a> {
    pub fn new(protocol: &'a EfiSimpleTextOutputProtocol) -> Self {
        EfiSimpleTextOutputProtocolWriter {
            protocol,
            _pinned: PhantomPinned,
        }
    }
}

impl EfiSimpleTextOutputProtocolWriter<'_> {
    pub fn write_char(&mut self, c: u8) {
        let cbuf: [u16; 2] = [c.into(), 0];
        (self.protocol.output_string)(self.protocol, cbuf.as_ptr())
            .into_result()
            .unwrap();
    }
    pub fn write_str(&mut self, s: &str) {
        for c in s.bytes() {
            if c == b'\n' {
                self.write_char(b'\r');
            }
            self.write_char(c);
        }
    }
}

impl fmt::Write for EfiSimpleTextOutputProtocolWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s);
        Ok(())
    }
}

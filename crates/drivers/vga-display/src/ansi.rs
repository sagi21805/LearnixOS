//! Decodes the ANSI subset the console understands, per ECMA-48 section
//! 5.4.

/// Parameters a color sequence may hold. A longer sequence is dropped.
const MAX_SGR_PARAMS: usize = 8;

const ESCAPE: u8 = 0x1b;
const BACKSPACE: u8 = 0x08;
const DELETE: u8 = 0x7f;

/// What one byte of text asks the console to do.
pub enum Decoded<'a> {
    Print(u8),
    LineFeed,
    Backspace,
    /// The parameters of a Select Graphic Rendition sequence.
    Rendition(&'a [u16]),
}

/// Turns a stream of bytes into [`Decoded`] actions.
///
/// It keeps its state between calls, because `write_fmt` splits one print
/// into several writes, so a sequence can arrive in pieces.
///
/// ```
/// use vga_display::ansi::{AnsiDecoder, Decoded};
///
/// let mut decoder = AnsiDecoder::default();
///
/// for &byte in b"\x1b[3" {
///     assert!(decoder.advance(byte).is_none());
/// }
/// for &byte in b"1;" {
///     assert!(decoder.advance(byte).is_none());
/// }
/// let Some(Decoded::Rendition(&[31, 0])) = decoder.advance(b'm') else {
///     panic!("an empty parameter means 0");
/// };
///
/// assert!(matches!(decoder.advance(b'a'), Some(Decoded::Print(b'a'))));
/// assert!(matches!(decoder.advance(0xc3), Some(Decoded::Print(b'?'))));
/// assert!(decoder.advance(0xa9).is_none());
/// ```
pub struct AnsiDecoder {
    state: DecoderState,
    params: [u16; MAX_SGR_PARAMS],
    param_count: usize,
}

/// Where in a control sequence the decoder is.
#[derive(Clone, Copy)]
enum DecoderState {
    Ground,
    Escape,
    ControlSequence,
    /// A sequence with a form the console ignores, skipped to its end.
    UnsupportedSequence,
}

const impl Default for AnsiDecoder {
    fn default() -> Self {
        Self {
            state: DecoderState::Ground,
            params: [0; MAX_SGR_PARAMS],
            param_count: 0,
        }
    }
}

impl AnsiDecoder {
    /// Takes the next byte, and returns the action it completes.
    pub fn advance(&mut self, byte: u8) -> Option<Decoded<'_>> {
        match self.state {
            DecoderState::Ground => self.advance_ground(byte),
            DecoderState::Escape => {
                self.advance_escape(byte);
                None
            }
            DecoderState::ControlSequence => {
                self.advance_control_sequence(byte)
            }
            DecoderState::UnsupportedSequence => {
                self.advance_unsupported_sequence(byte);
                None
            }
        }
    }

    /// Plain text. A non-ASCII character prints one `?`, from its leading
    /// byte, and its continuation bytes print nothing.
    fn advance_ground(&mut self, byte: u8) -> Option<Decoded<'_>> {
        match byte {
            ESCAPE => {
                self.state = DecoderState::Escape;
                None
            }
            b'\n' => Some(Decoded::LineFeed),
            BACKSPACE | DELETE => Some(Decoded::Backspace),
            0x20..=0x7e => Some(Decoded::Print(byte)),
            0xc0..=0xff => Some(Decoded::Print(b'?')),
            _ => None,
        }
    }

    /// The byte after `ESC`, where only `[` starts a control sequence.
    fn advance_escape(&mut self, byte: u8) {
        self.state = match byte {
            b'[' => {
                self.params[0] = 0;
                self.param_count = 1;
                DecoderState::ControlSequence
            }
            _ => DecoderState::Ground,
        };
    }

    /// `CSI P ... P I ... I F`, where only numeric parameters are read.
    fn advance_control_sequence(
        &mut self,
        byte: u8,
    ) -> Option<Decoded<'_>> {
        match byte {
            b'0'..=b'9' => self.push_digit(byte - b'0'),
            b';' => self.next_param(),
            0x20..=0x2f | b':' | 0x3c..=0x3f => {
                self.state = DecoderState::UnsupportedSequence;
            }
            b'm' => {
                self.state = DecoderState::Ground;
                return Some(Decoded::Rendition(
                    &self.params[..self.param_count],
                ));
            }
            _ => self.state = DecoderState::Ground,
        }
        None
    }

    /// Skips to the final byte, or to any byte a sequence can't hold.
    fn advance_unsupported_sequence(&mut self, byte: u8) {
        if !matches!(byte, 0x20..=0x3f) {
            self.state = DecoderState::Ground;
        }
    }

    fn push_digit(&mut self, digit: u8) {
        let param = &mut self.params[self.param_count - 1];
        *param = param.saturating_mul(10).saturating_add(digit.into());
    }

    fn next_param(&mut self) {
        if self.param_count == MAX_SGR_PARAMS {
            self.state = DecoderState::UnsupportedSequence;
            return;
        }
        self.params[self.param_count] = 0;
        self.param_count += 1;
    }
}

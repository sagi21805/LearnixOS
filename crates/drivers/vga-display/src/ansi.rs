//! Decodes the ANSI subset the console understands, per ECMA-48 section
//! 5.4.

use core::ascii::Char;

use extend::ext;

use crate::color_code::ColorCode;

/// What one byte of text asks the console to do.
pub enum Decoded {
    Print(Char),
    LineFeed,
    Backspace,
}

/// Turns a stream of bytes into [`Decoded`] actions and change
/// 'self.color' as the stream is parsed.
///
/// It keeps its state between calls, because `write_fmt` splits one print
/// into several writes, so a sequence can arrive in pieces.
pub struct AnsiDecoder {
    state: DecoderState,
    /// The number the current control sequence is reading.
    param: u8,
    color: ColorCode,
    /// `color` with the current sequence's parameters applied, which
    /// replaces it only if the sequence turns out to be a color sequence.
    pending_color: ColorCode,
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
            param: 0,
            color: ColorCode::default(),
            pending_color: ColorCode::default(),
        }
    }
}

impl AnsiDecoder {
    /// The color the next printed character should have.
    pub fn color(&self) -> ColorCode { self.color }

    /// Takes the next byte, and returns the action it completes.
    ///
    /// A non-ASCII byte reads as `?`.
    pub fn advance(&mut self, byte: u8) -> Option<Decoded> {
        let ascii = byte.as_ascii().unwrap_or(Char::QuestionMark);
        match self.state {
            DecoderState::Ground => return self.advance_ground(ascii),
            DecoderState::Escape => self.advance_escape(ascii),
            DecoderState::ControlSequence => {
                self.advance_control_sequence(ascii)
            }
            DecoderState::UnsupportedSequence => {
                self.advance_unsupported_sequence(ascii)
            }
        }
        None
    }

    /// Plain text.
    fn advance_ground(&mut self, ascii: Char) -> Option<Decoded> {
        match ascii {
            Char::Escape => {
                self.state = DecoderState::Escape;
                None
            }
            Char::LineFeed => Some(Decoded::LineFeed),
            Char::Backspace | Char::Delete => Some(Decoded::Backspace),
            _ if ascii.is_control() => None,
            _ => Some(Decoded::Print(ascii)),
        }
    }

    /// The byte after `ESC`, where only `[` starts a control sequence.
    fn advance_escape(&mut self, ascii: Char) {
        self.state = match ascii {
            Char::LeftSquareBracket => {
                self.param = 0;
                self.pending_color = self.color;
                DecoderState::ControlSequence
            }
            _ => DecoderState::Ground,
        };
    }

    /// `CSI P ... P I ... I F`, where only numeric parameters are read.
    fn advance_control_sequence(&mut self, ascii: Char) {
        match ascii {
            Char::SmallM => {
                self.pending_color.apply_sgr(self.param);
                self.color = self.pending_color;
                self.state = DecoderState::Ground;
            }
            Char::Semicolon => self.finish_param(),
            _ if ascii.is_digit() => {
                self.push_digit(ascii.to_u8() - Char::Digit0.to_u8());
            }
            _ if ascii.is_sequence_byte() => {
                self.state = DecoderState::UnsupportedSequence;
            }
            _ => self.state = DecoderState::Ground,
        }
    }

    /// Skips to the final byte, or to any byte a sequence can't hold.
    fn advance_unsupported_sequence(&mut self, ascii: Char) {
        if !ascii.is_sequence_byte() {
            self.state = DecoderState::Ground;
        }
    }

    fn push_digit(&mut self, digit: u8) {
        self.param = self.param.saturating_mul(10).saturating_add(digit);
    }

    /// Applies the parameter before a `;`. Parameters 38 and 48 drop the
    /// sequence, so the color number after them doesn't read as a color.
    fn finish_param(&mut self) {
        match self.param {
            38 | 48 => self.state = DecoderState::UnsupportedSequence,
            param => {
                self.pending_color.apply_sgr(param);
                self.param = 0;
            }
        }
    }
}

#[ext]
impl Char {
    /// Whether this is a parameter or intermediate byte of a control
    /// sequence, per ECMA-48 section 5.4.
    fn is_sequence_byte(self) -> bool {
        matches!(self.to_u8(), 0x20..=0x3f)
    }
}

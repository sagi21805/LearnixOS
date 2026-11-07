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
/// This decoder keeps its state between calls, because `write_fmt` can
/// split one print into several writes, so a sequence can arrive in
/// pieces.
pub struct AnsiDecoder {
    state: DecoderState,
    /// The number the current control sequence is reading.
    param: u8,
    /// The color that is currently used by the console.
    color: ColorCode,
    /// The color the current sequence builds.
    pending_color: ColorCode,
}

/// Where the decoder is relative to a control sequence.
#[derive(Clone, Copy)]
enum DecoderState {
    /// No sequence is decoded, regular text printing.
    Text,
    /// ESCAPE (\x1b) has encountered, starting sequence.
    SeqStart,
    /// [`Char::LeftSquareBracket`] has encountered, sequence continues.
    InSeq,
    /// In a sequence the console ignores.
    SkipSeq,
}

const impl Default for AnsiDecoder {
    fn default() -> Self {
        Self {
            state: DecoderState::Text,
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
            DecoderState::Text => return self.advance_text(ascii),
            DecoderState::SeqStart => self.advance_sequence_start(ascii),
            DecoderState::InSeq => self.advance_in_sequence(ascii),
            DecoderState::SkipSeq => self.advance_skipping_sequence(ascii),
        }
        None
    }

    /// Plain text.
    fn advance_text(&mut self, ascii: Char) -> Option<Decoded> {
        match ascii {
            Char::Escape => {
                self.state = DecoderState::SeqStart;
                None
            }
            Char::LineFeed => Some(Decoded::LineFeed),
            Char::Backspace | Char::Delete => Some(Decoded::Backspace),
            _ if ascii.is_control() => None,
            _ => Some(Decoded::Print(ascii)),
        }
    }

    /// The byte after [`Char::Escape`], where only
    /// [`Char::LeftSquareBracket`] starts a control sequence.
    fn advance_sequence_start(&mut self, ascii: Char) {
        self.state = match ascii {
            Char::LeftSquareBracket => {
                self.param = 0;
                self.pending_color = self.color;
                DecoderState::InSeq
            }
            _ => DecoderState::Text,
        };
    }

    /// Parse the sequence.
    fn advance_in_sequence(&mut self, ascii: Char) {
        match ascii.to_u8() {
            b'm' => self.finish_sequence(),
            b';' => self.finish_param(),
            digit @ b'0'..=b'9' => self.push_digit(digit - b'0'),
            _ if ascii.is_sequence_byte() => {
                self.state = DecoderState::SkipSeq
            }
            _ => self.state = DecoderState::Text,
        }
    }

    /// Skips to the final byte, or to any byte a sequence can't hold.
    fn advance_skipping_sequence(&mut self, ascii: Char) {
        if !ascii.is_sequence_byte() {
            self.state = DecoderState::Text;
        }
    }

    fn push_digit(&mut self, digit: u8) {
        self.param = self.param.saturating_mul(10).saturating_add(digit);
    }

    /// Applies the parameter before a `;`.
    ///
    /// Parameters 38 and 48 currently drop the sequence, because they pick
    /// from a different color scheme that is not parsed in this
    /// decoder.
    fn finish_param(&mut self) {
        match self.param {
            38 | 48 => self.state = DecoderState::SkipSeq,
            param => {
                self.pending_color.apply_sgr(param);
                self.param = 0;
            }
        }
    }

    fn finish_sequence(&mut self) {
        self.pending_color.apply_sgr(self.param);
        self.color = self.pending_color;
        self.state = DecoderState::Text;
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

mod decode;
mod encode;
mod tables;
pub use decode::*;
pub use encode::*;

const PAD_BYTE: u8 = b'=';
/// Standard character set with padding.
pub const STANDARD: Config = Config {
    char_set: CharacterSet::Standard,
    pad: true,
    decode_allow_trailing_bits: false,
};
/// Contains configuration parameters for base64 encoding
#[derive(Clone, Copy, Debug)]
pub struct Config {
    /// Character set to use
    char_set: CharacterSet,
    /// True to pad output with `=` characters
    pad: bool,
    /// True to ignore excess nonzero bits in the last few symbols, otherwise an error is returned.
    decode_allow_trailing_bits: bool,
}

impl Config {
    /// Create a new `Config`.
    pub const fn new(char_set: CharacterSet, pad: bool) -> Config {
        Config {
            char_set,
            pad,
            decode_allow_trailing_bits: false,
        }
    }

    /// Sets whether to pad output with `=` characters.
    pub const fn pad(self, pad: bool) -> Config {
        Config { pad, ..self }
    }

    /// Sets whether to emit errors for nonzero trailing bits.
    ///
    /// This is useful when implementing
    /// [forgiving-base64 decode](https://infra.spec.whatwg.org/#forgiving-base64-decode).
    pub const fn decode_allow_trailing_bits(self, allow: bool) -> Config {
        Config {
            decode_allow_trailing_bits: allow,
            ..self
        }
    }
}

/// Available encoding character sets
#[derive(Clone, Copy, Debug)]
pub enum CharacterSet {
    /// The standard character set (uses `+` and `/`).
    ///
    /// See [RFC 3548](https://tools.ietf.org/html/rfc3548#section-3).
    Standard,
    /// The URL safe character set (uses `-` and `_`).
    ///
    /// See [RFC 3548](https://tools.ietf.org/html/rfc3548#section-4).
    UrlSafe,
    /// The `crypt(3)` character set (uses `./0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz`).
    ///
    /// Not standardized, but folk wisdom on the net asserts that this alphabet is what crypt uses.
    Crypt,
    /// The bcrypt character set (uses `./ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789`).
    Bcrypt,
    /// The character set used in IMAP-modified UTF-7 (uses `+` and `,`).
    ///
    /// See [RFC 3501](https://tools.ietf.org/html/rfc3501#section-5.1.3)
    ImapMutf7,
    /// The character set used in BinHex 4.0 files.
    ///
    /// See [BinHex 4.0 Definition](http://files.stairways.com/other/binhex-40-specs-info.txt)
    BinHex,
}

impl CharacterSet {
    fn encode_table(self) -> &'static [u8; 64] {
        match self {
            CharacterSet::Standard => tables::STANDARD_ENCODE,
            CharacterSet::UrlSafe => tables::URL_SAFE_ENCODE,
            CharacterSet::Crypt => tables::CRYPT_ENCODE,
            CharacterSet::Bcrypt => tables::BCRYPT_ENCODE,
            CharacterSet::ImapMutf7 => tables::IMAP_MUTF7_ENCODE,
            CharacterSet::BinHex => tables::BINHEX_ENCODE,
        }
    }

    fn decode_table(self) -> &'static [u8; 256] {
        match self {
            CharacterSet::Standard => tables::STANDARD_DECODE,
            CharacterSet::UrlSafe => tables::URL_SAFE_DECODE,
            CharacterSet::Crypt => tables::CRYPT_DECODE,
            CharacterSet::Bcrypt => tables::BCRYPT_DECODE,
            CharacterSet::ImapMutf7 => tables::IMAP_MUTF7_DECODE,
            CharacterSet::BinHex => tables::BINHEX_DECODE,
        }
    }
}

mod chunked_encoder {
    use core::cmp;
    use super::{Config, add_padding, encode_to_slice};

    /// The output mechanism for ChunkedEncoder's encoded bytes.
    pub trait Sink {
        type Error;

        /// Handle a chunk of encoded base64 data (as UTF-8 bytes)
        fn write_encoded_bytes(&mut self, encoded: &[u8]) -> Result<(), Self::Error>;
    }

    const BUF_SIZE: usize = 1024;

    /// A base64 encoder that emits encoded bytes in chunks without heap allocation.
    pub struct ChunkedEncoder {
        config: Config,
        max_input_chunk_len: usize,
    }

    impl ChunkedEncoder {
        pub fn new(config: Config) -> ChunkedEncoder {
            ChunkedEncoder {
                config,
                max_input_chunk_len: max_input_length(BUF_SIZE, config),
            }
        }

        pub fn encode<S: Sink>(&self, bytes: &[u8], sink: &mut S) -> Result<(), S::Error> {
            let mut encode_buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
            let encode_table = self.config.char_set.encode_table();

            let mut input_index = 0;

            while input_index < bytes.len() {
                // either the full input chunk size, or it's the last iteration
                let input_chunk_len = cmp::min(self.max_input_chunk_len, bytes.len() - input_index);

                let chunk = &bytes[input_index..(input_index + input_chunk_len)];

                let mut b64_bytes_written = encode_to_slice(chunk, &mut encode_buf, encode_table);

                input_index += input_chunk_len;
                let more_input_left = input_index < bytes.len();

                if self.config.pad && !more_input_left {
                    // no more input, add padding if needed. Buffer will have room because
                    // max_input_length leaves room for it.
                    b64_bytes_written +=
                        add_padding(bytes.len(), &mut encode_buf[b64_bytes_written..]);
                }

                sink.write_encoded_bytes(&encode_buf[0..b64_bytes_written])?;
            }

            Ok(())
        }
    }

    /// Calculate the longest input that can be encoded for the given output buffer size.
    ///
    /// If the config requires padding, two bytes of buffer space will be set aside so that the last
    /// chunk of input can be encoded safely.
    ///
    /// The input length will always be a multiple of 3 so that no encoding state has to be carried over
    /// between chunks.
    fn max_input_length(encoded_buf_len: usize, config: Config) -> usize {
        let effective_buf_len = if config.pad {
            // make room for padding
            encoded_buf_len
                .checked_sub(2)
                .expect("Don't use a tiny buffer")
        } else {
            encoded_buf_len
        };

        // No padding, so just normal base64 expansion.
        (effective_buf_len / 4) * 3
    }

    // A really simple sink that just appends to a string
    #[cfg(any(feature = "alloc", feature = "std", test))]
    pub(crate) struct StringSink<'a> {
        string: &'a mut String,
    }

    #[cfg(any(feature = "alloc", feature = "std", test))]
    impl<'a> StringSink<'a> {
        pub(crate) fn new(s: &mut String) -> StringSink {
            StringSink { string: s }
        }
    }

    #[cfg(any(feature = "alloc", feature = "std", test))]
    impl<'a> Sink for StringSink<'a> {
        type Error = ();

        fn write_encoded_bytes(&mut self, s: &[u8]) -> Result<(), Self::Error> {
            self.string.push_str(std::str::from_utf8(s).unwrap());

            Ok(())
        }
    }
}

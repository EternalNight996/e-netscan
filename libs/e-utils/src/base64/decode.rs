use super::{Config, STANDARD};
use std::fmt;
use crate::base64::{tables, PAD_BYTE};

pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, DecodeError> {
    decode_config(input, STANDARD)
}
pub fn decode_config<T: AsRef<[u8]>>(input: T, config: Config) -> Result<Vec<u8>, DecodeError> {
    let mut buffer = Vec::<u8>::with_capacity(input.as_ref().len() * 4 / 3);
    decode_config_buf(input, config, &mut buffer).map(|_| buffer)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DecodeError {
    /// An invalid byte was found in the input. The offset and offending byte are provided.
    InvalidByte(usize, u8),
    /// The length of the input is invalid.
    /// A typical cause of this is stray trailing whitespace or other separator bytes.
    /// In the case where excess trailing bytes have produced an invalid length *and* the last byte
    /// is also an invalid base64 symbol (as would be the case for whitespace, etc), `InvalidByte`
    /// will be emitted instead of `InvalidLength` to make the issue easier to debug.
    InvalidLength,
    /// The last non-padding input symbol's encoded 6 bits have nonzero bits that will be discarded.
    /// This is indicative of corrupted or truncated Base64.
    /// Unlike InvalidByte, which reports symbols that aren't in the alphabet, this error is for
    /// symbols that are in the alphabet but represent nonsensical encodings.
    InvalidLastSymbol(usize, u8),
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DecodeError::InvalidByte(index, byte) => {
                write!(f, "Invalid byte {}, offset {}.", byte, index)
            }
            DecodeError::InvalidLength => write!(f, "Encoded text cannot have a 6-bit remainder."),
            DecodeError::InvalidLastSymbol(index, byte) => {
                write!(f, "Invalid last symbol {}, offset {}.", byte, index)
            }
        }
    }
}

pub fn decode_config_buf<T: AsRef<[u8]>>(
    input: T,
    config: Config,
    buffer: &mut Vec<u8>,
) -> Result<(), DecodeError> {
    let input_bytes = input.as_ref();

    let starting_output_len = buffer.len();

    let num_chunks = num_chunks(input_bytes);
    let decoded_len_estimate = num_chunks
        .checked_mul(DECODED_CHUNK_LEN)
        .and_then(|p| p.checked_add(starting_output_len))
        .expect("Overflow when calculating output buffer length");
    buffer.resize(decoded_len_estimate, 0);

    let bytes_written;
    {
        let buffer_slice = &mut buffer.as_mut_slice()[starting_output_len..];
        bytes_written = decode_helper(input_bytes, num_chunks, config, buffer_slice)?;
    }

    buffer.truncate(starting_output_len + bytes_written);

    Ok(())
}
// decode logic operates on chunks of 8 input bytes without padding
const INPUT_CHUNK_LEN: usize = 8;
const DECODED_CHUNK_LEN: usize = 6;
/// Return the number of input chunks (including a possibly partial final chunk) in the input
fn num_chunks(input: &[u8]) -> usize {
    input
        .len()
        .checked_add(INPUT_CHUNK_LEN - 1)
        .expect("Overflow when calculating number of chunks in input")
        / INPUT_CHUNK_LEN
}
// we read a u64 and write a u64, but a u64 of input only yields 6 bytes of output, so the last
// 2 bytes of any output u64 should not be counted as written to (but must be available in a
// slice).
const DECODED_CHUNK_SUFFIX: usize = 2;
// how many u64's of input to handle at a time
const CHUNKS_PER_FAST_LOOP_BLOCK: usize = 4;
const INPUT_BLOCK_LEN: usize = CHUNKS_PER_FAST_LOOP_BLOCK * INPUT_CHUNK_LEN;
// includes the trailing 2 bytes for the final u64 write
const DECODED_BLOCK_LEN: usize =
    CHUNKS_PER_FAST_LOOP_BLOCK * DECODED_CHUNK_LEN + DECODED_CHUNK_SUFFIX;

/// Helper to avoid duplicating num_chunks calculation, which is costly on short inputs.
/// Returns the number of bytes written, or an error.
// We're on the fragile edge of compiler heuristics here. If this is not inlined, slow. If this is
// inlined(always), a different slow. plain ol' inline makes the benchmarks happiest at the moment,
// but this is fragile and the best setting changes with only minor code modifications.
#[inline]
fn decode_helper(
    input: &[u8],
    num_chunks: usize,
    config: Config,
    output: &mut [u8],
) -> Result<usize, DecodeError> {
    let char_set = config.char_set;
    let decode_table = char_set.decode_table();

    let remainder_len = input.len() % INPUT_CHUNK_LEN;

    // Because the fast decode loop writes in groups of 8 bytes (unrolled to
    // CHUNKS_PER_FAST_LOOP_BLOCK times 8 bytes, where possible) and outputs 8 bytes at a time (of
    // which only 6 are valid data), we need to be sure that we stop using the fast decode loop
    // soon enough that there will always be 2 more bytes of valid data written after that loop.
    let trailing_bytes_to_skip = match remainder_len {
        // if input is a multiple of the chunk size, ignore the last chunk as it may have padding,
        // and the fast decode logic cannot handle padding
        0 => INPUT_CHUNK_LEN,
        // 1 and 5 trailing bytes are illegal: can't decode 6 bits of input into a byte
        1 | 5 => {
            // trailing whitespace is so common that it's worth it to check the last byte to
            // possibly return a better error message
            if let Some(b) = input.last() {
                if *b != PAD_BYTE && decode_table[*b as usize] == tables::INVALID_VALUE {
                    return Err(DecodeError::InvalidByte(input.len() - 1, *b));
                }
            }

            return Err(DecodeError::InvalidLength);
        }
        // This will decode to one output byte, which isn't enough to overwrite the 2 extra bytes
        // written by the fast decode loop. So, we have to ignore both these 2 bytes and the
        // previous chunk.
        2 => INPUT_CHUNK_LEN + 2,
        // If this is 3 unpadded chars, then it would actually decode to 2 bytes. However, if this
        // is an erroneous 2 chars + 1 pad char that would decode to 1 byte, then it should fail
        // with an error, not panic from going past the bounds of the output slice, so we let it
        // use stage 3 + 4.
        3 => INPUT_CHUNK_LEN + 3,
        // This can also decode to one output byte because it may be 2 input chars + 2 padding
        // chars, which would decode to 1 byte.
        4 => INPUT_CHUNK_LEN + 4,
        // Everything else is a legal decode len (given that we don't require padding), and will
        // decode to at least 2 bytes of output.
        _ => remainder_len,
    };

    // rounded up to include partial chunks
    let mut remaining_chunks = num_chunks;

    let mut input_index = 0;
    let mut output_index = 0;

    {
        let length_of_fast_decode_chunks = input.len().saturating_sub(trailing_bytes_to_skip);

        // Fast loop, stage 1
        // manual unroll to CHUNKS_PER_FAST_LOOP_BLOCK of u64s to amortize slice bounds checks
        if let Some(max_start_index) = length_of_fast_decode_chunks.checked_sub(INPUT_BLOCK_LEN) {
            while input_index <= max_start_index {
                let input_slice = &input[input_index..(input_index + INPUT_BLOCK_LEN)];
                let output_slice = &mut output[output_index..(output_index + DECODED_BLOCK_LEN)];

                decode_chunk(
                    &input_slice[0..],
                    input_index,
                    decode_table,
                    &mut output_slice[0..],
                )?;
                decode_chunk(
                    &input_slice[8..],
                    input_index + 8,
                    decode_table,
                    &mut output_slice[6..],
                )?;
                decode_chunk(
                    &input_slice[16..],
                    input_index + 16,
                    decode_table,
                    &mut output_slice[12..],
                )?;
                decode_chunk(
                    &input_slice[24..],
                    input_index + 24,
                    decode_table,
                    &mut output_slice[18..],
                )?;

                input_index += INPUT_BLOCK_LEN;
                output_index += DECODED_BLOCK_LEN - DECODED_CHUNK_SUFFIX;
                remaining_chunks -= CHUNKS_PER_FAST_LOOP_BLOCK;
            }
        }

        // Fast loop, stage 2 (aka still pretty fast loop)
        // 8 bytes at a time for whatever we didn't do in stage 1.
        if let Some(max_start_index) = length_of_fast_decode_chunks.checked_sub(INPUT_CHUNK_LEN) {
            while input_index < max_start_index {
                decode_chunk(
                    &input[input_index..(input_index + INPUT_CHUNK_LEN)],
                    input_index,
                    decode_table,
                    &mut output
                        [output_index..(output_index + DECODED_CHUNK_LEN + DECODED_CHUNK_SUFFIX)],
                )?;

                output_index += DECODED_CHUNK_LEN;
                input_index += INPUT_CHUNK_LEN;
                remaining_chunks -= 1;
            }
        }
    }

    // Stage 3
    // If input length was such that a chunk had to be deferred until after the fast loop
    // because decoding it would have produced 2 trailing bytes that wouldn't then be
    // overwritten, we decode that chunk here. This way is slower but doesn't write the 2
    // trailing bytes.
    // However, we still need to avoid the last chunk (partial or complete) because it could
    // have padding, so we always do 1 fewer to avoid the last chunk.
    for _ in 1..remaining_chunks {
        decode_chunk_precise(
            &input[input_index..],
            input_index,
            decode_table,
            &mut output[output_index..(output_index + DECODED_CHUNK_LEN)],
        )?;

        input_index += INPUT_CHUNK_LEN;
        output_index += DECODED_CHUNK_LEN;
    }

    // always have one more (possibly partial) block of 8 input
    debug_assert!(input.len() - input_index > 1 || input.is_empty());
    debug_assert!(input.len() - input_index <= 8);

    // Stage 4
    // Finally, decode any leftovers that aren't a complete input block of 8 bytes.
    // Use a u64 as a stack-resident 8 byte buffer.
    let mut leftover_bits: u64 = 0;
    let mut morsels_in_leftover = 0;
    let mut padding_bytes = 0;
    let mut first_padding_index: usize = 0;
    let mut last_symbol = 0_u8;
    let start_of_leftovers = input_index;
    for (i, b) in input[start_of_leftovers..].iter().enumerate() {
        // '=' padding
        if *b == PAD_BYTE {
            // There can be bad padding in a few ways:
            // 1 - Padding with non-padding characters after it
            // 2 - Padding after zero or one non-padding characters before it
            //     in the current quad.
            // 3 - More than two characters of padding. If 3 or 4 padding chars
            //     are in the same quad, that implies it will be caught by #2.
            //     If it spreads from one quad to another, it will be caught by
            //     #2 in the second quad.

            if i % 4 < 2 {
                // Check for case #2.
                let bad_padding_index = start_of_leftovers
                    + if padding_bytes > 0 {
                        // If we've already seen padding, report the first padding index.
                        // This is to be consistent with the faster logic above: it will report an
                        // error on the first padding character (since it doesn't expect to see
                        // anything but actual encoded data).
                        first_padding_index
                    } else {
                        // haven't seen padding before, just use where we are now
                        i
                    };
                return Err(DecodeError::InvalidByte(bad_padding_index, *b));
            }

            if padding_bytes == 0 {
                first_padding_index = i;
            }

            padding_bytes += 1;
            continue;
        }

        // Check for case #1.
        // To make '=' handling consistent with the main loop, don't allow
        // non-suffix '=' in trailing chunk either. Report error as first
        // erroneous padding.
        if padding_bytes > 0 {
            return Err(DecodeError::InvalidByte(
                start_of_leftovers + first_padding_index,
                PAD_BYTE,
            ));
        }
        last_symbol = *b;

        // can use up to 8 * 6 = 48 bits of the u64, if last chunk has no padding.
        // To minimize shifts, pack the leftovers from left to right.
        let shift = 64 - (morsels_in_leftover + 1) * 6;
        // tables are all 256 elements, lookup with a u8 index always succeeds
        let morsel = decode_table[*b as usize];
        if morsel == tables::INVALID_VALUE {
            return Err(DecodeError::InvalidByte(start_of_leftovers + i, *b));
        }

        leftover_bits |= (morsel as u64) << shift;
        morsels_in_leftover += 1;
    }

    let leftover_bits_ready_to_append = match morsels_in_leftover {
        0 => 0,
        2 => 8,
        3 => 16,
        4 => 24,
        6 => 32,
        7 => 40,
        8 => 48,
        _ => unreachable!(
            "Impossible: must only have 0 to 8 input bytes in last chunk, with no invalid lengths"
        ),
    };

    // if there are bits set outside the bits we care about, last symbol encodes trailing bits that
    // will not be included in the output
    let mask = !0 >> leftover_bits_ready_to_append;
    if !config.decode_allow_trailing_bits && (leftover_bits & mask) != 0 {
        // last morsel is at `morsels_in_leftover` - 1
        return Err(DecodeError::InvalidLastSymbol(
            start_of_leftovers + morsels_in_leftover - 1,
            last_symbol,
        ));
    }

    let mut leftover_bits_appended_to_buf = 0;
    while leftover_bits_appended_to_buf < leftover_bits_ready_to_append {
        // `as` simply truncates the higher bits, which is what we want here
        let selected_bits = (leftover_bits >> (56 - leftover_bits_appended_to_buf)) as u8;
        output[output_index] = selected_bits;
        output_index += 1;

        leftover_bits_appended_to_buf += 8;
    }

    Ok(output_index)
}


/// Decode 8 bytes of input into 6 bytes of output. 8 bytes of output will be written, but only the
/// first 6 of those contain meaningful data.
///
/// `input` is the bytes to decode, of which the first 8 bytes will be processed.
/// `index_at_start_of_input` is the offset in the overall input (used for reporting errors
/// accurately)
/// `decode_table` is the lookup table for the particular base64 alphabet.
/// `output` will have its first 8 bytes overwritten, of which only the first 6 are valid decoded
/// data.
// yes, really inline (worth 30-50% speedup)
#[inline(always)]
fn decode_chunk(
    input: &[u8],
    index_at_start_of_input: usize,
    decode_table: &[u8; 256],
    output: &mut [u8],
) -> Result<(), DecodeError> {
    let mut accum: u64;

    let morsel = decode_table[input[0] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(index_at_start_of_input, input[0]));
    }
    accum = (morsel as u64) << 58;

    let morsel = decode_table[input[1] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 1,
            input[1],
        ));
    }
    accum |= (morsel as u64) << 52;

    let morsel = decode_table[input[2] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 2,
            input[2],
        ));
    }
    accum |= (morsel as u64) << 46;

    let morsel = decode_table[input[3] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 3,
            input[3],
        ));
    }
    accum |= (morsel as u64) << 40;

    let morsel = decode_table[input[4] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 4,
            input[4],
        ));
    }
    accum |= (morsel as u64) << 34;

    let morsel = decode_table[input[5] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 5,
            input[5],
        ));
    }
    accum |= (morsel as u64) << 28;

    let morsel = decode_table[input[6] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 6,
            input[6],
        ));
    }
    accum |= (morsel as u64) << 22;

    let morsel = decode_table[input[7] as usize];
    if morsel == tables::INVALID_VALUE {
        return Err(DecodeError::InvalidByte(
            index_at_start_of_input + 7,
            input[7],
        ));
    }
    accum |= (morsel as u64) << 16;

    write_u64(output, accum);

    Ok(())
}

#[inline]
fn write_u64(output: &mut [u8], value: u64) {
    output[..8].copy_from_slice(&value.to_be_bytes());
}

/// Decode an 8-byte chunk, but only write the 6 bytes actually decoded instead of including 2
/// trailing garbage bytes.
#[inline]
fn decode_chunk_precise(
    input: &[u8],
    index_at_start_of_input: usize,
    decode_table: &[u8; 256],
    output: &mut [u8],
) -> Result<(), DecodeError> {
    let mut tmp_buf = [0_u8; 8];

    decode_chunk(
        input,
        index_at_start_of_input,
        decode_table,
        &mut tmp_buf[..],
    )?;

    output[0..6].copy_from_slice(&tmp_buf[0..6]);

    Ok(())
}

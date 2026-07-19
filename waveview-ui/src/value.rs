use waveview_model::vcd::Value;
use waveview_model::viewer::ValueFormat;

pub fn format(value: &[Value], format: ValueFormat) -> String {
    if let [bit] = value {
        return bit_character(*bit).to_string();
    }
    if value.iter().any(|bit| matches!(bit, Value::X | Value::Z)) {
        return match format {
            ValueFormat::Hexadecimal => format_hexadecimal(value),
            _ => format_binary(value),
        };
    }

    match format {
        ValueFormat::Binary => format_binary(value),
        ValueFormat::Hexadecimal => format_hexadecimal(value),
        ValueFormat::Unsigned => format_unsigned(value),
        ValueFormat::Signed => format_signed(value),
        ValueFormat::Ascii => format_ascii(value),
    }
}

fn format_binary(value: &[Value]) -> String {
    let mut text = String::with_capacity(value.len() + 2);
    text.push_str("0b");
    text.extend(value.iter().copied().map(bit_character));
    text
}

fn format_hexadecimal(value: &[Value]) -> String {
    let padding = (4 - value.len() % 4) % 4;
    let mut digits = String::with_capacity(value.len().div_ceil(4));
    for nibble in std::iter::repeat_n(Value::V0, padding)
        .chain(value.iter().copied())
        .collect::<Vec<_>>()
        .chunks_exact(4)
    {
        let digit = if nibble.contains(&Value::X) {
            'x'
        } else if nibble.contains(&Value::Z) {
            'z'
        } else {
            let number = nibble.iter().fold(0_u8, |number, bit| {
                (number << 1) | u8::from(*bit == Value::V1)
            });
            char::from_digit(u32::from(number), 16).expect("nibble is hexadecimal")
        };
        digits.push(digit);
    }
    format!("0x{digits}")
}

fn format_unsigned(value: &[Value]) -> String {
    decimal_from_bits(value.iter().copied())
}

fn format_signed(value: &[Value]) -> String {
    if value.first() != Some(&Value::V1) {
        return format_unsigned(value);
    }

    let mut magnitude = value
        .iter()
        .map(|bit| match bit {
            Value::V0 => Value::V1,
            Value::V1 => Value::V0,
            Value::X | Value::Z => unreachable!("unknowns handled before signed formatting"),
        })
        .collect::<Vec<_>>();
    let mut carry = true;
    for bit in magnitude.iter_mut().rev() {
        if !carry {
            break;
        }
        match bit {
            Value::V0 => {
                *bit = Value::V1;
                carry = false;
            }
            Value::V1 => *bit = Value::V0,
            Value::X | Value::Z => unreachable!(),
        }
    }
    format!("-{}", decimal_from_bits(magnitude))
}

fn decimal_from_bits(bits: impl IntoIterator<Item = Value>) -> String {
    let mut digits = vec![0_u8];
    for bit in bits {
        let mut carry = u8::from(bit == Value::V1);
        for digit in &mut digits {
            let next = *digit * 2 + carry;
            *digit = next % 10;
            carry = next / 10;
        }
        if carry != 0 {
            digits.push(carry);
        }
    }
    digits
        .into_iter()
        .rev()
        .map(|digit| char::from(b'0' + digit))
        .collect()
}

fn format_ascii(value: &[Value]) -> String {
    let padding = (8 - value.len() % 8) % 8;
    let padded = std::iter::repeat_n(Value::V0, padding)
        .chain(value.iter().copied())
        .collect::<Vec<_>>();
    let mut text = String::from("\"");
    for byte in padded.chunks_exact(8) {
        let number = byte.iter().fold(0_u8, |number, bit| {
            (number << 1) | u8::from(*bit == Value::V1)
        });
        match number {
            b'\\' => text.push_str("\\\\"),
            b'\"' => text.push_str("\\\""),
            0x20..=0x7e => text.push(char::from(number)),
            _ => text.push('.'),
        }
    }
    text.push('"');
    text
}

fn bit_character(value: Value) -> char {
    match value {
        Value::V0 => '0',
        Value::V1 => '1',
        Value::X => 'x',
        Value::Z => 'z',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bits(text: &str) -> Vec<Value> {
        text.chars()
            .map(|bit| match bit {
                '0' => Value::V0,
                '1' => Value::V1,
                'x' => Value::X,
                'z' => Value::Z,
                _ => panic!("invalid test bit"),
            })
            .collect()
    }

    #[test]
    fn formats_binary_hexadecimal_and_four_state_values() {
        assert_eq!(format(&bits("1"), ValueFormat::Hexadecimal), "1");
        assert_eq!(format(&bits("x"), ValueFormat::Hexadecimal), "x");
        assert_eq!(format(&bits("1010"), ValueFormat::Binary), "0b1010");
        assert_eq!(format(&bits("1010"), ValueFormat::Hexadecimal), "0xa");
        assert_eq!(format(&bits("xxxxzzzz"), ValueFormat::Hexadecimal), "0xxz");
        assert_eq!(format(&bits("10xz"), ValueFormat::Unsigned), "0b10xz");
    }

    #[test]
    fn formats_arbitrarily_wide_unsigned_and_twos_complement_values() {
        assert_eq!(format(&bits("11111111"), ValueFormat::Unsigned), "255");
        assert_eq!(format(&bits("01111111"), ValueFormat::Signed), "127");
        assert_eq!(format(&bits("11111111"), ValueFormat::Signed), "-1");
        assert_eq!(format(&bits("10000000"), ValueFormat::Signed), "-128");
        assert_eq!(
            format(
                &bits(&format!("1{}", "0".repeat(128))),
                ValueFormat::Unsigned
            ),
            "340282366920938463463374607431768211456"
        );
    }

    #[test]
    fn formats_ascii_with_escaped_and_nonprinting_bytes() {
        assert_eq!(
            format(&bits("0100000101000010"), ValueFormat::Ascii),
            "\"AB\""
        );
        assert_eq!(
            format(&bits("0000000001011100"), ValueFormat::Ascii),
            "\".\\\\\""
        );
    }
}

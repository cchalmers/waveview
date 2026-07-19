use waveview_model::vcd::Value;

pub fn format(value: &[Value]) -> String {
    if let [bit] = value {
        return match bit {
            Value::V0 => "0",
            Value::V1 => "1",
            Value::X => "x",
            Value::Z => "z",
        }
        .to_owned();
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_compact_and_preserve_four_state_logic() {
        assert_eq!(format(&[Value::V1]), "1");
        assert_eq!(format(&[Value::X]), "x");
        assert_eq!(format(&[Value::V1, Value::V0, Value::V1, Value::V0]), "0xa");
        assert_eq!(
            format(&[
                Value::X,
                Value::X,
                Value::X,
                Value::X,
                Value::Z,
                Value::Z,
                Value::Z,
                Value::Z,
            ]),
            "0xxz"
        );
    }
}

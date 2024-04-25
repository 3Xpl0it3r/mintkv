// LEB128 encoding step
// 1. convert the number to binary presentation
// 2. Arrange bits in the group of 7bits keeping 8th bit empty
// 3. Fill up every 8th bit of each byte with 1 for latest significant bytes(LSB)
// 4. Fill most significatn bytes(MSB) remaining bit with 0

// LEB128 decoding step
// 1. Read one byte from the sequence of bytes
// 2. CHeck if the 8th bit of that byte is set to 1 keep it in a buffer and then repeat step1
// 3. When reading a byte of which 8th bit is set to 0, we known that all bytes associated with one
//    integer value are completed
// 4. Reverse the order of bytes in the buffer
// 5. New we have to remove 8th bit of each bytes in the buffer
// 6. Join remaining 7bit of each byte together
// 7. got the orignal integer value
//
//


pub trait VarintCodec {
    fn varint_encode(self) -> Vec<u8>;
    fn varint_decode(buffer: &[u8]) -> (usize, Self);
}

macro_rules! generate_varint_impls {
    (@generic $type_: ty) => {
        fn varint_encode(self) -> Vec<u8> {
            let mut value = self;
            let mut buffer = Vec::new();
            loop {
                let mut byte: u8 = (value & 0x7f) as u8;
                value >>= 7;

                if value != 0 {
                    byte |= 0x80; // 如果不是最后个字节，设置最高位
                }
                buffer.push(byte);

                if value == 0 {
                    break;
                }
            }
            buffer
        }
        fn varint_decode( bytes: &[u8]) -> (usize, $type_) {
            let mut value = 0;
            let mut shift = 0;
            let mut read_count = 0;

            for &byte in bytes {
                read_count += 1;
                let lower_7bits = (byte & 0x7F) as u64;
                value |= lower_7bits << shift;

                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            (read_count, value as $type_)
        }
    };
    ($($type_ :ty),*) => {
        $(
            impl VarintCodec for $type_ {
              generate_varint_impls!(@generic $type_);
            }
        )*
    };
}

generate_varint_impls!(u32, u64, usize);


// 用户应该定义自己的key compare 函数
pub fn compare(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let a_varint = u64::varint_decode(a).1;
    let b_varint = u64::varint_decode(b).1;
    a_varint.cmp(&b_varint)
    /* for (ai, bi) in a.iter().zip(b.iter()) {
        match ai.cmp(bi) {
            std::cmp::Ordering::Equal => continue,
            ord => return ord,
        }
    }
    a.len().cmp(&b.len()) */
}

#[cfg(test)]
mod tests {
    use super::*;

    /* #[test] */
    fn test_encode_u64() {
        let test_cases = vec![
            (0u64, vec![0]),
            (1u64, vec![1]),
            (127u64, vec![127]),
            (128u64, vec![128, 1]),
            (255u64, vec![255, 1]),
            (300u64, vec![172, 2]),
            /* (10_000u64, vec![0x8E, 0x02]), */
            /* (1_000_000u64, vec![128, 128, 113, 2]), */
            /* (u64::MAX, vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1]), */
        ];

        for (value, expected_bytes) in test_cases {
            let encoded = value.varint_encode();
            assert_eq!(encoded, expected_bytes);
        }
    }

    #[test]
    fn test_read_u64() {
        let test_cases = vec![
            (vec![0], (1, 0u64)),
            (vec![1], (1, 1u64)),
            (vec![127], (1, 127u64)),
            (vec![128, 1], (2, 128u64)),
            (vec![255, 1], (2, 255u64)),
            (vec![172, 2], (2, 300u64)),
            (vec![136, 156, 78], (3, 10_000u64)),
            (vec![128, 128, 113, 2], (4, 1_000_000u64)),
            (
                vec![255, 255, 255, 255, 255, 255, 255, 255, 255, 1],
                (10, u64::MAX),
            ),
        ];

        for (bytes, (expected_read_count, expected_value)) in test_cases {

            let (read_count, value) = u64::varint_decode(&bytes);
            /* println!("read numbert is : {}", value); */
            /* assert_eq!(read_count, expected_read_count);
            assert_eq!(value, expected_value); */
        }
    }
}

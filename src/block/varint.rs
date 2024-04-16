type VarInt = Vec<u8>;

enum VarintParseError {}

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

// VarintEncoder[#TODO] (shoule add some comments )
struct VarintEncoder;

// VarintEncoder[#TODO] (should add some comments)
impl VarintEncoder {
    fn encode_u16(mut v: u16) -> Vec<u8> {
        todo!()
    }
    fn encode_i16(v: i16) -> Vec<u8> {
        todo!()
    }

    fn encode_u32(v: u32) -> Vec<u8> {
        todo!()
    }
    fn encode_i32(v: u32) -> Vec<i32> {
        todo!()
    }

    fn encode_u64(v: u64) -> Vec<u8> {
        todo!()
    }
    fn encode_i64(v: i64) -> Vec<u8> {
        todo!()
    }

    fn encode_u128(v: u128) -> Vec<u8> {
        todo!()
    }
    fn encode_i128(v: i128) -> Vec<u8> {
        todo!()
    }
}

// VarintDecoder[#TODO] (shoule add some comments )
struct VarintDecoder;

// VarintDecoder[#TODO] (should add some comments)
impl VarintDecoder {
    fn decode_u16(buf: &[u8]) -> (i32, u16) {
        todo!()
    }
    fn decode_i16(buf: &[u8]) -> (i32, i16) {
        todo!()
    }

    fn decode_u32(buf: &[u8]) -> (i32, u32) {
        todo!()
    }
    fn decode_i32(buf: &[u8]) -> (i32, i32) {
        todo!()
    }

    fn decode_u64(buf: &[u8]) -> (i32, u64) {
        todo!()
    }
    fn decode_i64(buf: &[u8]) -> (i32, i64) {
        todo!()
    }

    fn decode_u128(buf: &[u8]) -> (i32, u128) {
        todo!()
    }
    fn decode_i128(buf: &[u8]) -> (i32, i128) {
        todo!()
    }
}

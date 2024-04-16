use crate::errors::Error;

use super::constant::OpKind;

pub(crate) struct Encoder;

// Encoder[#TODO] (should add some comments)
impl Encoder {
    pub(super) fn encode(opkind: OpKind, data: &[u8]) -> Vec<u8> {
        let mut result = vec![0u8; data.len()];
        result[0] = opkind.as_u8();
        result[1..].clone_from_slice(data);
        result
    }

    fn decode(data: &[u8]) -> Result<(OpKind, &[u8]), Error> {
        todo!()
    }
}

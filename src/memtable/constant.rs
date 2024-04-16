pub(super) enum OpKind {
    Insert,
    Delete,
}
// OpKind[#TODO] (should add some comments)
impl OpKind {
    #[inline]
    pub(super) fn as_u8(self) -> u8 {
        match self {
            OpKind::Insert => 1,
            OpKind::Delete => 0,
        }
    }
}

mod constant;
mod meta;
mod node;
mod pager;
mod freelist;
mod btree;
pub mod error;
pub use btree::BTree as Mintkv;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}

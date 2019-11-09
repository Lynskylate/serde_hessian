pub mod value;
pub mod de;
pub mod ser;
pub mod error;
pub mod constant;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

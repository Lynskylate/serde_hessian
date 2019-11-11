pub mod constant;
pub mod de;
pub mod error;
pub mod ser;
pub mod value;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

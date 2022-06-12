#[macro_use]
extern crate serde_derive;

pub mod node;
pub mod error;
pub mod arch;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}



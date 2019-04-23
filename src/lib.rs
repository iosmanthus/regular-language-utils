pub mod ast;
pub mod re;

#[cfg(test)]
mod tests {
    #[test]
    fn test_re_new() {
        use crate::re::Re;
        println!("{:#?}", Re::new("1|2|3"));
    }
}

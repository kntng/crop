#[cfg(feature = "serde")]
mod tests {
    use crop::Rope;

    #[test]
    fn ser_de_empty() {
        let rope = Rope::new();

        serde_test::assert_tokens(&rope, &[serde_test::Token::Str("")]);
    }

    #[test]
    fn ser_de() {
        let mut rope = Rope::new();
        rope.insert(0, "lorem dolor");
        rope.insert(6, "ipsuma ");
        rope.delete(11..12);

        serde_test::assert_tokens(
            &rope,
            &[serde_test::Token::Str("lorem ipsum dolor")],
        );
    }

    #[test]
    fn ser_de_lf() {
        let mut rope = Rope::new();
        rope.insert(0, "lorem\n");
        rope.insert(6, "ipsum");

        serde_test::assert_tokens(
            &rope,
            &[serde_test::Token::Str("lorem\nipsum")],
        );
    }

    #[test]
    fn ser_de_crlf() {
        let mut rope = Rope::new();
        rope.insert(0, "lorem\r\n");
        rope.insert(7, "ipsum");

        serde_test::assert_tokens(
            &rope,
            &[serde_test::Token::Str("lorem\r\nipsum")],
        );
    }
}

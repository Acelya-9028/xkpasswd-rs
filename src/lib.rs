pub mod bit_flags;
pub mod prelude;
pub mod settings;
mod wasm;

#[cfg(test)]
mod tests {
    use super::bit_flags::*;
    use super::wasm::*;
    use wasm_bindgen_test::*;
    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_gen_passwd() {
        let pass = WasmXkpasswd::new();

        let settings = WasmSettings::default()
            .with_words_count(3)
            .with_word_lengths(4, 8)
            .with_separators(".")
            .with_padding_digits(0, 2)
            .with_padding_symbols("!@#$%^&*-_=+:|~?/;")
            .with_padding_symbol_lengths(0, 2)
            .with_word_transforms(&[
                FieldSize::from_flag(WordTransform::Lowercase),
                FieldSize::from_flag(WordTransform::Uppercase),
            ])
            .with_fixed_padding();
        assert_eq!(4, pass.gen_pass(&settings).split('.').count());
    }
}

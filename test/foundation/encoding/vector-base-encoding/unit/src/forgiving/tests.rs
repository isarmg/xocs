
    use super::*;

    use crate::AsOut;

    #[cfg_attr(not(target_arch = "wasm32"), test)]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_forgiving() {
        use const_str::hex;

        let mut inputs: Vec<&str> = Vec::new();
        let mut outputs: Vec<&[u8]> = Vec::new();

        {
            let mut i = |i| inputs.push(i);
            let mut o = |o| outputs.push(o);

            i("ab");
            o(&[0x69]);

            i("abc");
            o(&[0x69, 0xB7]);

            i("abcd");
            o(&[0x69, 0xB7, 0x1D]);

            i("helloworld");
            o(&hex!("85 E9 65 A3 0A 2B 95"));

            i(" h e l l o w o r\nl\rd\t");
            o(&hex!("85 E9 65 A3 0A 2B 95"));
        }

        for i in 0..inputs.len() {
            let (src, expected) = (inputs[i], outputs[i]);

            let mut buf = src.to_owned().into_bytes();

            let ans = forgiving_decode_inplace(&mut buf).unwrap();
            assert_eq!(ans, expected);

            let ans = crate::forgiving_decode(src.as_bytes(), buf.as_out()).unwrap();
            assert_eq!(ans, expected);

            #[cfg(feature = "alloc")]
            {
                let ans = crate::forgiving_decode_to_vec(src.as_bytes()).unwrap();
                assert_eq!(ans, expected);
            }
        }
    }

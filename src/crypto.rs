use sha2::digest::Output;
use sha2::{Digest, Sha512};

use argon2::PasswordHash;

fn split_by_parity(arr: &[u8; 256]) -> ([u8; 128], [u8; 128]) {
    let mut evens = [0u8; 128];
    let mut odds = [0u8; 128];

    for i in 0..128 {
        evens[i] = arr[i * 2];
        odds[i] = arr[i * 2 + 1];
    }

    (evens, odds)
}

fn digest_sha512(arr: &[u8; 128]) -> Output<Sha512> {
    Sha512::digest(arr)
}

#[expect(
    clippy::todo,
    unused_variables,
    reason = "Реализация Argon2id будет добавлена позже"
)]
fn digest_argon2id(arr: &[u8; 128], salt: &[u8], pepper: &[u8]) -> PasswordHash {
    todo!("Реализовать argon2id хеширование");
}

#[allow(unused)]
pub fn check_user() -> bool {
    let arr = [0u8; 256];
    let (first_arr, second_arr) = split_by_parity(&arr);
    let login = digest_sha512(&second_arr);
    let _hash = digest_argon2id(&first_arr, b"salt", b"pepper");
    true
}
#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn test_split_by_parity() {
        let data: [u8; 256] = std::array::from_fn(|i| (i % 2) as u8);

        assert_eq!(split_by_parity(&data), ([0u8; 128], [1u8; 128]))
    }

    #[test]
    fn test_digest_sha512_zeros() {
        // Тест с массивом из 128 нулей
        let input = [0u8; 128];

        let hash512 = digest_sha512(&input);

        assert_eq!(
            &hash512[..],
            &hex!(
                "ab942f526272e456ed68a979f50202905ca903a141ed98443567b11ef0bf25a552d639051a01be58558122c58e3de07d749ee59ded36acf0c55cd91924d6ba11"
            )[..]
        )
    }
}

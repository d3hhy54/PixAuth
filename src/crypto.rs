use argon2::PasswordVerifier;
use sha2::digest::Output;
use sha2::{Digest, Sha512};

use argon2::{Algorithm, Argon2, Params, PasswordHash, Version, password_hash::PasswordHasher};

use dotenvy;
use envy;

use serde::Deserialize;

use anyhow::Result;

#[derive(Deserialize)]
struct ArgonConfig {
    #[serde(rename = "argon2_m_cost")]
    m_cost: u32,
    #[serde(rename = "argon2_t_cost")]
    t_cost: u32,
    #[serde(rename = "argon2_p_cost")]
    p_cost: u32,
    #[allow(unused)]
    trash_argon2: String,
    pepper: String,
}

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

pub struct CryptoEngine {
    #[allow(unused)]
    argon2: Argon2<'static>,
    #[allow(unused)]
    config: ArgonConfig,
}

impl CryptoEngine {
    pub fn init() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = envy::from_env::<ArgonConfig>().expect("Missing or invalid args in .env");

        let pepper_bytes = config.pepper.clone().into_bytes();
        let leaked_pepper: &'static [u8] = Box::leak(pepper_bytes.into_boxed_slice());

        let params = Params::new(config.m_cost, config.t_cost, config.p_cost, None)?;

        let argon2 =
            Argon2::new_with_secret(leaked_pepper, Algorithm::Argon2id, Version::V0x13, params)?;

        Ok(Self { argon2, config })
    }

    #[allow(unused)]
    fn digest_argon2id(&self, arr: &[u8; 128]) -> Result<String, argon2::password_hash::Error> {
        let raw_salt = argon2::password_hash::generate_salt();
        let password_hash = self
            .argon2
            .hash_password_with_salt(arr, &raw_salt)?
            .to_string();

        Ok(password_hash)
    }

    #[allow(unused)]
    fn verify_password(&self, arr: &[u8; 128], hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)?;
        Ok(self.argon2.verify_password(arr, &parsed_hash).is_ok())
    }
}

#[allow(unused)]
pub fn check_user() {
    let arr = [0u8; 256];
    let (first_arr, second_arr) = split_by_parity(&arr);
    let login = digest_sha512(&second_arr);
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use std::sync::OnceLock;

    fn get_test_engine() -> &'static CryptoEngine {
        static ENGINE: OnceLock<CryptoEngine> = OnceLock::new();
        ENGINE.get_or_init(|| {
            unsafe {
                std::env::set_var("PEPPER", "super_secret_test_pepper_that_is_long_enough");
                std::env::set_var("ARGON2_M_COST", "4096"); // Маленькие значения, чтобы тесты
                std::env::set_var("ARGON2_T_COST", "2"); // прогонялись мгновенно
                std::env::set_var("ARGON2_P_COST", "1");
            }
            CryptoEngine::init().expect("Failed to initialize test CryptoEngine")
        })
    }

    // Вспомогательная функция для генерации тестового массива [u8; 128]
    fn make_test_arr(fill: u8) -> [u8; 128] {
        let mut arr = [0u8; 128];
        arr[0..5].copy_from_slice(&[fill; 5]); // Просто заполняем начало для уникальности
        arr
    }

    #[test]
    fn test_hash_and_verify_success() {
        let engine = get_test_engine();
        let password = make_test_arr(1);

        // Хешируем
        let hash = engine
            .digest_argon2id(&password)
            .expect("Failed to hash password");

        assert!(!hash.is_empty(), "Hash should not be empty");

        // Проверяем валидный пароль
        let is_valid = engine
            .verify_password(&password, &hash)
            .expect("Failed to verify password");

        assert!(is_valid, "Password verification should succeed");
    }

    #[test]
    fn test_verify_wrong_password() {
        let engine = get_test_engine();
        let correct_password = make_test_arr(1);
        let wrong_password = make_test_arr(2);

        let hash = engine.digest_argon2id(&correct_password).unwrap();

        // Проверяем неверный пароль
        let is_valid = engine.verify_password(&wrong_password, &hash).unwrap();

        assert!(!is_valid, "Verification should fail for wrong password");
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let engine = get_test_engine();
        let password = make_test_arr(1);
        let invalid_hash = "$argon2id$v=19$m=4096,t=3,p=1$invalidformat";

        // Проверяем, что ломается парсинг некорректного хеша
        let result = engine.verify_password(&password, invalid_hash);

        assert!(
            result.is_err(),
            "Should return an error for mangled hash format"
        );
    }

    #[test]
    fn test_split_by_parity() {
        // Заполняем массив числами 0, 1, 2, 3, ..., 255
        let data: [u8; 256] = std::array::from_fn(|i| i as u8);

        // Генерируем ожидаемые массивы
        // Чётные: 0, 2, 4, 6 ... 254
        let expected_even: [u8; 128] = std::array::from_fn(|i| (i * 2) as u8);
        // Нечётные: 1, 3, 5, 7 ... 255
        let expected_odd: [u8; 128] = std::array::from_fn(|i| (i * 2 + 1) as u8);

        assert_eq!(split_by_parity(&data), (expected_even, expected_odd));
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

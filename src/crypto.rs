use std::sync::OnceLock;

use argon2::PasswordVerifier;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha512;

use argon2::{Algorithm, Argon2, Params, PasswordHash, Version, password_hash::PasswordHasher};

use serde::Deserialize;

use anyhow::Result;

static PEPPER_STORAGE: OnceLock<Vec<u8>> = OnceLock::new();

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

pub struct CryptoEngine {
    #[allow(unused)]
    argon2: Argon2<'static>,
    #[allow(unused)]
    config: ArgonConfig,
}

pub struct HashedUser {
    #[allow(unused)]
    login: String,
    #[allow(unused)]
    password: Option<String>,
}

impl CryptoEngine {
    pub fn init() -> Result<Self> {
        dotenvy::dotenv().ok();

        let config = envy::from_env::<ArgonConfig>().expect("Missing or invalid args in .env");

        let pepper_vec = PEPPER_STORAGE.get_or_init(|| config.pepper.clone().into_bytes());
        let leaked_pepper: &'static [u8] = pepper_vec.as_slice();

        let params = Params::new(config.m_cost, config.t_cost, config.p_cost, None)?;

        let argon2 =
            Argon2::new_with_secret(leaked_pepper, Algorithm::Argon2id, Version::V0x13, params)?;

        Ok(Self { argon2, config })
    }

    #[allow(unused)]
    pub fn split_by_parity(arr: &[u8; 256]) -> ([u8; 128], [u8; 128]) {
        let mut evens = [0u8; 128];
        let mut odds = [0u8; 128];

        for i in 0..128 {
            evens[i] = arr[i * 2];
            odds[i] = arr[i * 2 + 1];
        }

        (evens, odds)
    }

    #[allow(unused)]
    fn digest_hmac_sha512(&self, arr: &[u8; 128]) -> Result<String> {
        type HmacSha512 = Hmac<Sha512>;
        let mut mac = HmacSha512::new_from_slice(self.config.pepper.as_bytes())?;
        mac.update(arr);
        let result = mac.finalize();
        let result = hex::encode(result.into_bytes());
        Ok(result)
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

    #[allow(unused)]
    pub fn verify_user(&self, arr: &[u8; 256], password_hash: Option<&str>) -> Result<bool> {
        let (arr, _) = Self::split_by_parity(arr);
        match password_hash {
            Some(hash) => Ok(self.verify_password(&arr, hash)?),
            None => Ok(self.verify_password(&arr, &self.config.trash_argon2)?),
        }
    }

    #[allow(unused)]
    pub fn hash_user(&self, arr: &[u8; 256], password: bool) -> Result<HashedUser> {
        let (evens, odds) = Self::split_by_parity(arr);
        let hash_login = self.digest_hmac_sha512(&odds)?;
        let hash_password = if password {
            Some(self.digest_argon2id(&evens)?)
        } else {
            None
        };
        Ok(HashedUser {
            login: hash_login,
            password: hash_password,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Улучшенная версия: не трогает env, а собирает конфиг руками
    fn get_test_engine() -> &'static CryptoEngine {
        static ENGINE: OnceLock<CryptoEngine> = OnceLock::new();
        ENGINE.get_or_init(|| {
            // Создаем конфигурацию напрямую, без опасных манипуляций с std::env
            let config = ArgonConfig {
                m_cost: 4096, // Быстро для тестов
                t_cost: 2,
                p_cost: 1,
                trash_argon2: "$argon2id$v=19$m=4096,t=2,p=1$c29tZXJhbmRvbXNhbHQ$vR4S/zG/q+jP2vI35Z1NfA3k9dJxl6QzU6jX8jL5Zok".to_string(), // Валидный хэш под наши параметры!
                pepper: "super_secret_test_pepper_that_is_long_enough_for_hmac_and_argon".to_string(),
            };

            let pepper_vec = PEPPER_STORAGE.get_or_init(|| config.pepper.clone().into_bytes());
            let leaked_pepper: &'static [u8] = pepper_vec.as_slice();
            let params = Params::new(config.m_cost, config.t_cost, config.p_cost, None).unwrap();
            let argon2 = Argon2::new_with_secret(leaked_pepper, Algorithm::Argon2id, Version::V0x13, params).unwrap();

            CryptoEngine { argon2, config }
        })
    }

    #[test]
    fn test_split_by_parity_correctness() {
        // Создаем массив [0, 1, 2, 3, 4, 5, ..., 255]
        let mut input = [0u8; 256];
        for i in 0..256 {
            input[i] = i as u8;
        }

        let (evens, odds) = CryptoEngine::split_by_parity(&input);

        // Проверяем четные индексы: 0, 2, 4... должны превратиться в 0, 2, 4...
        assert_eq!(evens[0], 0);
        assert_eq!(evens[1], 2);
        assert_eq!(evens[127], 254);

        // Проверяем нечетные индексы: 1, 3, 5... должны превратиться в 1, 3, 5...
        assert_eq!(odds[0], 1);
        assert_eq!(odds[1], 3);
        assert_eq!(odds[127], 255);
    }

    #[test]
    fn test_hash_and_verify_success() {
        let engine = get_test_engine();
        let input_data = [42u8; 256]; // Имитируем какой-то ключ/пароль на 256 байт

        // Хешируем пользователя (включая пароль)
        let hashed_user = engine.hash_user(&input_data, true).unwrap();

        assert!(
            !hashed_user.login.is_empty(),
            "HMAC логина не должен быть пустым"
        );
        assert!(
            hashed_user.password.is_some(),
            "Хеш пароля должен присутствовать"
        );

        let pwd_hash_str = hashed_user.password.unwrap();
        assert!(
            pwd_hash_str.contains("$argon2id$"),
            "Это должен быть валидный Argon2id хэш"
        );

        // Проверяем верификацию существующего пользователя
        let is_valid = engine
            .verify_user(&input_data, Some(&pwd_hash_str))
            .unwrap();
        assert!(
            is_valid,
            "Валидные данные должны успешно проходить верификацию"
        );
    }

    #[test]
    fn test_verify_user_wrong_password() {
        let engine = get_test_engine();
        let input_data = [42u8; 256];
        let mut wrong_data = [42u8; 256];
        wrong_data[0] = 0; // Чуть-чуть ломаем входные данные для проверки пароля

        let hashed_user = engine.hash_user(&input_data, true).unwrap();

        // Передаем измененные данные со старым хэшем
        let is_valid = engine
            .verify_user(&wrong_data, Some(&hashed_user.password.unwrap()))
            .unwrap();
        assert!(
            !is_valid,
            "Измененные данные не должны проходить верификацию"
        );
    }

    #[test]
    fn test_verify_user_missing_hash_protection() {
        let engine = get_test_engine();
        let input_data = [77u8; 256];

        // Замеряем время, чтобы убедиться, что фейковое хеширование РАБОТАЕТ (нет тайминг-атаки)
        let start = std::time::Instant::now();

        // Передаем None вместо хэша (пользователя нет в БД)
        let is_valid = engine.verify_user(&input_data, None).unwrap();

        let duration = start.elapsed();

        assert!(
            !is_valid,
            "Если хэша нет, верификация всегда должна возвращать false"
        );

        // Убеждаемся, что Argon2 крутился, а не вылетел за микросекунду.
        // Наш заниженный m_cost=4096 обычно выполняется от 1 до 10+ миллисекунд (зависит от процессора).
        // Проверим, что это заняло хотя бы больше 50 микросекунд (обычный парсинг строки занимает < 1 мкс).
        assert!(
            duration.as_micros() > 50,
            "Слишком быстрый ответ ({:?})! Похоже, фейковый Argon2 не выполнился.",
            duration
        );
    }

    #[test]
    fn test_hash_user_without_password() {
        let engine = get_test_engine();
        let input_data = [99u8; 256];

        // Генерируем только логин (например, для проверки существования или быстрой сверки)
        let hashed_user = engine.hash_user(&input_data, false).unwrap();

        assert!(!hashed_user.login.is_empty());
        assert!(
            hashed_user.password.is_none(),
            "Если password = false, Argon2 не должен запускаться"
        );
    }
}

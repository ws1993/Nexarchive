use std::{fs, path::PathBuf};

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use rand::RngCore;

use crate::models::AppConfig;

const ENC_PREFIX: &str = "enc:";

pub struct ConfigService {
    config_path: PathBuf,
    secret_key_path: PathBuf,
}

impl ConfigService {
    pub fn new(app_data_dir: PathBuf) -> Self {
        Self {
            config_path: app_data_dir.join("config.json"),
            secret_key_path: app_data_dir.join("secret.key"),
        }
    }

    pub fn load_config(&self) -> Result<AppConfig> {
        if !self.config_path.exists() {
            return Ok(AppConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)
            .with_context(|| format!("cannot read {}", self.config_path.display()))?;
        let mut config: AppConfig =
            serde_json::from_str(&content).context("invalid config json")?;

        if config.llm.api_key_encrypted.starts_with(ENC_PREFIX) {
            config.llm.api_key_encrypted = self.decrypt_api_key(&config.llm.api_key_encrypted)?;
        }
        if config.mineru.api_token_encrypted.starts_with(ENC_PREFIX) {
            config.mineru.api_token_encrypted =
                self.decrypt_api_key(&config.mineru.api_token_encrypted)?;
        }
        if config.updater.proxy_url_encrypted.starts_with(ENC_PREFIX) {
            config.updater.proxy_url_encrypted =
                self.decrypt_api_key(&config.updater.proxy_url_encrypted)?;
        }

        Ok(config)
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        let mut disk = config.clone();
        if !disk.llm.api_key_encrypted.is_empty()
            && !disk.llm.api_key_encrypted.starts_with(ENC_PREFIX)
        {
            disk.llm.api_key_encrypted = self.encrypt_api_key(&disk.llm.api_key_encrypted)?;
        }
        if !disk.mineru.api_token_encrypted.is_empty()
            && !disk.mineru.api_token_encrypted.starts_with(ENC_PREFIX)
        {
            disk.mineru.api_token_encrypted = self.encrypt_api_key(&disk.mineru.api_token_encrypted)?;
        }
        if !disk.updater.proxy_url_encrypted.is_empty()
            && !disk.updater.proxy_url_encrypted.starts_with(ENC_PREFIX)
        {
            disk.updater.proxy_url_encrypted =
                self.encrypt_api_key(&disk.updater.proxy_url_encrypted)?;
        }

        let content = serde_json::to_string_pretty(&disk)?;
        fs::write(&self.config_path, content)
            .with_context(|| format!("cannot write {}", self.config_path.display()))?;
        Ok(())
    }

    fn encrypt_api_key(&self, plain: &str) -> Result<String> {
        if plain.is_empty() {
            return Ok(String::new());
        }

        let key = self.ensure_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key).context("invalid key length")?;

        let mut nonce_raw = [0_u8; 12];
        rand::rngs::OsRng.fill_bytes(&mut nonce_raw);
        let nonce = Nonce::from_slice(&nonce_raw);

        let ciphertext = cipher
            .encrypt(nonce, plain.as_bytes())
            .map_err(|_| anyhow!("encrypt api key failed"))?;

        let mut payload = nonce_raw.to_vec();
        payload.extend_from_slice(&ciphertext);

        Ok(format!("{ENC_PREFIX}{}", STANDARD.encode(payload)))
    }

    fn decrypt_api_key(&self, encrypted: &str) -> Result<String> {
        if encrypted.is_empty() {
            return Ok(String::new());
        }

        if !encrypted.starts_with(ENC_PREFIX) {
            return Ok(encrypted.to_string());
        }

        let encoded = encrypted.trim_start_matches(ENC_PREFIX);
        let payload = STANDARD.decode(encoded).context("api key decode failed")?;
        if payload.len() <= 12 {
            anyhow::bail!("invalid encrypted payload");
        }

        let key = self.ensure_key()?;
        let cipher = Aes256Gcm::new_from_slice(&key).context("invalid key length")?;

        let nonce = Nonce::from_slice(&payload[..12]);
        let plaintext = cipher
            .decrypt(nonce, &payload[12..])
            .map_err(|_| anyhow!("decrypt api key failed"))?;

        Ok(String::from_utf8(plaintext).context("api key utf8 decode failed")?)
    }

    fn ensure_key(&self) -> Result<[u8; 32]> {
        if self.secret_key_path.exists() {
            let key = fs::read(&self.secret_key_path)?;
            if key.len() == 32 {
                let mut out = [0_u8; 32];
                out.copy_from_slice(&key);
                return Ok(out);
            }
        }

        let mut key = [0_u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut key);
        fs::write(&self.secret_key_path, key)?;
        Ok(key)
    }
}

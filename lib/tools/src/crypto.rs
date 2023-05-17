use std::{fs, sync::Arc};

use anyhow::anyhow;
use ed25519::{
    pkcs8::{
        spki::der::pem::LineEnding, DecodePrivateKey, DecodePublicKey,
        EncodePrivateKey, EncodePublicKey,
    },
    signature::{Signer, Verifier},
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;

use crate::atomic::Atomic;

pub trait ToPublicKeyPem {
    fn public_key(&self) -> Result<String, Ed25519Error>;
}

#[derive(thiserror::Error, Debug)]
pub enum Ed25519Error {
    #[error(transparent)]
    FromPemError(anyhow::Error),
    #[error(transparent)]
    ReadFileError(anyhow::Error),
    #[error(transparent)]
    WriteFileError(anyhow::Error),
    #[error(transparent)]
    ToPkcs8PemError(anyhow::Error),
    #[error(transparent)]
    SigningError(ed25519::Error),
    #[error(transparent)]
    VerifyError(ed25519::Error),
    #[error(transparent)]
    ToPublickeyPemError(anyhow::Error),
}

pub struct KeyGenerator;

impl KeyGenerator {
    #[inline]
    pub fn generate() -> SigningKey {
        let mut csprng = OsRng;
        SigningKey::generate(&mut csprng)
    }
}

#[derive(Debug, Clone)]
pub struct SignerManager {
    pub signer: Arc<Atomic<MessageSigner>>,
}

impl SignerManager {
    #[inline]
    pub fn new(signer: MessageSigner) -> Self {
        Self { signer: Atomic::new(signer).into() }
    }

    #[inline]
    pub fn new_rand() -> Self {
        Self::new(MessageSigner::new_rand())
    }

    #[inline]
    pub fn store_to_pem_file(&self, path: &str) -> Result<(), Ed25519Error> {
        self.signer.load().store_to_pem_file(path)
    }

    #[inline]
    pub fn from_pem_file(
        ed25519_private_key_file_path: &str,
    ) -> Result<Self, Ed25519Error> {
        Ok(Self::new(MessageSigner::from_pem_file(
            ed25519_private_key_file_path,
        )?))
    }

    #[inline]
    pub fn from_pem(ed25519_private_key: &str) -> Result<Self, Ed25519Error> {
        Ok(Self::new(MessageSigner::from_pem(ed25519_private_key)?))
    }

    #[inline]
    pub fn reload_signer(&self, signer: MessageSigner) {
        self.signer.store(signer.into())
    }

    #[inline]
    pub fn reload_from_pem(
        &self,
        ed25519_private_key: &str,
    ) -> Result<(), Ed25519Error> {
        self.reload_signer(MessageSigner::from_pem(ed25519_private_key)?);
        Ok(())
    }
    #[inline]
    pub fn sign(
        &self,
        message: &[u8],
    ) -> Result<ed25519::Signature, Ed25519Error> {
        self.signer.load().sign(message)
    }

    #[inline]
    pub fn verify(
        &self,
        message: &[u8],
        signature: &ed25519::Signature,
    ) -> Result<(), Ed25519Error> {
        self.signer.load().verify(message, signature)
    }
    #[inline]
    pub fn reload_from_pem_file(&self, path: &str) -> Result<(), Ed25519Error> {
        self.reload_signer(MessageSigner::from_pem_file(path)?);
        Ok(())
    }

    #[inline]
    pub fn public_key(&self) -> Result<String, Ed25519Error> {
        self.signer.load().public_key()
    }
}

#[derive(Debug, Clone)]
pub struct MessageSigner {
    pub signing_key: SigningKey,
}

impl MessageSigner {
    #[inline]
    pub fn new(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }

    #[inline]
    pub fn new_rand() -> Self {
        Self { signing_key: KeyGenerator::generate() }
    }

    #[inline]
    pub fn store_to_pem_file(&self, path: &str) -> Result<(), Ed25519Error> {
        let contents = self
            .signing_key
            .to_pkcs8_pem(LineEnding::LF)
            .map_err(|err| Ed25519Error::ToPkcs8PemError(anyhow!(err)))?;

        fs::write(path, contents)
            .map_err(|err| Ed25519Error::WriteFileError(anyhow!(err)))
    }

    #[inline]
    pub fn from_pem(pem: &str) -> Result<Self, Ed25519Error> {
        Ok(Self {
            signing_key: SigningKey::from_pkcs8_pem(pem)
                .map_err(|err| Ed25519Error::FromPemError(anyhow!(err)))?,
        })
    }

    #[inline]
    pub fn from_pem_file(path: &str) -> Result<Self, Ed25519Error> {
        Self::from_pem(
            fs::read_to_string(path)
                .map_err(|err| Ed25519Error::ReadFileError(anyhow!(err)))?
                .as_str(),
        )
    }

    #[inline]
    pub fn sign(
        &self,
        message: &[u8],
    ) -> Result<ed25519::Signature, Ed25519Error> {
        self.signing_key.try_sign(message).map_err(Ed25519Error::SigningError)
    }

    #[inline]
    pub fn verify(
        &self,
        message: &[u8],
        signature: &ed25519::Signature,
    ) -> Result<(), Ed25519Error> {
        self.signing_key
            .verify(message, signature)
            .map_err(Ed25519Error::VerifyError)
    }
}

impl From<SigningKey> for MessageSigner {
    fn from(signing_key: SigningKey) -> Self {
        Self { signing_key }
    }
}

#[derive(Debug, Clone)]
pub struct MessageVerifier {
    pub verifying_key: VerifyingKey,
}

impl MessageVerifier {
    #[inline]
    pub fn new(verifying_key: VerifyingKey) -> Self {
        Self { verifying_key }
    }

    #[inline]
    pub fn verify(
        &self,
        message: &[u8],
        signature: &ed25519::Signature,
    ) -> Result<(), Ed25519Error> {
        self.verifying_key
            .verify(message, signature)
            .map_err(Ed25519Error::VerifyError)
    }

    #[inline]
    pub fn from_pem(pem: &str) -> Result<Self, Ed25519Error> {
        Ok(Self {
            verifying_key: VerifyingKey::from_public_key_pem(pem)
                .map_err(|err| Ed25519Error::FromPemError(anyhow!(err)))?,
        })
    }

    #[inline]
    pub fn from_pem_file(path: &str) -> Result<Self, Ed25519Error> {
        Self::from_pem(
            fs::read_to_string(path)
                .map_err(|err| Ed25519Error::ReadFileError(anyhow!(err)))?
                .as_str(),
        )
    }
}

impl From<VerifyingKey> for MessageVerifier {
    fn from(verifying_key: VerifyingKey) -> Self {
        Self { verifying_key }
    }
}

impl ToPublicKeyPem for MessageSigner {
    #[inline]
    fn public_key(&self) -> Result<String, Ed25519Error> {
        self.signing_key
            .verifying_key()
            .to_public_key_pem(LineEnding::LF)
            .map_err(|err| Ed25519Error::ToPublickeyPemError(anyhow!(err)))
    }
}

impl ToPublicKeyPem for MessageVerifier {
    #[inline]
    fn public_key(&self) -> Result<String, Ed25519Error> {
        self.verifying_key
            .to_public_key_pem(LineEnding::LF)
            .map_err(|err| Ed25519Error::ToPublickeyPemError(anyhow!(err)))
    }
}

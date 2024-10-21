use std::fmt::{Display, Formatter};
use std::marker::Sized;

use rsa::pss::Signature;
use rsa::signature::{Keypair, RandomizedSigner, Verifier};
use rsa::{sha2::Sha256, RsaPrivateKey};
use serde::{Deserialize, Serialize};

/*
--------------------------------------------------------------------------------
||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||||
--------------------------------------------------------------------------------
*/

pub type SigningKey = rsa::pss::BlindedSigningKey<Sha256>;
pub type VerifyingKey = rsa::pss::VerifyingKey<Sha256>;

// We don't really want the capabilities to be secure for this proof of concept, we'd rather have the emulator be fast
// 1024 bits is still relatively slow-ish in --DEBUG mode, in --RELEASE mode it's quite fast
const KEY_SIZE: usize = 1024;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Signed<T> {
	#[serde(skip)]
	signature: Option<Signature>,

	inner: T,
}

pub fn create_key_pair() -> (SigningKey, VerifyingKey) {
	let mut rng = rand::thread_rng();
	let private_key = RsaPrivateKey::new(&mut rng, KEY_SIZE).expect("Failed to generate a key.");

	let signing_key = SigningKey::new(private_key);
	let verifying_key = signing_key.verifying_key();

	(signing_key, verifying_key)
}

impl<T> Signed<T>
where
	T: Serialize,
{
	pub fn new_signed(inner: T, signing_key: &SigningKey) -> Signed<T> {
		let mut rng = rand::thread_rng();

		let data = bincode::serialize(&inner).expect("Failed to serialize inner type to bincode.");
		let signature = Some(signing_key.sign_with_rng(&mut rng, &data));

		Self { signature, inner }
	}

	pub fn new_unsigned(inner: T) -> Signed<T> {
		Self { signature: None, inner }
	}

	pub fn verify(self, verifying_key: &VerifyingKey) -> Option<T> {
		let data = bincode::serialize(&self.inner).expect("Failed to serialize inner type to bincode.");

		verifying_key.verify(&data, &self.signature?).ok()?;

		Some(self.inner)
	}

	pub fn re_signed(self, signing_key: &SigningKey) -> Self {
		Self::new_signed(self.inner, signing_key)
	}
}

impl<T> Display for Signed<T>
where
	T: Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "${}", self.inner)
	}
}

pub trait Signable {
	fn signed(self, signing_key: &SigningKey) -> Signed<Self>
	where
		Self: Sized;
}

impl<T> Signable for T
where
	T: Serialize,
{
	fn signed(self, signing_key: &SigningKey) -> Signed<Self> {
		Signed::<Self>::new_signed(self, signing_key)
	}
}

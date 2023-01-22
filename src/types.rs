
use parity_scale_codec::{Decode, Encode};
use sp_runtime::serde::{Deserialize, Serialize};
use sp_runtime::traits::Extrinsic;

// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Error {
	NoSignature,
	NoAdminKey,
	InvalidSignature,
	SupplyOverflow,
	BalanceTooLow,
	FeeTooLow,

}

// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub enum Call {
	Mint([u8; 32], u128),
	Transfer([u8; 32], [u8; 32], u128),
	SetMinFee(u32),
	Upgrade(Vec<u8>),
}

// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct ExtrinsicPayload {
	pub call: Call,
	pub fee: u32
}

impl ExtrinsicPayload {
	pub fn new(call: Call, fee: u32) -> ExtrinsicPayload{
		ExtrinsicPayload {
			call,
			fee
		}
	}
}

// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Encode, Decode, Debug, PartialEq, Eq, Clone)]
pub struct Signature{
	pub signature: Vec<u8>,
	pub origin: Vec<u8>
}

impl Signature {
	// pub fn new(sig: [u8;64], origin:) -> Self{
	// 	Signature(sig.to_vec())
	// }

	pub fn origin_as_array(&self) -> [u8;32] {
		Signature::to_array(self.origin.clone())
	}

	fn to_array<T, const N: usize>(v: Vec<T>) -> [T; N] {
		v.try_into()
			.unwrap_or_else(|v: Vec<T>| panic!("Expected a Vec of length {} but it was {}", N, v.len()))
	}
}

impl Into<sp_core::sr25519::Signature> for Signature {
	fn into(self) -> sp_core::sr25519::Signature {
		sp_core::sr25519::Signature(Signature::to_array(self.signature))
	}

}

// this extrinsic type does nothing other than fulfill the compiler.
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize, parity_util_mem::MallocSizeOf))]
#[derive(Debug, Encode, Decode, PartialEq, Eq, Clone)]
pub struct BasicExtrinsic(pub ExtrinsicPayload,  pub Option<Signature>);

#[cfg(test)]
impl BasicExtrinsic {
	pub fn new_unsigned(payload: ExtrinsicPayload) -> Self {
		<Self as Extrinsic>::new(payload, None).unwrap()
	}
}

impl sp_runtime::traits::Extrinsic for BasicExtrinsic {
	type Call = ExtrinsicPayload;
	type SignaturePayload = Signature;

	fn new(data: Self::Call, signature: Option<Self::SignaturePayload>) -> Option<Self> {
		Some(Self(data, signature))
	}
}


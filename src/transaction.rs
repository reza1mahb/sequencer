use std::collections::BTreeMap;
use std::fmt::Display;
use std::sync::Arc;

use derive_more::From;
use serde::{Deserialize, Serialize};

use crate::block::{BlockHash, BlockNumber};
use crate::core::{
    ClassHash, CompiledClassHash, ContractAddress, EntryPointSelector, EthAddress, Nonce,
};
use crate::hash::{StarkFelt, StarkHash};
use crate::serde_utils::PrefixedBytesAsHex;

/// A transaction.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum Transaction {
    /// A declare transaction.
    Declare(DeclareTransaction),
    /// A deploy transaction.
    Deploy(DeployTransaction),
    /// A deploy account transaction.
    DeployAccount(DeployAccountTransaction),
    /// An invoke transaction.
    Invoke(InvokeTransaction),
    /// An L1 handler transaction.
    L1Handler(L1HandlerTransaction),
}

/// A transaction output.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum TransactionOutput {
    /// A declare transaction output.
    Declare(DeclareTransactionOutput),
    /// A deploy transaction output.
    Deploy(DeployTransactionOutput),
    /// A deploy account transaction output.
    DeployAccount(DeployAccountTransactionOutput),
    /// An invoke transaction output.
    Invoke(InvokeTransactionOutput),
    /// An L1 handler transaction output.
    L1Handler(L1HandlerTransactionOutput),
}

impl TransactionOutput {
    pub fn actual_fee(&self) -> Fee {
        match self {
            TransactionOutput::Declare(output) => output.actual_fee,
            TransactionOutput::Deploy(output) => output.actual_fee,
            TransactionOutput::DeployAccount(output) => output.actual_fee,
            TransactionOutput::Invoke(output) => output.actual_fee,
            TransactionOutput::L1Handler(output) => output.actual_fee,
        }
    }

    pub fn events(&self) -> &[Event] {
        match self {
            TransactionOutput::Declare(output) => &output.events,
            TransactionOutput::Deploy(output) => &output.events,
            TransactionOutput::DeployAccount(output) => &output.events,
            TransactionOutput::Invoke(output) => &output.events,
            TransactionOutput::L1Handler(output) => &output.events,
        }
    }
}

/// A StorageDomain.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum StorageDomain {
    #[default]
    OnChain,
    OffChain,
}

/// Account parameters.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct AccountParams {
    pub max_fee: Fee,
    pub signature: TransactionSignature,
    pub nonce: Nonce,
}

/// Transaction parameters V3.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct TransactionParamsV3 {
    pub signature: TransactionSignature,
    pub nonce: Nonce,
    pub nonce_da_mode: StorageDomain,
    pub fee_da_mode: StorageDomain,
    pub resource_bounds: ResourceBounds,
    pub tip: Tip,
}

/// A declare V0 or V1 transaction (same schema but different version).
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeclareTransactionV0V1 {
    #[serde(flatten)]
    pub account_params: AccountParams,
    pub class_hash: ClassHash,
    pub sender_address: ContractAddress,
}

/// A declare V2 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeclareTransactionV2 {
    #[serde(flatten)]
    pub account_params: AccountParams,
    pub class_hash: ClassHash,
    pub compiled_class_hash: CompiledClassHash,
    pub sender_address: ContractAddress,
}

/// A declare V3 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeclareTransactionV3 {
    pub transaction_params: TransactionParamsV3,
    pub class_hash: ClassHash,
    pub compiled_class_hash: CompiledClassHash,
    pub sender_address: ContractAddress,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum DeclareTransaction {
    V0(DeclareTransactionV0V1),
    V1(DeclareTransactionV0V1),
    V2(DeclareTransactionV2),
    V3(DeclareTransactionV3),
}

macro_rules! implement_declare_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(
            pub fn $field(&self) -> $field_type {
                match self {
                    Self::V0(tx) => tx.$field.clone(),
                    Self::V1(tx) => tx.$field.clone(),
                    Self::V2(tx) => tx.$field.clone(),
                    Self::V3(tx) => tx.$field.clone(),
                }
            }
        )*
    };
}

impl DeclareTransaction {
    implement_declare_tx_getters!((class_hash, ClassHash), (sender_address, ContractAddress));

    pub fn max_fee(&self) -> Option<Fee> {
        match self {
            Self::V0(tx) => Some(tx.account_params.max_fee),
            Self::V1(tx) => Some(tx.account_params.max_fee),
            Self::V2(tx) => Some(tx.account_params.max_fee),
            Self::V3(_) => None,
        }
    }
    pub fn signature(&self) -> TransactionSignature {
        match self {
            Self::V0(tx) => tx.account_params.signature.clone(),
            Self::V1(tx) => tx.account_params.signature.clone(),
            Self::V2(tx) => tx.account_params.signature.clone(),
            Self::V3(tx) => tx.transaction_params.signature.clone(),
        }
    }
    pub fn nonce(&self) -> Nonce {
        match self {
            Self::V0(tx) => tx.account_params.nonce,
            Self::V1(tx) => tx.account_params.nonce,
            Self::V2(tx) => tx.account_params.nonce,
            Self::V3(tx) => tx.transaction_params.nonce,
        }
    }
    pub fn compiled_class_hash(&self) -> Option<CompiledClassHash> {
        match self {
            Self::V0(_) => None,
            Self::V1(_) => None,
            Self::V2(tx) => Some(tx.compiled_class_hash),
            Self::V3(tx) => Some(tx.compiled_class_hash),
        }
    }
    pub fn version(&self) -> TransactionVersion {
        match self {
            DeclareTransaction::V0(_) => TransactionVersion(StarkFelt::from(0_u8)),
            DeclareTransaction::V1(_) => TransactionVersion(StarkFelt::from(1_u8)),
            DeclareTransaction::V2(_) => TransactionVersion(StarkFelt::from(2_u8)),
            DeclareTransaction::V3(_) => TransactionVersion(StarkFelt::from(3_u8)),
        }
    }
}

/// A deploy account V1 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeployAccountTransactionV1 {
    #[serde(flatten)]
    pub account_params: AccountParams,
    pub class_hash: ClassHash,
    pub contract_address_salt: ContractAddressSalt,
    pub constructor_calldata: Calldata,
}

/// A deploy account V3 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeployAccountTransactionV3 {
    pub transaction_params: TransactionParamsV3,
    pub class_hash: ClassHash,
    pub contract_address_salt: ContractAddressSalt,
    pub constructor_calldata: Calldata,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord, From)]
pub enum DeployAccountTransaction {
    V1(DeployAccountTransactionV1),
    V3(DeployAccountTransactionV3),
}

macro_rules! implement_deploy_account_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(
            pub fn $field(&self) -> $field_type {
                match self {
                    Self::V1(tx) => tx.$field.clone(),
                    Self::V3(tx) => tx.$field.clone(),
                }
            }
        )*
    };
}

impl DeployAccountTransaction {
    implement_deploy_account_tx_getters!(
        (class_hash, ClassHash),
        (constructor_calldata, Calldata),
        (contract_address_salt, ContractAddressSalt)
    );

    pub fn max_fee(&self) -> Option<Fee> {
        match self {
            Self::V1(tx) => Some(tx.account_params.max_fee),
            Self::V3(_) => None,
        }
    }
    pub fn signature(&self) -> TransactionSignature {
        match self {
            Self::V1(tx) => tx.account_params.signature.clone(),
            Self::V3(tx) => tx.transaction_params.signature.clone(),
        }
    }
    pub fn nonce(&self) -> Nonce {
        match self {
            Self::V1(tx) => tx.account_params.nonce,
            Self::V3(tx) => tx.transaction_params.nonce,
        }
    }
    pub fn version(&self) -> TransactionVersion {
        match self {
            DeployAccountTransaction::V1(_) => TransactionVersion(StarkFelt::from(1_u8)),
            DeployAccountTransaction::V3(_) => TransactionVersion(StarkFelt::from(3_u8)),
        }
    }
}

/// A deploy transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeployTransaction {
    pub version: TransactionVersion,
    pub class_hash: ClassHash,
    pub contract_address_salt: ContractAddressSalt,
    pub constructor_calldata: Calldata,
}

/// An invoke V0 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct InvokeTransactionV0 {
    pub max_fee: Fee,
    pub signature: TransactionSignature,
    pub contract_address: ContractAddress,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
}

/// An invoke V1 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct InvokeTransactionV1 {
    #[serde(flatten)]
    pub account_params: AccountParams,
    pub sender_address: ContractAddress,
    pub calldata: Calldata,
}

/// An invoke V3 transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct InvokeTransactionV3 {
    pub transaction_params: TransactionParamsV3,
    pub sender_address: ContractAddress,
    pub calldata: Calldata,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord, From)]
pub enum InvokeTransaction {
    V0(InvokeTransactionV0),
    V1(InvokeTransactionV1),
    V3(InvokeTransactionV3),
}

macro_rules! implement_invoke_tx_getters {
    ($(($field:ident, $field_type:ty)),*) => {
        $(
            pub fn $field(&self) -> $field_type {
                match self {
                    Self::V0(tx) => tx.$field.clone(),
                    Self::V1(tx) => tx.$field.clone(),
                    Self::V3(tx) => tx.$field.clone(),
                }
            }
        )*
    };
}

impl InvokeTransaction {
    implement_invoke_tx_getters!((calldata, Calldata));
    pub fn max_fee(&self) -> Option<Fee> {
        match self {
            Self::V0(tx) => Some(tx.max_fee),
            Self::V1(tx) => Some(tx.account_params.max_fee),
            Self::V3(_) => None,
        }
    }
    pub fn signature(&self) -> TransactionSignature {
        match self {
            Self::V0(tx) => tx.signature.clone(),
            Self::V1(tx) => tx.account_params.signature.clone(),
            Self::V3(tx) => tx.transaction_params.signature.clone(),
        }
    }
    pub fn nonce(&self) -> Option<Nonce> {
        match self {
            Self::V0(_) => None,
            Self::V1(tx) => Some(tx.account_params.nonce),
            Self::V3(tx) => Some(tx.transaction_params.nonce),
        }
    }
    pub fn sender_address(&self) -> ContractAddress {
        match self {
            Self::V0(tx) => tx.contract_address,
            Self::V1(tx) => tx.sender_address,
            Self::V3(tx) => tx.sender_address,
        }
    }
    pub fn version(&self) -> TransactionVersion {
        match self {
            InvokeTransaction::V0(_) => TransactionVersion(StarkFelt::from(0_u8)),
            InvokeTransaction::V1(_) => TransactionVersion(StarkFelt::from(1_u8)),
            InvokeTransaction::V3(_) => TransactionVersion(StarkFelt::from(3_u8)),
        }
    }
}

/// An L1 handler transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct L1HandlerTransaction {
    pub version: TransactionVersion,
    pub nonce: Nonce,
    pub contract_address: ContractAddress,
    pub entry_point_selector: EntryPointSelector,
    pub calldata: Calldata,
}

/// A declare transaction output.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeclareTransactionOutput {
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
}

/// A deploy-account transaction output.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeployAccountTransactionOutput {
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
    pub contract_address: ContractAddress,
}

/// A deploy transaction output.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct DeployTransactionOutput {
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
    pub contract_address: ContractAddress,
}

/// An invoke transaction output.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct InvokeTransactionOutput {
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
}

/// An L1 handler transaction output.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct L1HandlerTransactionOutput {
    pub actual_fee: Fee,
    pub messages_sent: Vec<MessageToL1>,
    pub events: Vec<Event>,
}

/// A transaction receipt.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct TransactionReceipt {
    pub transaction_hash: TransactionHash,
    pub block_hash: BlockHash,
    pub block_number: BlockNumber,
    #[serde(flatten)]
    pub output: TransactionOutput,
}

/// Transaction execution status.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord, Default)]
pub enum TransactionExecutionStatus {
    #[serde(rename = "SUCCEEDED")]
    #[default]
    // Succeeded is the default variant because old versions of Starknet don't have an execution
    // status and every transaction is considered succeeded
    Succeeded,
    #[serde(rename = "REVERTED")]
    Reverted,
}

/// A fee.
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[serde(from = "PrefixedBytesAsHex<16_usize>", into = "PrefixedBytesAsHex<16_usize>")]
pub struct Fee(pub u128);

impl From<PrefixedBytesAsHex<16_usize>> for Fee {
    fn from(val: PrefixedBytesAsHex<16_usize>) -> Self {
        Self(u128::from_be_bytes(val.0))
    }
}

impl From<Fee> for PrefixedBytesAsHex<16_usize> {
    fn from(fee: Fee) -> Self {
        Self(fee.0.to_be_bytes())
    }
}

impl From<Fee> for StarkFelt {
    fn from(fee: Fee) -> Self {
        Self::from(fee.0)
    }
}

/// A Tip.
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
#[serde(from = "PrefixedBytesAsHex<8_usize>", into = "PrefixedBytesAsHex<8_usize>")]
pub struct Tip(pub u64);

impl From<PrefixedBytesAsHex<8_usize>> for Tip {
    fn from(val: PrefixedBytesAsHex<8_usize>) -> Self {
        Self(u64::from_be_bytes(val.0))
    }
}

impl From<Tip> for PrefixedBytesAsHex<8_usize> {
    fn from(tip: Tip) -> Self {
        Self(tip.0.to_be_bytes())
    }
}

impl From<Tip> for StarkFelt {
    fn from(tip: Tip) -> Self {
        Self::from(tip.0)
    }
}

/// A Resourcs.
#[derive(Debug, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub enum Resource {
    L1Gas,
    L2Gas,
}

/// A ResourceBounds.
#[derive(Debug, Clone, Eq, Default, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct ResourceBounds {
    // Specifies the maximum amount of each resource allowed for usage during the execution.
    pub max_amount: u64,
    // Specifies the maximum price the user is willing to pay for each resource unit.
    pub max_price_per_unit: u128,
}

/// A ResourcesBounds.
#[derive(Debug, Clone, Default, Eq, Hash, PartialEq, Deserialize, Serialize, Ord, PartialOrd)]
pub struct ResourcesBounds(pub BTreeMap<Resource, ResourceBounds>);

/// The hash of a [Transaction](`crate::transaction::Transaction`).
#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct TransactionHash(pub StarkHash);

impl Display for TransactionHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A contract address salt.
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct ContractAddressSalt(pub StarkHash);

/// A transaction signature.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct TransactionSignature(pub Vec<StarkFelt>);

/// A transaction version.
#[derive(
    Debug, Copy, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct TransactionVersion(pub StarkFelt);

/// The calldata of a transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct Calldata(pub Arc<Vec<StarkFelt>>);

#[macro_export]
macro_rules! calldata {
    ( $( $x:expr ),* ) => {
        Calldata(vec![$($x),*].into())
    };
}

/// An L1 to L2 message.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct MessageToL2 {
    pub from_address: EthAddress,
    pub payload: L1ToL2Payload,
}

/// An L2 to L1 message.
#[derive(Debug, Default, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct MessageToL1 {
    pub from_address: ContractAddress,
    pub to_address: EthAddress,
    pub payload: L2ToL1Payload,
}

/// The payload of [`MessageToL2`].
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct L1ToL2Payload(pub Vec<StarkFelt>);

/// The payload of [`MessageToL1`].
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct L2ToL1Payload(pub Vec<StarkFelt>);

/// An event.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct Event {
    pub from_address: ContractAddress,
    #[serde(flatten)]
    pub content: EventContent,
}

/// An event content.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct EventContent {
    pub keys: Vec<EventKey>,
    pub data: EventData,
}

/// An event key.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct EventKey(pub StarkFelt);

/// An event data.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord)]
pub struct EventData(pub Vec<StarkFelt>);

/// The index of a transaction in [BlockBody](`crate::block::BlockBody`).
#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct TransactionOffsetInBlock(pub usize);

/// The index of an event in [TransactionOutput](`crate::transaction::TransactionOutput`).
#[derive(
    Debug, Default, Copy, Clone, Eq, PartialEq, Hash, Deserialize, Serialize, PartialOrd, Ord,
)]
pub struct EventIndexInTransactionOutput(pub usize);

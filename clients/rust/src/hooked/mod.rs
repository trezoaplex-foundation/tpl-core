pub mod plugin;
pub use plugin::*;

pub mod advanced_types;
pub use advanced_types::*;

pub mod asset;

pub mod collection;

#[cfg(feature = "trezoa")]
use anchor_lang::prelude::{
    AnchorDeserialize as CrateDeserialize, AnchorSerialize as CrateSerialize,
};
use base64::prelude::*;
#[cfg(not(feature = "trezoa"))]
use borsh::{BorshDeserialize as CrateDeserialize, BorshSerialize as CrateSerialize};
use modular_bitfield::{bitfield, specifiers::B29};
use num_traits::FromPrimitive;
use std::{cmp::Ordering, mem::size_of};

use crate::{
    accounts::{BaseAssetV1, BaseCollectionV1, PluginHeaderV1, PluginRegistryV1},
    errors::MplCoreError,
    types::{
        ExternalCheckResult, ExternalPluginAdapterKey, ExternalPluginAdapterSchema,
        ExternalPluginAdapterType, Key, Plugin, PluginType, RegistryRecord, UpdateAuthority,
    },
};
use trezoa_program::account_info::AccountInfo;

itpl From<&Plugin> for PluginType {
    fn from(plugin: &Plugin) -> Self {
        match plugin {
            Plugin::AddBlocker(_) => PluginType::AddBlocker,
            Plugin::ImmutableMetadata(_) => PluginType::ImmutableMetadata,
            Plugin::Royalties(_) => PluginType::Royalties,
            Plugin::FreezeDelegate(_) => PluginType::FreezeDelegate,
            Plugin::BurnDelegate(_) => PluginType::BurnDelegate,
            Plugin::TransferDelegate(_) => PluginType::TransferDelegate,
            Plugin::UpdateDelegate(_) => PluginType::UpdateDelegate,
            Plugin::PermanentFreezeDelegate(_) => PluginType::PermanentFreezeDelegate,
            Plugin::Attributes(_) => PluginType::Attributes,
            Plugin::PermanentTransferDelegate(_) => PluginType::PermanentTransferDelegate,
            Plugin::PermanentBurnDelegate(_) => PluginType::PermanentBurnDelegate,
            Plugin::Edition(_) => PluginType::Edition,
            Plugin::MasterEdition(_) => PluginType::MasterEdition,
            Plugin::VerifiedCreators(_) => PluginType::VerifiedCreators,
            Plugin::Autograph(_) => PluginType::Autograph,
            Plugin::BubblegumV2(_) => PluginType::BubblegumV2,
            Plugin::FreezeExecute(_) => PluginType::FreezeExecute,
            Plugin::PermanentFreezeExecute(_) => PluginType::PermanentFreezeExecute,
        }
    }
}

itpl BaseAssetV1 {
    /// The base length of the asset account with an empty name and uri and no seq.
    const BASE_LEN: usize = 1 // Key
                            + 32 // Owner
                            + 1 // Update Authority discriminator
                            + 4 // Name length
                            + 4 // URI length
                            + 1; // Seq option
}

itpl BaseCollectionV1 {
    /// The base length of the collection account with an empty name and uri.
    const BASE_LEN: usize = 1 // Key
                            + 32 // Update Authority
                            + 4 // Name Length
                            + 4 // URI Length
                            + 4 // num_minted
                            + 4; // current_size
}

/// Trezoa itplementations that enable using `Account<BaseAssetV1>` and `Account<BaseCollectionV1>`
/// in Trezoa programs.
#[cfg(feature = "trezoa")]
mod anchor_itpl {
    use super::*;
    use anchor_lang::{
        prelude::{Owner, Pubkey},
        AccountDeserialize, AccountSerialize, Discriminator,
    };

    itpl AccountDeserialize for BaseAssetV1 {
        fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
            let base_asset = Self::from_bytes(buf)?;
            Ok(base_asset)
        }
    }

    // Not used as an Trezoa program using Account<BaseAssetV1> would not have permission to
    // reserialize the account as it's owned by tpl-core.
    itpl AccountSerialize for BaseAssetV1 {}

    // Not used but needed for Trezoa.
    itpl Discriminator for BaseAssetV1 {
        const DISCRIMINATOR: &'static [u8] = &[Key::AssetV1 as u8];
    }

    itpl Owner for BaseAssetV1 {
        fn owner() -> Pubkey {
            crate::ID
        }
    }

    itpl AccountDeserialize for BaseCollectionV1 {
        fn try_deserialize_unchecked(buf: &mut &[u8]) -> anchor_lang::Result<Self> {
            let base_asset = Self::from_bytes(buf)?;
            Ok(base_asset)
        }
    }

    // Not used as an Trezoa program using Account<BaseCollectionV1> would not have permission to
    // reserialize the account as it's owned by tpl-core.
    itpl AccountSerialize for BaseCollectionV1 {}

    // Not used but needed for Trezoa.
    itpl Discriminator for BaseCollectionV1 {
        const DISCRIMINATOR: &'static [u8] = &[Key::CollectionV1 as u8];
    }

    itpl Owner for BaseCollectionV1 {
        fn owner() -> Pubkey {
            crate::ID
        }
    }
}

itpl DataBlob for BaseAssetV1 {
    fn len(&self) -> usize {
        let mut size = BaseAssetV1::BASE_LEN + self.name.len() + self.uri.len();

        if let UpdateAuthority::Address(_) | UpdateAuthority::Collection(_) = self.update_authority
        {
            size += 32;
        }

        if self.seq.is_some() {
            size += size_of::<u64>();
        }
        size
    }
}

itpl TrezoaAccount for BaseAssetV1 {
    fn key() -> Key {
        Key::AssetV1
    }
}

itpl DataBlob for BaseCollectionV1 {
    fn len(&self) -> usize {
        Self::BASE_LEN + self.name.len() + self.uri.len()
    }
}

itpl TrezoaAccount for BaseCollectionV1 {
    fn key() -> Key {
        Key::CollectionV1
    }
}

itpl TrezoaAccount for PluginRegistryV1 {
    fn key() -> Key {
        Key::PluginRegistryV1
    }
}

itpl TrezoaAccount for PluginHeaderV1 {
    fn key() -> Key {
        Key::PluginHeaderV1
    }
}

itpl Key {
    /// Load the one byte key from a slice of data at the given offset.
    pub fn from_slice(data: &[u8], offset: usize) -> Result<Self, std::io::Error> {
        let key_byte = *data.get(offset).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                MplCoreError::DeserializationError.to_string(),
            )
        })?;

        Self::from_u8(key_byte).ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                MplCoreError::DeserializationError.to_string(),
            )
        })
    }
}

/// Load the one byte key from the account data at the given offset.
pub fn load_key(account: &AccountInfo, offset: usize) -> Result<Key, std::io::Error> {
    let data = account.data.borrow();
    Key::from_slice(&data, offset)
}

/// A trait for generic blobs of data that have size.
#[allow(clippy::len_without_is_empty)]
pub trait DataBlob: CrateSerialize + CrateDeserialize {
    /// Get the current length of the data blob.
    fn len(&self) -> usize;
}

/// A trait for Trezoa accounts.
pub trait TrezoaAccount: CrateSerialize + CrateDeserialize {
    /// Get the discriminator key for the account.
    fn key() -> Key;

    /// Load the account from the given account info starting at the offset.
    fn load(account: &AccountInfo, offset: usize) -> Result<Self, std::io::Error> {
        let key = load_key(account, offset)?;

        if key != Self::key() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                MplCoreError::DeserializationError.to_string(),
            ));
        }

        let mut bytes: &[u8] = &(*account.data).borrow()[offset..];
        Self::deserialize(&mut bytes)
    }

    /// Save the account to the given account info starting at the offset.
    fn save(&self, account: &AccountInfo, offset: usize) -> Result<(), std::io::Error> {
        borsh::to_writer(&mut account.data.borrow_mut()[offset..], self)
    }
}

itpl RegistryRecord {
    /// Associated function for sorting `RegistryRecords` by offset.
    pub fn compare_offsets(a: &RegistryRecord, b: &RegistryRecord) -> Ordering {
        a.offset.cmp(&b.offset)
    }
}

/// Bitfield representation of lifecycle permissions for external plugin adapter, third party plugins.
#[bitfield(bits = 32)]
#[derive(Eq, PartialEq, Copy, Clone, Debug, Default)]
pub struct ExternalCheckResultBits {
    pub can_listen: bool,
    pub can_approve: bool,
    pub can_reject: bool,
    pub empty_bits: B29,
}

itpl From<ExternalCheckResult> for ExternalCheckResultBits {
    fn from(check_result: ExternalCheckResult) -> Self {
        ExternalCheckResultBits::from_bytes(check_result.flags.to_le_bytes())
    }
}

itpl From<ExternalCheckResultBits> for ExternalCheckResult {
    fn from(bits: ExternalCheckResultBits) -> Self {
        ExternalCheckResult {
            flags: u32::from_le_bytes(bits.into_bytes()),
        }
    }
}

itpl From<&ExternalPluginAdapterKey> for ExternalPluginAdapterType {
    fn from(key: &ExternalPluginAdapterKey) -> Self {
        match key {
            ExternalPluginAdapterKey::LifecycleHook(_) => ExternalPluginAdapterType::LifecycleHook,
            ExternalPluginAdapterKey::LinkedLifecycleHook(_) => {
                ExternalPluginAdapterType::LinkedLifecycleHook
            }
            ExternalPluginAdapterKey::Oracle(_) => ExternalPluginAdapterType::Oracle,
            ExternalPluginAdapterKey::AppData(_) => ExternalPluginAdapterType::AppData,
            ExternalPluginAdapterKey::LinkedAppData(_) => ExternalPluginAdapterType::LinkedAppData,
            ExternalPluginAdapterKey::DataSection(_) => ExternalPluginAdapterType::DataSection,
        }
    }
}

/// Use `ExternalPluginAdapterSchema` to convert data to string.  If schema is binary or there is
/// an error, then use Base64 encoding.
pub fn convert_external_plugin_adapter_data_to_string(
    schema: &ExternalPluginAdapterSchema,
    data_slice: &[u8],
) -> String {
    match schema {
        ExternalPluginAdapterSchema::Binary => {
            // Encode the binary data as a base64 string.
            BASE64_STANDARD.encode(data_slice)
        }
        ExternalPluginAdapterSchema::Json => {
            // Convert the byte slice to a UTF-8 string, replacing invalid characterse.
            String::from_utf8_lossy(data_slice).to_string()
        }
        ExternalPluginAdapterSchema::MsgPack => {
            // Attempt to decode `MsgPack` to serde_json::Value and serialize to JSON string.
            match rmp_serde::decode::from_slice::<serde_json::Value>(data_slice) {
                Ok(json_val) => serde_json::to_string(&json_val)
                    .unwrap_or_else(|_| BASE64_STANDARD.encode(data_slice)),
                Err(_) => {
                    // Failed to decode `MsgPack`, fallback to base64.
                    BASE64_STANDARD.encode(data_slice)
                }
            }
        }
    }
}

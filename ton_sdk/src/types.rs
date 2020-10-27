/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use num_traits::cast::ToPrimitive;
use std::fmt;
use ton_types::Result;

use crate::error::SdkError;

pub const MASTERCHAIN_ID: i32 = -1;

fn default_workchain() -> i32 {
    0
}

fn default_message_expiration_timeout() -> u32 {
    40000
}

fn default_message_expiration_timeout_grow_factor() -> f32 {
    1.5
}

#[derive(Deserialize, Debug, Clone, ApiType)]
pub struct SdkAbiConfig {
    #[serde(default = "default_workchain")]
    pub workchain: i32,
    #[serde(default = "default_message_expiration_timeout")]
    pub message_expiration_timeout: u32,
    #[serde(default = "default_message_expiration_timeout_grow_factor")]
    pub message_expiration_timeout_grow_factor: f32,
}

impl Default for SdkAbiConfig {
    fn default() -> Self {
        Self {
            workchain: default_workchain(),
            message_expiration_timeout: default_message_expiration_timeout(),
            message_expiration_timeout_grow_factor: default_message_expiration_timeout_grow_factor(
            ),
        }
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq)]
pub struct StringId(String);

pub type BlockId = StringId;

impl From<String> for StringId {
    fn from(id: String) -> Self {
        StringId { 0: id }
    }
}

impl From<&str> for StringId {
    fn from(id: &str) -> Self {
        StringId { 0: id.to_owned() }
    }
}

impl From<Vec<u8>> for StringId {
    fn from(id: Vec<u8>) -> Self {
        StringId { 0: hex::encode(id) }
    }
}

impl From<&[u8]> for StringId {
    fn from(id: &[u8]) -> Self {
        StringId { 0: hex::encode(id) }
    }
}

impl fmt::Display for StringId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl StringId {
    pub fn to_base64(&self) -> Result<String> {
        let bytes = self.to_bytes()?;
        Ok(base64::encode(&bytes))
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        hex::decode(&self.0).map_err(Into::into)
    }
}

pub fn grams_to_u64(grams: &ton_block::types::Grams) -> Result<u64> {
    grams.0.to_u64().ok_or(
        SdkError::InvalidData {
            msg: "Cannot convert grams value".to_owned(),
        }
        .into(),
    )
}

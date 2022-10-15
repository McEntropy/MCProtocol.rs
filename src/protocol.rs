use drax::{Maybe, SizedVec, VarInt};

pub mod chunk;
pub mod bit_storage;

pub const CURRENT_VERSION_IMPL: VarInt = 760;

#[derive(
    drax_derive::DraxTransport, serde_derive::Deserialize, serde_derive::Serialize, Debug, Clone,
)]
pub struct GameProfile {
    pub id: uuid::Uuid,
    pub name: String,
    pub properties: SizedVec<Property>,
}
#[derive(
    drax_derive::DraxTransport, serde_derive::Deserialize, serde_derive::Serialize, Debug, Clone,
)]
pub struct Property {
    pub name: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Maybe<String>,
}

pub mod handshaking {
    pub mod sb {
        use drax::VarInt;

        #[derive(
            drax_derive::DraxTransport, Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq,
        )]
        #[drax(key = {match VarInt})]
        pub enum NextState {
            Handshaking,
            Status,
            Login,
        }

        #[derive(drax_derive::DraxTransport, Debug, Clone)]
        pub struct Handshake {
            pub protocol_version: VarInt,
            #[drax(limit = 255)]
            pub server_address: String,
            pub server_port: u16,
            pub next_state: NextState,
        }

        use crate::registry::UNKNOWN_VERSION;

        crate::import_registrations! {
            Handshake {
                UNKNOWN_VERSION -> 0x00,
            }
        }
    }

    pub mod cb {}
}

pub mod status {
    pub mod sb {
        #[derive(drax_derive::DraxTransport, Debug, Clone, Copy)]
        pub struct Request;

        #[derive(drax_derive::DraxTransport, Debug, Clone, Copy)]
        pub struct Ping {
            pub start_time: i64,
        }

        impl From<super::cb::Pong> for Ping {
            fn from(pong: super::cb::Pong) -> Self {
                Self {
                    start_time: pong.start_time,
                }
            }
        }

        use super::super::CURRENT_VERSION_IMPL;

        crate::import_registrations! {
            Request {
                0..CURRENT_VERSION_IMPL -> 0x00,
            }
            Ping {
                0..CURRENT_VERSION_IMPL -> 0x01,
            }
        }
    }

    pub mod cb {
        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct StatusResponseVersion {
            pub name: String,
            pub protocol: i32,
        }

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct StatusResponsePlayerSample {
            pub name: String,
            pub id: uuid::Uuid,
        }

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct StatusResponsePlayers {
            pub max: i32,
            pub online: i32,
            pub sample: Vec<StatusResponsePlayerSample>,
        }

        #[derive(serde_derive::Serialize, serde_derive::Deserialize, Debug)]
        pub struct StatusResponse {
            pub version: StatusResponseVersion,
            pub players: StatusResponsePlayers,
            pub description: crate::chat::Chat,
            #[serde(skip_serializing_if = "Option::is_none")]
            pub favicon: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none", rename = "previewsChat")]
            pub previews_chat: Option<bool>,
        }

        #[derive(drax_derive::DraxTransport)]
        pub struct Response(#[drax(json = 32767)] pub StatusResponse);

        #[derive(drax_derive::DraxTransport, Debug, Clone, Copy)]
        pub struct Pong {
            pub start_time: i64,
        }

        impl From<super::sb::Ping> for Pong {
            fn from(ping: super::sb::Ping) -> Self {
                Self {
                    start_time: ping.start_time,
                }
            }
        }

        use super::super::CURRENT_VERSION_IMPL;

        crate::import_registrations! {
            Response {
                0..CURRENT_VERSION_IMPL -> 0x00,
            }
            Pong {
                0..CURRENT_VERSION_IMPL -> 0x01,
            }
        }
    }
}

pub mod login {
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    use drax::SizedVec;

    use crate::crypto::{key_from_der, MCPublicKey};

    #[derive(drax_derive::DraxTransport, Debug, Clone)]
    pub struct MojangIdentifiedKey {
        pub timestamp: u64,
        pub public_key: SizedVec<u8>,
        pub signature: SizedVec<u8>,
    }

    impl MojangIdentifiedKey {
        pub fn has_expired(&self) -> bool {
            let key_instant = UNIX_EPOCH + Duration::from_millis(self.timestamp as u64);
            SystemTime::now() > key_instant
        }

        pub fn verify_signature(
            &self,
            verify_against: &MCPublicKey,
        ) -> Result<(), crate::crypto::CapturedRsaError> {
            let encoded_pem = crate::crypto::encode_key_pem(self.timestamp, &self.public_key)
                .map_err(crate::crypto::CapturedRsaError::SpecificationError)?;
            crate::crypto::verify_signature(
                Some(crate::crypto::SHA1_HASH),
                verify_against,
                &self.signature,
                crate::crypto::sha1_message(encoded_pem.as_bytes()).as_slice(),
            )
            .map_err(crate::crypto::CapturedRsaError::RsaError)
        }
    }

    #[derive(Clone)]
    pub struct IdentifiedKey {
        public_key: MCPublicKey,
    }

    impl IdentifiedKey {
        pub fn new(key: &[u8]) -> Result<Self, crate::crypto::CapturedRsaError> {
            Ok(Self {
                public_key: key_from_der(key)?,
            })
        }

        pub fn verify_data_signature(
            &self,
            signature: &[u8],
            data: &[u8],
        ) -> rsa::errors::Result<()> {
            crate::crypto::verify_signature(
                Some(crate::crypto::SHA256_HASH),
                &self.public_key,
                signature,
                data,
            )
        }
    }

    pub mod sb {
        use drax::Maybe;
        use drax::ShortSizedVec;
        use drax::SizedVec;
        use drax::VarInt;
        use uuid::Uuid;

        use super::MojangIdentifiedKey;

        use crate::registry::ProtocolVersionKey;

        #[derive(drax_derive::DraxTransport, Debug)]
        #[drax(include = {ProtocolVersionKey as protocol_version})]
        pub struct LoginStart {
            #[drax(limit = 16)]
            pub name: String,
            #[drax(skip_if = {protocol_version < 759})]
            pub sig_data: Maybe<MojangIdentifiedKey>,
            #[drax(skip_if = {protocol_version < 759})]
            pub sig_holder: Maybe<Uuid>,
        }
        
        #[derive(drax_derive::DraxTransport, Debug)]
        #[drax(key = {match bool})]
        pub enum EncryptionResponseData {
            #[drax(key = {true})]
            VerifyTokenData(SizedVec<u8>),
            #[drax(key = {false})]
            MessageSignature {
                salt: i64,
                message_signature: SizedVec<u8>,
            },
        }

        #[derive(drax_derive::DraxTransport, Debug)]
        #[drax(include = {ProtocolVersionKey as protocol_version})]
        pub struct EncryptionResponse {
            pub shared_secret: SizedVec<u8>,
            #[drax(skip_if = {protocol_version < 759})]
            pub response_data: Option<EncryptionResponseData>,
            #[drax(skip_if = {protocol_version > 758 || protocol_version < 47})]
            pub verify_token: Option<SizedVec<u8>>,
            #[drax(skip_if = {protocol_version > 5})]
            pub legacy_shared_secret: Option<ShortSizedVec<u8>>,
            #[drax(skip_if = {protocol_version > 5})]
            pub legacy_verify_token: Option<ShortSizedVec<u8>>,
        }
        
        #[derive(drax_derive::DraxTransport, Debug)]
        pub struct LoginPluginResponse {
            pub message_id: VarInt,
            pub successful: bool,
            pub data: Vec<u8>,
        }

        use super::super::CURRENT_VERSION_IMPL;

        crate::import_registrations! {
            LoginStart {
                0..CURRENT_VERSION_IMPL -> 0x00,
            }
            EncryptionResponse {
                0..CURRENT_VERSION_IMPL -> 0x01,
            }
            LoginPluginResponse {
                382..CURRENT_VERSION_IMPL -> 0x02,
            }
        }
    }

    pub mod cb {
        use crate::protocol::{GameProfile, Property};
        use crate::registry::ProtocolVersionKey;
        use drax::{Maybe, SizedVec, ShortSizedVec};

        #[derive(drax_derive::DraxTransport, Debug)]
        #[drax(include = {ProtocolVersionKey as protocol_version})]
        pub struct Disconnect {
            #[drax(json = 262144, skip_if = {protocol_version < 47})]
            pub reason: Option<crate::chat::Chat>,
            #[drax(skip_if = {protocol_version > 5})]
            pub legacy_reason: Option<String>,
        }

        #[derive(drax_derive::DraxTransport, Debug)]
        #[drax(include = {ProtocolVersionKey as protocol_version})]
        pub struct EncryptionRequest {
            #[drax(limit = 20)]
            pub server_id: String,
            #[drax(skip_if = {protocol_version < 47})]
            pub public_key: Option<SizedVec<u8>>,
            #[drax(skip_if = {protocol_version < 47})]
            pub verify_token: Option<SizedVec<u8>>,
            #[drax(skip_if = {protocol_version > 5})]
            pub legacy_public_key: Option<ShortSizedVec<u8>>,
            #[drax(skip_if = {protocol_version > 5})]
            pub legacy_verify_token: Option<ShortSizedVec<u8>>,
        }

        #[derive(drax_derive::DraxTransport, Debug)]
        pub struct LoginProperty {
            pub name: String,
            pub value: String,
            pub signature: Maybe<String>,
        }

        impl From<&Property> for LoginProperty {
            fn from(property: &Property) -> Self {
                Self {
                    name: property.name.clone(),
                    value: property.value.clone(),
                    signature: property.signature.as_ref().cloned(),
                }
            }
        }

        #[derive(drax_derive::DraxTransport, Debug)]
        #[drax(include = {ProtocolVersionKey as protocol_version})]
        pub struct LoginSuccess {
            #[drax(skip_if = {protocol_version < 707})]
            pub uuid: Option<uuid::Uuid>,
            #[drax(limit = 16, skip_if = {protocol_version > 706})]
            pub legacy_uuid: Option<String>,
            #[drax(limit = 16)]
            pub username: String,
            #[drax(skip_if = {protocol_version < 759})]
            pub properties: Option<SizedVec<LoginProperty>>,
        }

        impl From<&GameProfile> for LoginSuccess {
            fn from(profile: &GameProfile) -> Self {
                Self {
                    uuid: Some(profile.id.clone()),
                    legacy_uuid: None,
                    username: profile.name.clone(),
                    properties: Some(profile.properties.iter().map(LoginProperty::from).collect()),
                }
            }
        }

        #[derive(drax_derive::DraxTransport, Debug)]
        pub struct SetCompression {
            pub threshold: drax::VarInt,
        }

        #[derive(drax_derive::DraxTransport, Debug)]
        pub struct LoginPluginRequest {
            pub message_id: drax::VarInt,
            pub channel: String,
            pub data: Vec<u8>,
        }

        use super::super::CURRENT_VERSION_IMPL;

        crate::import_registrations! {
            Disconnect {
                0..CURRENT_VERSION_IMPL -> 0x00,
            }
            EncryptionRequest {
                0..CURRENT_VERSION_IMPL -> 0x01,
            }
            LoginSuccess {
                759..CURRENT_VERSION_IMPL -> 0x02,
            }
            SetCompression {
                47..CURRENT_VERSION_IMPL -> 0x03,
            }
            LoginPluginRequest {
                382..CURRENT_VERSION_IMPL -> 0x04,
            }
        }
    }
}

pub mod play;

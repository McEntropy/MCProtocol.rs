use crate::shared_types::login::{LoginUsername, MCIdentifiedKey};
use mc_serializer::primitive::VarInt;

#[derive(mc_serializer_derive::MCSerde, Debug)]
pub struct LoginStart {
    pub name: LoginUsername,
    #[serial_if(protocol >= ProtocolVersion::V119)]
    pub sig_data: (bool, Option<MCIdentifiedKey>),
    #[serial_if(protocol >= ProtocolVersion::V1191)]
    pub sig_holder: (bool, uuid::Uuid),
}

#[derive(mc_serializer_derive::MCSerde, Debug)]
#[key(bool)]
pub enum EncryptionResponseData {
    #[key(true)]
    VerifyTokenData((VarInt, Vec<u8>)),
    #[key(false)]
    MessageSignature {
        salt: i64,
        message_signature: (VarInt, Vec<u8>),
    },
}

#[derive(mc_serializer_derive::MCSerde, Debug)]
pub struct EncryptionResponse {
    pub shared_secret: (VarInt, Vec<u8>),
    pub response_data: EncryptionResponseData,
}

#[derive(mc_serializer_derive::MCSerde, Debug)]
pub struct LoginPluginResponse {
    pub message_id: VarInt,
    pub successful: bool,
    pub data: Vec<u8>,
}

crate::create_mappings! {
    LoginStart {
        def 0x00;
    }

    EncryptionResponse {
        def 0x01;
    }

    LoginPluginResponse {
        def 0x02;
    }
}

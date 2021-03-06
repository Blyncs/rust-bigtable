use rustc_serialize::base64::{ToBase64, STANDARD};

use goauth::auth::{JwtClaims, Token};
use goauth::scopes::Scope;
use goauth::get_token_with_creds;
use goauth::credentials::Credentials;
use smpl_jwt::Jwt;

use error::BTErr;

pub fn encode_str(str: &str) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(str.as_bytes().to_base64(STANDARD).as_bytes());
    v
}

pub fn encode_vecu8(vec: &Vec<u8>) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(vec.to_base64(STANDARD).as_bytes());
    v
}

pub fn get_auth_token(c: &str, fp: bool) -> Result<Token, BTErr> {
    let credentials;
    if fp {
        credentials = Credentials::from_file(c)?;
    } else {
        credentials = Credentials::from_str(c)?;
    }
    let claims = JwtClaims::new(credentials.iss(),
                                Scope::CloudPlatform,
                                credentials.token_uri(),
                                None, Some(1));
    let jwt = Jwt::new(claims, credentials.rsa_key()?, None);
    Ok(get_token_with_creds(&jwt, &credentials)?)
}

pub fn row_key_from_str(str: &str) -> Vec<u8> {
    let mut row_key = Vec::new();
    row_key.extend_from_slice(str.as_bytes());
    row_key
}
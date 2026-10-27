use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TokenUse {
    Access,
    Refresh,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub token_use: TokenUse,
    pub auth_epoch: i32,
    pub sub: String,
    pub org_id: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub exp: usize,
    pub iat: usize,
}

pub fn encode_jwt(claims: &Claims, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    jsonwebtoken::encode(
        &jsonwebtoken::Header::default(),
        claims,
        &jsonwebtoken::EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &jsonwebtoken::Validation::default(),
    )?;
    Ok(token_data.claims)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_claims() -> Claims {
        Claims {
            token_use: TokenUse::Access,
            auth_epoch: 0,
            sub: "user-123".into(),
            org_id: "org-456".into(),
            role: "ADMIN".into(),
            permissions: vec!["asset:read".into(), "*".into()],
            exp: 2000000000,
            iat: 1000000000,
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let secret = "super-secret-key";
        let claims = make_claims();
        let token = encode_jwt(&claims, secret).expect("encode should succeed");
        let decoded = decode_jwt(&token, secret).expect("decode should succeed");
        assert_eq!(decoded.sub, claims.sub);
        assert_eq!(decoded.org_id, claims.org_id);
        assert_eq!(decoded.role, claims.role);
        assert_eq!(decoded.permissions, claims.permissions);
        assert_eq!(decoded.exp, claims.exp);
        assert_eq!(decoded.iat, claims.iat);
    }

    #[test]
    fn test_decode_with_wrong_secret_fails() {
        let claims = make_claims();
        let token = encode_jwt(&claims, "correct-secret").unwrap();
        let result = decode_jwt(&token, "wrong-secret");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_garbage_token_fails() {
        let result = decode_jwt("not-a-valid-jwt", "secret");
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod token_purpose_tests {
    use super::*;
    #[test]
    fn token_purpose_survives_signature_validation() {
        for purpose in [TokenUse::Access, TokenUse::Refresh] {
            let claims = Claims {
                token_use: purpose,
                auth_epoch: 4,
                sub: "user".into(),
                org_id: "org".into(),
                role: "VIEWER".into(),
                permissions: vec![],
                exp: 2000000000,
                iat: 1000000000,
            };
            let token = encode_jwt(&claims, "test secret").unwrap();
            let decoded = decode_jwt(&token, "test secret").unwrap();
            assert_eq!(decoded.token_use, purpose);
            assert_eq!(decoded.auth_epoch, 4);
        }
    }
    #[test]
    fn legacy_tokens_without_purpose_are_rejected() {
        let value = serde_json::json!({"sub":"user","org_id":"org","role":"ADMIN",
            "permissions":["*"],"exp":2000000000,"iat":1000000000});
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &value,
            &jsonwebtoken::EncodingKey::from_secret(b"test secret"),
        )
        .unwrap();
        assert!(decode_jwt(&token, "test secret").is_err());
    }
}

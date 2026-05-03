use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
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

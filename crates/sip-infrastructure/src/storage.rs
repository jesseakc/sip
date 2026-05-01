use anyhow::Result;

pub struct ObjectStorage;

impl ObjectStorage {
    pub fn new(
        _endpoint: &str,
        _bucket: &str,
        _access_key: &str,
        _secret_key: &str,
    ) -> Result<Self> {
        Ok(Self)
    }

    pub async fn upload(&self, _path: &str, _data: Vec<u8>) -> Result<()> {
        Ok(())
    }

    pub async fn download(&self, _path: &str) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }
}

use gill_db::user::CreateSSHKey as CreateSSHKeyEntity;

pub struct CreateSSHKey {
    pub name: String,
    pub key: String,
}

impl From<CreateSSHKey> for CreateSSHKeyEntity {
    fn from(val: CreateSSHKey) -> Self {
        CreateSSHKeyEntity {
            name: val.name,
            key: val.key,
        }
    }
}

pub struct RawSshkey {
    inner: String,
}

impl From<String> for RawSshkey {
    fn from(inner: String) -> Self {
        RawSshkey { inner }
    }
}

impl RawSshkey {
    pub fn key_parts(&self) -> (&str, &str) {
        let key = self.inner.trim();
        let mut parts = key.split(' ');
        let key_type = parts.next().expect("ssh key type");
        let key = parts.next().expect("ssh key");
        (key_type, key)
    }

    pub fn full_key(&self) -> &str {
        &self.inner
    }
}

use crate::GitRepository;

impl GitRepository {
    pub fn merge(&self, base: &str, compare: &str) -> anyhow::Result<()> {
        self.clone();
        Ok(())
    }
}

use async_trait::async_trait;
use deadpool::managed::RecycleResult;
use ldap3::{Ldap, LdapConnAsync, LdapConnSettings, LdapError};
use std::future::Future;

pub struct Manager(String, LdapConnSettings);
pub type Pool = deadpool::managed::Pool<Manager>;

/// LDAP Manager for the `deadpool` managed connection pool.
impl Manager {
    /// Creates a new manager with the given URL.
    /// URL can be anything that can go Into a String (e.g. String or &str)
    pub fn new<S: Into<String>>(ldap_url: S) -> Self {
        Self(ldap_url.into(), LdapConnSettings::new())
    }

    /// Set a custom LdapConnSettings object on the manager.
    /// Returns a copy of the Manager.
    pub fn with_connection_settings(mut self, settings: LdapConnSettings) -> Self {
        self.1 = settings;
        self
    }
}

#[async_trait]
impl deadpool::managed::Manager for Manager {
    type Type = Ldap;
    type Error = LdapError;

    fn create(&self) -> impl Future<Output = Result<Self::Type, Self::Error>> + Send {
        async move {
            let (conn, ldap) = LdapConnAsync::with_settings(self.1.clone(), &self.0).await?;
            //#[cfg(feature = "default")]
            ldap3::drive!(conn);
            /*
            #[cfg(feature = "rt-actix")]
            actix_rt::spawn(async move {
                if let Err(e) = conn.drive().await {
                    log::warn!("LDAP connection error: {:?}", e);
                }
            });
            */
            Ok(ldap)
        }
    }

    fn recycle(
        &self,
        conn: &mut Self::Type,
        _: &deadpool::managed::Metrics,
    ) -> impl Future<Output = RecycleResult<Self::Error>> + Send {
        async move {
            // Revert back to anonymous bind by binding with zero credentials
            conn.simple_bind("", "").await?;
            Ok(())
        }
    }
}

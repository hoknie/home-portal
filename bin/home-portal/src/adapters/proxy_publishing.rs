use portal_feature::FieldError;
use portal_model::Publication;
use portal_proxy::{CheckPublication, CurrentProxySettings};
use portal_services::Publishing;

pub struct ProxyPublishing {
    pub settings: CurrentProxySettings,
    pub check: CheckPublication,
}

impl ProxyPublishing {
    pub fn new(settings: CurrentProxySettings, check: CheckPublication) -> ProxyPublishing {
        ProxyPublishing { settings, check }
    }
}

impl Publishing for ProxyPublishing {
    fn https_port(&self) -> Option<u16> {
        self.settings
            .run()
            .ok()
            .filter(|settings| settings.enabled)
            .map(|settings| settings.https_port)
    }

    fn problems(&self, publication: &Publication) -> Vec<FieldError> {
        self.check.run(publication)
    }
}

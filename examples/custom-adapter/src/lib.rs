//! Example Acme Router adapter — detection only.

use parallax_adapter_sdk::{
    AdapterCapabilities, AdapterKind, AdapterManifest, AdapterMaturity, CapabilitySupport,
    DetectionConfidence, DetectionResult, ParallaxAdapter, ProjectContext,
};

pub struct AcmeRouterAdapter;

impl ParallaxAdapter for AcmeRouterAdapter {
    fn manifest(&self) -> AdapterManifest {
        let mut m = AdapterManifest::builtin(
            "example.acme.router",
            "Acme Router Framework Adapter",
            AdapterKind::Framework,
            AdapterMaturity::Experimental,
            &["typescript", "javascript"],
        );
        m.version = "0.1.0".into();
        m.priority = 84;
        m.owns = vec!["routes".into(), "middleware".into()];
        m.notes = "Example adapter — not registered in Atlas builtins".into();
        m
    }

    fn detect(&self, context: &ProjectContext) -> DetectionResult {
        if context.package_contains("acme-router") {
            DetectionResult::matched(DetectionConfidence::High)
                .evidence("package", "acme-router")
                .owns(&["routes", "middleware"])
        } else {
            DetectionResult::no_match()
        }
    }

    fn capabilities(&self) -> AdapterCapabilities {
        AdapterCapabilities::framework_http()
            .with_flag("codegen", CapabilitySupport::Unsupported)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn detects_package_name() {
        let mut ctx = ProjectContext::new(PathBuf::from("."));
        ctx.packages.push("acme-router".into());
        let d = AcmeRouterAdapter.detect(&ctx);
        assert!(d.matched);
    }
}

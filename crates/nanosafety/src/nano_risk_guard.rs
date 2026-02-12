pub trait NanoRiskGuard {
    fn nano_risk(&self) -> f32;           // 0.0 .. 1.0
    fn nano_risk_domain(&self) -> NanoRiskDomain; // BCI, Nanoswarm, NeuromorphAI, SmartCity
}

use focusa_spec135_client::{SemanticPairPortfolio, SemanticPairTruthState};

#[test]
fn decodes_shared_portfolio_fixture() {
    let raw = include_str!("../../fixtures/semantic-pair-portfolio.json");
    let portfolio: SemanticPairPortfolio = serde_json::from_str(raw).unwrap();
    assert_eq!(portfolio.state, SemanticPairTruthState::VerificationRequired);
    assert_eq!(portfolio.items[0].findings[0].verdict, "unknown");
}

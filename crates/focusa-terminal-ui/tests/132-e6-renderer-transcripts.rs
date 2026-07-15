use focusa_terminal_ui::{
    HybridRenderer, InstallCompletionSummary, InstallEvent, InstallPhase, InstallRendererMode,
    InstallState, VerificationScanOutcome,
};
use ratatui::{Terminal, backend::TestBackend, style::Color};

const REVIEWED_WIDTH: u16 = 120;
const REVIEWED_HEIGHT: u16 = 40;
const TRUECOLOR_TRANSCRIPT_SEED: u64 = 7_310_421;
const MONOCHROME_TRANSCRIPT_SEED: u64 = 7_310_422;

fn normalized_frame_text(content: &[ratatui::buffer::Cell], width: u16, height: u16) -> String {
    let width = width as usize;
    let mut lines = Vec::new();
    for row in content.chunks(width).take(height as usize) {
        let mut line = String::new();
        for cell in row {
            line.push_str(cell.symbol());
        }
        lines.push(line.trim_end().to_string());
    }
    lines.join("\n")
}

fn render_frame_text(
    terminal: &mut Terminal<TestBackend>,
    renderer: &mut HybridRenderer,
    state: &InstallState,
    mode: InstallRendererMode,
    width: u16,
    height: u16,
) -> String {
    terminal
        .draw(|frame| renderer.render(frame, state, mode))
        .unwrap();
    normalized_frame_text(terminal.backend().buffer().content(), width, height)
}

fn assert_no_rgb_or_indexed(content: &[ratatui::buffer::Cell]) {
    for (index, cell) in content.iter().enumerate() {
        assert!(
            !matches!(cell.fg, Color::Rgb(_, _, _) | Color::Indexed(_)),
            "cell {index} has non-reset RGB/ANSI foreground state {:?}",
            cell.fg
        );
        assert!(
            !matches!(cell.bg, Color::Rgb(_, _, _) | Color::Indexed(_)),
            "cell {index} has non-reset RGB/ANSI background state {:?}",
            cell.bg
        );
    }
}

#[test]
fn truecolor_transcript_moves_through_health_checks_and_finalize_then_finishes() {
    let mut terminal = Terminal::new(TestBackend::new(REVIEWED_WIDTH, REVIEWED_HEIGHT)).unwrap();
    let mut renderer = HybridRenderer::new(TRUECOLOR_TRANSCRIPT_SEED);
    let mut state = InstallState::default();

    for event in [
        InstallEvent::PhaseStarted {
            phase: InstallPhase::InitializeEnvironment,
            message: "prepare runtime scaffolding".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::InitializeEnvironment,
            detail: Some("environment ready".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::DetectSystem,
            message: "detect host architecture".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::DetectSystem,
            detail: Some("amd64 detected".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::ValidateLicense,
            message: "validate credentials".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::ValidateLicense,
            detail: Some("license checks passed".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::ResolveRelease,
            message: "resolve release metadata".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::ResolveRelease,
            detail: Some("metadata resolved".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::DownloadAssets,
            message: "download installer assets".into(),
        },
        InstallEvent::AssetStarted {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            total_bytes: Some(2048),
        },
        InstallEvent::AssetProgress {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            downloaded_bytes: 512,
            total_bytes: Some(2048),
        },
        InstallEvent::AssetProgress {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            downloaded_bytes: 2048,
            total_bytes: Some(2048),
        },
        InstallEvent::AssetFinished {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            downloaded_bytes: 2048,
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::DownloadAssets,
            detail: Some("download complete".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::VerifyIntegrity,
            message: "verify checksums and trust".into(),
        },
        InstallEvent::VerificationScan {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            outcome: VerificationScanOutcome::Active,
        },
        InstallEvent::VerificationScan {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            outcome: VerificationScanOutcome::Succeeded,
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::VerifyIntegrity,
            detail: Some("manifest matched".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::InstallBinaries,
            message: "install binaries".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::InstallBinaries,
            detail: Some("focusa binaries written".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::IntegratePi,
            message: "integrate Pi".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::IntegratePi,
            detail: Some("pi integration successful at /usr/local/bin/focusa-pi".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::RegisterService,
            message: "register service".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::RegisterService,
            detail: Some("service ready and active".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::PersistPath,
            message: "persist PATH updates".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::PersistPath,
            detail: Some("path persisted".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::RunHealthChecks,
            message: "run health checks".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::RunHealthChecks,
            detail: Some("health checks all green".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::Finalize,
            message: "finalize installation".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::Finalize,
            detail: Some("finalization complete".into()),
        },
        InstallEvent::InstallFinished {
            summary: InstallCompletionSummary::default(),
        },
    ] {
        state.apply_event(&event);
    }

    let frame = render_frame_text(
        &mut terminal,
        &mut renderer,
        &state,
        InstallRendererMode::TrueColorAnimated,
        REVIEWED_WIDTH,
        REVIEWED_HEIGHT,
    );
    println!("[truecolor transcript]\n{frame}");

    assert!(frame.contains("Continuity Core"));
    assert!(frame.contains("Matrix field · infrastructure platform"));
    assert!(frame.contains("asset: focusa-core-linux-x64.tar.gz · 2048 / 2048 bytes (100.0%)"));
    assert!(frame.contains("phase completion"));
    assert!(frame.contains("✓ Initialize environment"));
    assert!(frame.contains("✓ Verify checksums and trust"));
    assert!(frame.contains("✓ Integrate Pi"));
    assert!(frame.contains("✓ Register service"));
    assert!(frame.contains("✓ Run health checks"));
    assert!(frame.contains("✓ Finalize"));
    assert_eq!(
        state.verification_scan.as_ref().map(|scan| scan.outcome),
        Some(VerificationScanOutcome::Succeeded)
    );
    assert_eq!(state.phase_completion, 12.0 / 13.0);
}

#[test]
fn monochrome_transcript_shows_verify_failure_then_rollback_and_keeps_recovery() {
    let mut terminal = Terminal::new(TestBackend::new(REVIEWED_WIDTH, REVIEWED_HEIGHT)).unwrap();
    let mut renderer = HybridRenderer::new(MONOCHROME_TRANSCRIPT_SEED);
    let mut state = InstallState::default();

    for event in [
        InstallEvent::PhaseStarted {
            phase: InstallPhase::InitializeEnvironment,
            message: "prepare runtime scaffolding".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::InitializeEnvironment,
            detail: Some("environment ready".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::DetectSystem,
            message: "detect host architecture".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::DetectSystem,
            detail: Some("amd64 detected".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::ValidateLicense,
            message: "validate credentials".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::ValidateLicense,
            detail: Some("license checks passed".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::ResolveRelease,
            message: "resolve release metadata".into(),
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::ResolveRelease,
            detail: Some("metadata resolved".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::DownloadAssets,
            message: "download installer assets".into(),
        },
        InstallEvent::AssetStarted {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            total_bytes: Some(2048),
        },
        InstallEvent::AssetProgress {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            downloaded_bytes: 1024,
            total_bytes: Some(2048),
        },
        InstallEvent::AssetFinished {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            downloaded_bytes: 1024,
        },
        InstallEvent::PhaseSucceeded {
            phase: InstallPhase::DownloadAssets,
            detail: Some("partial download".into()),
        },
        InstallEvent::PhaseStarted {
            phase: InstallPhase::VerifyIntegrity,
            message: "verify checksums and trust".into(),
        },
        InstallEvent::VerificationScan {
            asset: "focusa-core-linux-x64.tar.gz".into(),
            outcome: VerificationScanOutcome::Failed,
        },
        InstallEvent::PhaseFailed {
            phase: InstallPhase::VerifyIntegrity,
            message: "durable checksum mismatch".into(),
            recovery_hint: Some("re-download the release from an official mirror".into()),
        },
    ] {
        state.apply_event(&event);
    }

    assert_eq!(state.failure.as_deref(), Some("durable checksum mismatch"));

    let pre_rollback = render_frame_text(
        &mut terminal,
        &mut renderer,
        &state,
        InstallRendererMode::MonochromeAnimated,
        REVIEWED_WIDTH,
        REVIEWED_HEIGHT,
    );
    println!("[monochrome transcript - failed pre-rollback]\n{pre_rollback}");

    state.apply_event(&InstallEvent::RollbackStarted {
        reason: "rollback after durable checksum mismatch".into(),
    });
    let rollback_active = render_frame_text(
        &mut terminal,
        &mut renderer,
        &state,
        InstallRendererMode::MonochromeAnimated,
        REVIEWED_WIDTH,
        REVIEWED_HEIGHT,
    );
    println!("[monochrome transcript - rollback started]\n{rollback_active}");

    state.apply_event(&InstallEvent::RollbackSucceeded);
    let rollback_succeeded = render_frame_text(
        &mut terminal,
        &mut renderer,
        &state,
        InstallRendererMode::MonochromeAnimated,
        REVIEWED_WIDTH,
        REVIEWED_HEIGHT,
    );
    println!("[monochrome transcript - rollback succeeded]\n{rollback_succeeded}");

    assert_no_rgb_or_indexed(terminal.backend().buffer().content());

    assert!(
        pre_rollback.contains("✓ Download assets")
            || pre_rollback.contains("✓ Initialize environment")
    );
    assert!(pre_rollback.contains("✗ Verify checksums and trust"));
    assert!(rollback_active.contains("↶ Rolling back safely"));
    assert!(rollback_succeeded.contains("✗ Installation failed"));
    assert!(rollback_succeeded.contains("✗ Verify checksums and trust"));
    assert!(!rollback_succeeded.contains("FOCUSA INSTALL COMPLETE"));
    assert!(state.phase_completion < 1.0);
    assert_eq!(
        state.recovery_hint.as_deref(),
        Some("re-download the release from an official mirror")
    );
}

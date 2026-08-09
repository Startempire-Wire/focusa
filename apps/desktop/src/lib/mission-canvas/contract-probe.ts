// CONTRACT-007 identity probe: the app's single package import path to the
// generated artifacts. The runtime test proves the package re-exports the
// SAME module instances (zero DTO copies).
export { validateMissionCanvasContract } from '../../../../../packages/mission-canvas-contracts/index';

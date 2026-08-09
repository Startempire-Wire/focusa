/**
 * CONTRACT-007 — ONE package import path for the generated Mission Canvas
 * artifacts (Desktop AND Pi).
 *
 * This package RE-EXPORTS the canonical generated artifacts from
 * `docs/contracts/spec135/mission-canvas-v1/typescript/`. It never copies DTO
 * definitions: the generated files remain the single source of truth and
 * both apps resolve the SAME module instances through this package.
 */

export * from '../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-client.generated';
export * from '../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-types.generated';
export * from '../../docs/contracts/spec135/mission-canvas-v1/typescript/mission-canvas-validators.generated';

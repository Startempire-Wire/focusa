import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const outDir = mkdtempSync(join(tmpdir(), "focusa-bloatgaurd-profile-runtime-test-"));
const tempRoots = new Set();

function makeTempDir(prefix) {
  const dir = mkdtempSync(join(tmpdir(), prefix));
  tempRoots.add(dir);
  return dir;
}

function writeSettings(cwd, settings) {
  const settingsDir = join(cwd, ".pi");
  mkdirSync(settingsDir, { recursive: true });
  writeFileSync(join(settingsDir, "settings.json"), `${JSON.stringify(settings, null, 2)}\n`, "utf8");
}

function withIsolatedRuntime({ cwd, home, env, run }) {
  const previousCwd = process.cwd();
  const previousEnv = { ...process.env };

  try {
    for (const key of Object.keys(process.env)) delete process.env[key];
    process.env.HOME = home;
    for (const [key, value] of Object.entries(env)) {
      process.env[key] = value;
    }
    process.chdir(cwd);
    return run();
  } finally {
    process.chdir(previousCwd);
    for (const key of Object.keys(process.env)) delete process.env[key];
    for (const [key, value] of Object.entries(previousEnv)) {
      process.env[key] = value;
    }
  }
}

const expectedProfiles = {
  daily_driver: {
    bloatgaurdProfile: "daily_driver",
    warnPct: 50,
    compactPct: 70,
    hardPct: 85,
    cooldownMs: 180_000,
    maxCompactionsPerHour: 8,
    externalizeThresholdBytes: 8_192,
    externalizeThresholdTokens: 800,
    microCompactEveryNTurns: 5,
  },
  neat_freak: {
    bloatgaurdProfile: "neat_freak",
    warnPct: 50,
    compactPct: 70,
    hardPct: 85,
    cooldownMs: 180_000,
    maxCompactionsPerHour: 8,
    externalizeThresholdBytes: 8_192,
    externalizeThresholdTokens: 800,
    microCompactEveryNTurns: 5,
  },
  beast_mode: {
    bloatgaurdProfile: "beast_mode",
    warnPct: 60,
    compactPct: 80,
    hardPct: 92,
    cooldownMs: 300_000,
    maxCompactionsPerHour: 5,
    externalizeThresholdBytes: 16_384,
    externalizeThresholdTokens: 1_600,
    microCompactEveryNTurns: 8,
  },
  speedy: {
    bloatgaurdProfile: "speedy",
    warnPct: 40,
    compactPct: 60,
    hardPct: 75,
    cooldownMs: 120_000,
    maxCompactionsPerHour: 12,
    externalizeThresholdBytes: 4_096,
    externalizeThresholdTokens: 400,
    microCompactEveryNTurns: 3,
  },
  tightwad: {
    bloatgaurdProfile: "tightwad",
    warnPct: 40,
    compactPct: 60,
    hardPct: 75,
    cooldownMs: 120_000,
    maxCompactionsPerHour: 12,
    externalizeThresholdBytes: 2_048,
    externalizeThresholdTokens: 200,
    microCompactEveryNTurns: 2,
  },
};

try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: projectDir, stdio: "pipe" }
  );

  const { loadConfig } = await import(pathToFileURL(join(outDir, "config.js")).href);

  for (const expected of Object.values(expectedProfiles)) {
    const cwd = makeTempDir("focusa-bloatgaurd-profile-cwd-");
    const home = makeTempDir("focusa-bloatgaurd-profile-home-");
    writeSettings(cwd, { focusa: { bloatgaurdProfile: expected.bloatgaurdProfile } });

    const { config, errors } = withIsolatedRuntime({
      cwd,
      home,
      env: {},
      run: () => loadConfig(cwd),
    });

    assert.equal(
      config.bloatgaurdProfile,
      expected.bloatgaurdProfile,
      `profile ${expected.bloatgaurdProfile} should be applied from settings`
    );
    assert.equal(config.warnPct, expected.warnPct);
    assert.equal(config.compactPct, expected.compactPct);
    assert.equal(config.hardPct, expected.hardPct);
    assert.equal(config.cooldownMs, expected.cooldownMs);
    assert.equal(config.maxCompactionsPerHour, expected.maxCompactionsPerHour);
    assert.equal(config.externalizeThresholdBytes, expected.externalizeThresholdBytes);
    assert.equal(config.externalizeThresholdTokens, expected.externalizeThresholdTokens);
    assert.equal(config.microCompactEveryNTurns, expected.microCompactEveryNTurns);
    assert.equal(errors.length, 0, `profile ${expected.bloatgaurdProfile} should not produce config errors`);
  }

  const explicitCwd = makeTempDir("focusa-bloatgaurd-explicit-");
  const explicitHome = makeTempDir("focusa-bloatgaurd-explicit-home-");
  writeSettings(explicitCwd, {
    focusa: { bloatgaurdProfile: "beast_mode" },
    extensions: {
      focusaPiBridge: {
        externalizeThresholdBytes: 7777,
        externalizeThresholdTokens: 777,
        microCompactEveryNTurns: 15,
      },
    },
  });
  const explicitResult = withIsolatedRuntime({
    cwd: explicitCwd,
    home: explicitHome,
    env: {},
    run: () => loadConfig(explicitCwd),
  });
  assert.equal(explicitResult.config.warnPct, expectedProfiles.beast_mode.warnPct);
  assert.equal(explicitResult.config.compactPct, expectedProfiles.beast_mode.compactPct);
  assert.equal(explicitResult.config.hardPct, expectedProfiles.beast_mode.hardPct);
  assert.equal(explicitResult.config.externalizeThresholdBytes, 7777);
  assert.equal(explicitResult.config.externalizeThresholdTokens, 777);
  assert.equal(explicitResult.config.microCompactEveryNTurns, 15);

  const settingsCwd = makeTempDir("focusa-bloatgaurd-settings-env-");
  const settingsHome = makeTempDir("focusa-bloatgaurd-settings-env-home-");
  writeSettings(settingsCwd, {
    focusa: { bloatgaurdProfile: "daily_driver" },
  });
  const envProfileResult = withIsolatedRuntime({
    cwd: settingsCwd,
    home: settingsHome,
    env: { FOCUSA_BLOATGAURD_PROFILE: "speedy" },
    run: () => loadConfig(settingsCwd),
  });
  assert.equal(envProfileResult.config.bloatgaurdProfile, "speedy");
  assert.equal(envProfileResult.config.warnPct, expectedProfiles.speedy.warnPct);
  assert.equal(
    envProfileResult.config.externalizeThresholdBytes,
    expectedProfiles.speedy.externalizeThresholdBytes
  );
  assert.equal(envProfileResult.errors.length, 0);

  const invalidCwd = makeTempDir("focusa-bloatgaurd-invalid-");
  const invalidHome = makeTempDir("focusa-bloatgaurd-invalid-home-");
  writeSettings(invalidCwd, { focusa: { bloatgaurdProfile: "invalid_profile" } });
  const invalidResult = withIsolatedRuntime({
    cwd: invalidCwd,
    home: invalidHome,
    env: {},
    run: () => loadConfig(invalidCwd),
  });
  assert.equal(invalidResult.config.bloatgaurdProfile, "daily_driver");
  assert.equal(invalidResult.config.warnPct, expectedProfiles.daily_driver.warnPct);
  assert.equal(invalidResult.config.compactPct, expectedProfiles.daily_driver.compactPct);
  assert.equal(invalidResult.config.hardPct, expectedProfiles.daily_driver.hardPct);
  assert.equal(invalidResult.errors.length, 1);
  assert.ok(invalidResult.errors[0].includes("Invalid bloatgaurdProfile"));

  const numericProfileCwd = makeTempDir("focusa-bloatgaurd-numeric-env-");
  const numericHome = makeTempDir("focusa-bloatgaurd-numeric-home-");
  writeSettings(numericProfileCwd, {
    focusa: { bloatgaurdProfile: "tightwad" },
    extensions: {
      focusaPiBridge: {
        externalizeThresholdBytes: 12_345,
        externalizeThresholdTokens: 900,
        microCompactEveryNTurns: 99,
      },
    },
  });
  const numericEnvResult = withIsolatedRuntime({
    cwd: numericProfileCwd,
    home: numericHome,
    env: {
      FOCUSA_BLOATGAURD_PROFILE: "speedy",
      FOCUSA_PI_EXTERNALIZE_BYTES: "4001",
      FOCUSA_PI_EXTERNALIZE_TOKENS: "412",
      FOCUSA_PI_MICRO_COMPACT_TURNS: "4",
    },
    run: () => loadConfig(numericProfileCwd),
  });

  assert.equal(numericEnvResult.config.bloatgaurdProfile, "speedy");
  assert.equal(numericEnvResult.config.warnPct, expectedProfiles.speedy.warnPct);
  assert.equal(numericEnvResult.config.externalizeThresholdBytes, 4001);
  assert.equal(numericEnvResult.config.externalizeThresholdTokens, 412);
  assert.equal(numericEnvResult.config.microCompactEveryNTurns, 4);

  console.log("spec101 bloatgaurd profile runtime config test passed");
} finally {
  rmSync(outDir, { recursive: true, force: true });
  for (const dir of tempRoots) {
    rmSync(dir, { recursive: true, force: true });
  }
}

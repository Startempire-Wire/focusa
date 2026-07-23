import { fetchJson } from '$lib/api';

export type MenubarUpdatePhase =
  | 'checking'
  | 'current'
  | 'available'
  | 'downloading'
  | 'installing'
  | 'blocked'
  | 'unavailable'
  | 'error';

export interface MenubarUpdateResult {
  phase: MenubarUpdatePhase;
  message: string;
  version?: string;
  downloadedBytes?: number;
  totalBytes?: number;
}

export type MenubarUpdateReporter = (result: MenubarUpdateResult) => void;

function report(reporter: MenubarUpdateReporter | undefined, result: MenubarUpdateResult) {
  reporter?.(result);
  window.dispatchEvent(new CustomEvent('focusa-menubar-update', { detail: result }));
  return result;
}

function isTauriRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

async function automaticApplyAllowed(): Promise<boolean> {
  const response = await fetchJson<any>('/v1/update/policy', 5000);
  const policy = response?.policy ?? {};
  return response?.status === 'completed'
    && response?.auto_apply_allowed === true
    && policy?.enabled !== false
    && policy?.parts?.menubar !== false;
}

export async function runMenubarUpdate(options: {
  install: boolean;
  automatic?: boolean;
  reporter?: MenubarUpdateReporter;
}): Promise<MenubarUpdateResult> {
  const { install, automatic = false, reporter } = options;
  if (!isTauriRuntime()) {
    return report(reporter, { phase: 'unavailable', message: 'Updater is available in the installed Focusa app.' });
  }

  if (automatic) {
    try {
      if (!(await automaticApplyAllowed())) {
        return report(reporter, { phase: 'blocked', message: 'Automatic update is disabled by Focusa policy.' });
      }
    } catch (error) {
      return report(reporter, {
        phase: 'blocked',
        message: `Automatic update policy unavailable: ${error instanceof Error ? error.message : String(error)}`,
      });
    }
  }

  report(reporter, { phase: 'checking', message: 'Checking signed Focusa updates…' });
  try {
    const [{ check }, { relaunch }] = await Promise.all([
      import('@tauri-apps/plugin-updater'),
      import('@tauri-apps/plugin-process'),
    ]);
    const update = await check();
    if (!update) {
      return report(reporter, { phase: 'current', message: 'Focusa is up to date.' });
    }

    const available = {
      phase: 'available' as const,
      message: `Focusa ${update.version} is signed and ready.`,
      version: update.version,
    };
    report(reporter, available);
    if (!install) return available;

    let downloadedBytes = 0;
    let totalBytes: number | undefined;
    await update.downloadAndInstall((event) => {
      if (event.event === 'Started') {
        totalBytes = event.data.contentLength ?? undefined;
        report(reporter, {
          phase: 'downloading',
          message: `Downloading signed Focusa ${update.version}…`,
          version: update.version,
          downloadedBytes,
          totalBytes,
        });
      } else if (event.event === 'Progress') {
        downloadedBytes += event.data.chunkLength;
        report(reporter, {
          phase: 'downloading',
          message: `Downloading signed Focusa ${update.version}…`,
          version: update.version,
          downloadedBytes,
          totalBytes,
        });
      } else if (event.event === 'Finished') {
        report(reporter, {
          phase: 'installing',
          message: `Installing verified Focusa ${update.version}; relaunching…`,
          version: update.version,
          downloadedBytes,
          totalBytes,
        });
      }
    });
    await relaunch();
    return report(reporter, {
      phase: 'installing',
      message: `Verified Focusa ${update.version} installed; relaunch requested.`,
      version: update.version,
    });
  } catch (error) {
    return report(reporter, {
      phase: 'error',
      message: `Signed update failed safely: ${error instanceof Error ? error.message : String(error)}`,
    });
  }
}

export function startAutomaticMenubarUpdate(reporter?: MenubarUpdateReporter): () => void {
  const timer = window.setTimeout(() => {
    void runMenubarUpdate({ install: true, automatic: true, reporter });
  }, 2500);
  return () => window.clearTimeout(timer);
}

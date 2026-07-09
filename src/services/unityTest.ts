import { ipcInvoke } from "./ipc";

import type { UnityTestSnapshot } from "../types";

export async function getUnityTestLatestSnapshot(): Promise<UnityTestSnapshot | null> {
  return ipcInvoke<UnityTestSnapshot | null>("unity_test_latest_snapshot");
}

import { useCallback, useEffect, useRef, useState } from "react";
import { api } from "@/services/api";
import { events } from "@/services/events";
import type { UpdateCheck, UpdateDownloadProgress } from "@/types";

export type UpdateStage =
  | "idle"
  | "checking"
  | "available"
  | "downloading"
  | "ready"
  | "error";

/**
 * Tracks the launcher-update lifecycle: check → download (progress) → install.
 * Progress/finish events come from the Rust backend.
 */
export function useUpdater() {
  const [stage, setStage] = useState<UpdateStage>("idle");
  const [check, setCheck] = useState<UpdateCheck | null>(null);
  const [progress, setProgress] = useState<UpdateDownloadProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const filePathRef = useRef<string | null>(null);
  const mounted = useRef(true);

  useEffect(() => {
    mounted.current = true;
    const unlisteners: Promise<() => void>[] = [
      events.onUpdateDownloadProgress((p) => {
        if (mounted.current) setProgress(p);
      }),
      events.onUpdateDownloadDone((path) => {
        filePathRef.current = path;
        if (mounted.current) {
          setProgress(null);
          setStage("ready");
        }
      }),
      events.onUpdateDownloadError((message) => {
        if (mounted.current) {
          setStage("error");
          setError(message);
        }
      }),
      events.onUpdateDownloadCancelled(() => {
        if (mounted.current) {
          setStage("available");
          setProgress(null);
        }
      }),
    ];
    return () => {
      mounted.current = false;
      unlisteners.forEach((p) => void p.then((fn) => fn()));
    };
  }, []);

  const checkForUpdates = useCallback(async () => {
    setStage("checking");
    setError(null);
    try {
      const result = await api.checkForUpdates();
      setCheck(result);
      setStage(result.updateAvailable ? "available" : "idle");
      return result;
    } catch (e) {
      if (mounted.current) {
        setStage("error");
        setError(String(e));
      }
      return null;
    }
  }, []);

  const startDownload = useCallback(async () => {
    setProgress({ downloaded: 0, total: 0, percent: 0 });
    setStage("downloading");
    try {
      await api.downloadUpdate();
    } catch (e) {
      setStage("error");
      setError(String(e));
    }
  }, []);

  const cancelDownload = useCallback(async () => {
    await api.cancelUpdateDownload();
  }, []);

  const install = useCallback(async () => {
    const path = filePathRef.current;
    if (path) {
      await api.installUpdate(path);
    }
  }, []);

  const reset = useCallback(() => {
    setStage("idle");
    setProgress(null);
    setError(null);
  }, []);

  return {
    stage,
    check,
    progress,
    error,
    checkForUpdates,
    startDownload,
    cancelDownload,
    install,
    reset,
  };
}

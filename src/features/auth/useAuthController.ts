import { useEffect, useRef, useState } from "react";
import { toast } from "sonner";
import { commands, type SessionInfo } from "@/bindings";

import { formatAuthError } from "./lib/formatError";

const DEFAULT_SERVER_URL = "http://127.0.0.1:1426";
const STORAGE_KEYS = {
  organizationCode: "medxz.organizationCode",
  serverUrl: "medxz.serverUrl",
} as const;

export type LoginValues = {
  email: string;
  password: string;
};

export type AuthController = {
  defaultServerUrl: string;
  serverUrl: string;
  setServerUrl: (next: string) => void;
  organizationCode: string;
  setOrganizationCode: (next: string) => void;
  organizationIsPersisted: boolean;
  session: SessionInfo | null;
  checkingSession: boolean;
  submitting: boolean;
  error: string | null;
  login: (values: LoginValues) => Promise<boolean>;
  logout: () => Promise<void>;
  changeOrg: () => void;
};

export function useAuthController(): AuthController {
  const [serverUrl, setServerUrl] = useState(() => {
    const stored = localStorage.getItem(STORAGE_KEYS.serverUrl);
    return stored?.trim() ? stored : DEFAULT_SERVER_URL;
  });
  const [organizationCode, setOrganizationCode] = useState(
    () => localStorage.getItem(STORAGE_KEYS.organizationCode) ?? "",
  );
  const [organizationIsPersisted, setOrganizationIsPersisted] = useState(() => {
    const stored = localStorage.getItem(STORAGE_KEYS.organizationCode);
    return Boolean(stored?.trim());
  });
  const [session, setSession] = useState<SessionInfo | null>(null);
  const [checkingSession, setCheckingSession] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const initialCheck = useRef(true);

  useEffect(() => {
    let cancelled = false;
    const delayMs = initialCheck.current ? 0 : 400;
    const timeoutId = window.setTimeout(async () => {
      setCheckingSession(true);
      setError(null);

      try {
        const result = await commands.me(serverUrl.trim());
        if (cancelled) return;

        if (result.status === "ok") {
          setSession(result.data);
        } else {
          setSession(null);
          const errorMessage = formatAuthError(result.error, "session");
          setError(errorMessage);
          if (initialCheck.current) {
            toast.error("Connection failed", {
              description: errorMessage,
            });
          }
        }
      } catch (e) {
        if (cancelled) return;
        setSession(null);
        const errorMessage = e instanceof Error ? e.message : "An unexpected error occurred";
        setError(errorMessage);
        if (initialCheck.current) {
          toast.error("Connection failed", {
            description: errorMessage,
          });
        }
      }

      if (!cancelled) setCheckingSession(false);
      initialCheck.current = false;
    }, delayMs);

    return () => {
      cancelled = true;
      window.clearTimeout(timeoutId);
    };
  }, [serverUrl]);

  async function login(values: LoginValues): Promise<boolean> {
    const trimmedServerUrl = serverUrl.trim();
    const trimmedOrganizationCode = organizationCode.trim();
    const trimmedEmail = values.email.trim();

    setSubmitting(true);
    setError(null);

    try {
      const result = await commands.login(
        trimmedServerUrl,
        trimmedOrganizationCode,
        trimmedEmail,
        values.password,
      );
      if (result.status === "ok") {
        localStorage.setItem(STORAGE_KEYS.organizationCode, trimmedOrganizationCode);
        localStorage.setItem(STORAGE_KEYS.serverUrl, trimmedServerUrl);
        setOrganizationIsPersisted(true);
        setOrganizationCode(trimmedOrganizationCode);
        setServerUrl(trimmedServerUrl);
        setSession(result.data);
        toast.success("Signed in", {
          description: `Welcome back, ${result.data.user.email}`,
        });
        return true;
      }

      const errorMessage = formatAuthError(result.error, "login");
      setError(errorMessage);
      toast.error("Sign in failed", {
        description: errorMessage,
      });
      return false;
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : "An unexpected error occurred";
      setError(errorMessage);
      toast.error("Sign in failed", {
        description: errorMessage,
      });
      return false;
    } finally {
      setSubmitting(false);
    }
  }

  async function logout() {
    const trimmedServerUrl = serverUrl.trim();
    setSubmitting(true);
    setError(null);

    try {
      const result = await commands.logout(trimmedServerUrl);
      if (result.status === "ok") {
        setSession(null);
        toast.success("Signed out");
        return;
      }

      const errorMessage = formatAuthError(result.error, "logout");
      setError(errorMessage);
      toast.error("Sign out failed", {
        description: errorMessage,
      });
    } catch (e) {
      const errorMessage = e instanceof Error ? e.message : "An unexpected error occurred";
      setError(errorMessage);
      toast.error("Sign out failed", {
        description: errorMessage,
      });
    } finally {
      setSubmitting(false);
    }
  }

  function changeOrg() {
    setOrganizationCode("");
    localStorage.removeItem(STORAGE_KEYS.organizationCode);
    setOrganizationIsPersisted(false);
    setSession(null);
    setError(null);
  }

  return {
    defaultServerUrl: DEFAULT_SERVER_URL,
    serverUrl,
    setServerUrl,
    organizationCode,
    setOrganizationCode,
    organizationIsPersisted,
    session,
    checkingSession,
    submitting,
    error,
    login,
    logout,
    changeOrg,
  };
}

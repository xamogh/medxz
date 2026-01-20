import "./App.css";
import { type ReactNode, useEffect, useMemo, useState } from "react";
import { type AppError, commands, type SessionInfo } from "./bindings";

function App() {
  const defaultServerUrl = "http://127.0.0.1:1426";
  const storageKeys = useMemo(
    () => ({
      organizationCode: "medxz.organizationCode",
      serverUrl: "medxz.serverUrl",
    }),
    [],
  );

  const [serverUrl, setServerUrl] = useState(() => {
    const stored = localStorage.getItem(storageKeys.serverUrl);
    return stored?.trim() ? stored : defaultServerUrl;
  });
  const [organizationCode, setOrganizationCode] = useState(
    () => localStorage.getItem(storageKeys.organizationCode) ?? "",
  );

  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showAdvanced, setShowAdvanced] = useState(false);

  const [session, setSession] = useState<SessionInfo | null>(null);
  const [checkingSession, setCheckingSession] = useState(true);
  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    (async () => {
      setCheckingSession(true);
      setError(null);
      const result = await commands.me(serverUrl);
      if (cancelled) return;
      if (result.status === "ok") {
        setSession(result.data);
      } else {
        setSession(null);
        setError(formatError(result.error));
      }
      setCheckingSession(false);
    })();
    return () => {
      cancelled = true;
    };
  }, [serverUrl]);

  const orgIsConfigured = organizationCode.trim().length > 0;
  const canSubmit =
    !checkingSession &&
    !submitting &&
    serverUrl.trim().length > 0 &&
    organizationCode.trim().length > 0 &&
    email.trim().length > 0 &&
    password.length > 0;

  async function onLogin() {
    setSubmitting(true);
    setError(null);
    const result = await commands.login(
      serverUrl.trim(),
      organizationCode.trim(),
      email.trim(),
      password,
    );
    if (result.status === "ok") {
      localStorage.setItem(storageKeys.organizationCode, organizationCode.trim());
      localStorage.setItem(storageKeys.serverUrl, serverUrl.trim());
      setSession(result.data);
      setPassword("");
    } else {
      setError(formatError(result.error));
    }
    setSubmitting(false);
  }

  async function onLogout() {
    setSubmitting(true);
    setError(null);
    const result = await commands.logout(serverUrl);
    if (result.status === "ok") {
      setSession(null);
    } else {
      setError(formatError(result.error));
    }
    setSubmitting(false);
  }

  function onChangeOrg() {
    setOrganizationCode("");
    localStorage.removeItem(storageKeys.organizationCode);
    setSession(null);
    setShowAdvanced(true);
  }

  return (
    <main className="shell">
      <section className="card" aria-busy={checkingSession || submitting}>
        <header className="head">
          <div>
            <div className="kicker">medxz</div>
            <h1>{session ? "Welcome" : "Sign in"}</h1>
          </div>
          <div className="meta">
            {orgIsConfigured ? (
              <button type="button" className="link" onClick={onChangeOrg}>
                Org: <span className="mono">{organizationCode.trim()}</span> · Change
              </button>
            ) : (
              <span className="hint">Set org once, then just sign in.</span>
            )}
          </div>
        </header>

        {error ? (
          <div className="error" role="alert">
            {error}
          </div>
        ) : null}

        {checkingSession ? (
          <div className="loading">Checking session…</div>
        ) : session ? (
          <div className="stack">
            <div className="panel">
              <div className="rowline">
                <span className="label">Organization</span>
                <span className="value">
                  {session.organization.name}{" "}
                  <span className="mono faint">({session.organization.code})</span>
                </span>
              </div>
              <div className="rowline">
                <span className="label">User</span>
                <span className="value">
                  {session.user.email} <span className="pill">{session.user.role}</span>
                </span>
              </div>
            </div>

            <button type="button" className="btn danger" onClick={onLogout} disabled={submitting}>
              Sign out
            </button>
          </div>
        ) : (
          <form
            className="stack"
            onSubmit={(e) => {
              e.preventDefault();
              if (canSubmit) onLogin();
            }}
          >
            {!orgIsConfigured ? (
              <Field
                id="org-code"
                label="Org code"
                tip="Provided by your admin. Example: acme-dental"
              >
                <input
                  id="org-code"
                  value={organizationCode}
                  onChange={(e) => setOrganizationCode(e.currentTarget.value)}
                  inputMode="text"
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
                  placeholder="e.g. acme-dental"
                />
              </Field>
            ) : null}

            <Field id="email" label="Email" tip="Your org login email.">
              <input
                id="email"
                value={email}
                onChange={(e) => setEmail(e.currentTarget.value)}
                inputMode="email"
                autoCapitalize="none"
                autoCorrect="off"
                spellCheck={false}
                autoComplete="username"
                placeholder="name@org.com"
              />
            </Field>

            <Field id="password" label="Password" tip="Case-sensitive.">
              <input
                id="password"
                value={password}
                onChange={(e) => setPassword(e.currentTarget.value)}
                type="password"
                autoComplete="current-password"
                placeholder="••••••••"
              />
            </Field>

            <div className="advanced">
              <button type="button" className="link" onClick={() => setShowAdvanced((v) => !v)}>
                {showAdvanced ? "Hide advanced" : "Advanced"}
              </button>
            </div>

            {showAdvanced ? (
              <Field
                id="server-url"
                label="Server URL"
                tip="Only change if your server address is different."
              >
                <input
                  id="server-url"
                  value={serverUrl}
                  onChange={(e) => setServerUrl(e.currentTarget.value)}
                  inputMode="url"
                  autoCapitalize="none"
                  autoCorrect="off"
                  spellCheck={false}
                  placeholder={defaultServerUrl}
                />
              </Field>
            ) : null}

            <button type="submit" className="btn" disabled={!canSubmit}>
              {submitting ? "Signing in…" : "Sign in"}
            </button>
          </form>
        )}
      </section>
    </main>
  );
}

export default App;

function Field(props: { id: string; label: string; tip: string; children: ReactNode }) {
  return (
    <div className="field">
      <div className="fieldHead">
        <label htmlFor={props.id}>{props.label}</label>
        <Tip text={props.tip} />
      </div>
      {props.children}
    </div>
  );
}

function Tip(props: { text: string }) {
  return (
    <button type="button" className="tip" data-tip={props.text} aria-label={props.text}>
      ?
    </button>
  );
}

function formatError(err: AppError): string {
  switch (err.type) {
    case "EmptyName":
      return "Name cannot be empty.";
    case "InvalidServerUrl":
    case "Network":
    case "Keychain":
      return err.details.message;
    case "ServerError":
      return `${err.details.message} (HTTP ${err.details.status}, ${err.details.code})`;
  }
}

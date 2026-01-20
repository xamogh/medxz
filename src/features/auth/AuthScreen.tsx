import { useState } from "react";
import { AuthHeader } from "./components/AuthHeader";
import { LoginForm, type LoginFormValues } from "./components/LoginForm";
import type { AuthController } from "./useAuthController";

export function AuthScreen(props: { auth: AuthController }) {
  const [showAdvanced, setShowAdvanced] = useState(false);

  const orgIsConfigured =
    props.auth.organizationIsPersisted && props.auth.organizationCode.trim().length > 0;

  return (
    <div aria-busy={props.auth.checkingSession || props.auth.submitting} className="grid gap-6">
      <AuthHeader
        title="Sign in"
        orgIsConfigured={orgIsConfigured}
        organizationCode={props.auth.organizationCode.trim()}
        onChangeOrg={() => {
          props.auth.changeOrg();
          setShowAdvanced(true);
        }}
      />

      {props.auth.error ? (
        <p className="text-sm text-destructive" role="alert">
          {props.auth.error}
        </p>
      ) : null}

      {props.auth.checkingSession ? (
        <p className="text-sm text-muted-foreground">Checking session…</p>
      ) : (
        <LoginForm
          defaultServerUrl={props.auth.defaultServerUrl}
          serverUrl={props.auth.serverUrl}
          onChangeServerUrl={props.auth.setServerUrl}
          showAdvanced={showAdvanced}
          onToggleAdvanced={() => setShowAdvanced((v) => !v)}
          organizationCode={props.auth.organizationCode}
          onChangeOrganizationCode={props.auth.setOrganizationCode}
          showOrganizationField={!orgIsConfigured}
          submitting={props.auth.submitting}
          checkingSession={props.auth.checkingSession}
          onSubmit={(values: LoginFormValues) => props.auth.login(values)}
        />
      )}
    </div>
  );
}

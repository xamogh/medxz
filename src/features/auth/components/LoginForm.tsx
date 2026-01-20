import { useState } from "react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

export type LoginFormValues = {
  email: string;
  password: string;
};

function EyeIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <title>Show password</title>
      <path d="M2.062 12.348a1 1 0 0 1 0-.696 10.75 10.75 0 0 1 19.876 0 1 1 0 0 1 0 .696 10.75 10.75 0 0 1-19.876 0" />
      <circle cx="12" cy="12" r="3" />
    </svg>
  );
}

function EyeOffIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <title>Hide password</title>
      <path d="M10.733 5.076a10.744 10.744 0 0 1 11.205 6.575 1 1 0 0 1 0 .696 10.747 10.747 0 0 1-1.444 2.49" />
      <path d="M14.084 14.158a3 3 0 0 1-4.242-4.242" />
      <path d="M17.479 17.499a10.75 10.75 0 0 1-15.417-5.151 1 1 0 0 1 0-.696 10.75 10.75 0 0 1 4.446-5.143" />
      <path d="m2 2 20 20" />
    </svg>
  );
}

function LoaderIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <title>Loading</title>
      <path d="M21 12a9 9 0 1 1-6.219-8.56" />
    </svg>
  );
}

export function LoginForm(props: {
  defaultServerUrl: string;
  serverUrl: string;
  onChangeServerUrl: (next: string) => void;
  showAdvanced: boolean;
  onToggleAdvanced: () => void;
  organizationCode: string;
  onChangeOrganizationCode: (next: string) => void;
  showOrganizationField: boolean;
  submitting: boolean;
  checkingSession: boolean;
  onSubmit: (values: LoginFormValues) => Promise<boolean>;
}) {
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [showPassword, setShowPassword] = useState(false);

  const canSubmit =
    !props.checkingSession &&
    !props.submitting &&
    props.serverUrl.trim().length > 0 &&
    props.organizationCode.trim().length > 0 &&
    email.trim().length > 0 &&
    password.length > 0;

  return (
    <form
      className="grid gap-6"
      onSubmit={async (e) => {
        e.preventDefault();
        if (!canSubmit) return;
        try {
          const ok = await props.onSubmit({ email: email.trim(), password });
          if (ok) setPassword("");
        } catch {
          // Error handled by parent
        }
      }}
    >
      <div className="grid gap-4">
        {props.showOrganizationField ? (
          <div className="grid gap-2">
            <Label htmlFor="org-code">Organization</Label>
            <Input
              id="org-code"
              value={props.organizationCode}
              onChange={(e) => props.onChangeOrganizationCode(e.currentTarget.value)}
              inputMode="text"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
              placeholder="acme-dental"
              disabled={props.submitting}
            />
          </div>
        ) : null}

        <div className="grid gap-2">
          <Label htmlFor="email">Email</Label>
          <Input
            id="email"
            type="email"
            value={email}
            onChange={(e) => setEmail(e.currentTarget.value)}
            autoCapitalize="none"
            autoCorrect="off"
            spellCheck={false}
            autoComplete="username"
            placeholder="name@example.com"
            disabled={props.submitting}
          />
        </div>

        <div className="grid gap-2">
          <div className="flex items-center justify-between">
            <Label htmlFor="password">Password</Label>
            <Button
              type="button"
              variant="link"
              size="sm"
              className="h-auto p-0 text-xs text-muted-foreground"
              onClick={props.onToggleAdvanced}
            >
              {props.showAdvanced ? "Hide options" : "More options"}
            </Button>
          </div>
          <div className="relative">
            <Input
              id="password"
              value={password}
              onChange={(e) => setPassword(e.currentTarget.value)}
              type={showPassword ? "text" : "password"}
              autoComplete="current-password"
              disabled={props.submitting}
              className="pr-10"
            />
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-0 top-0 h-full px-3 hover:bg-transparent"
              onClick={() => setShowPassword(!showPassword)}
              tabIndex={-1}
            >
              {showPassword ? (
                <EyeOffIcon className="h-4 w-4 text-muted-foreground" />
              ) : (
                <EyeIcon className="h-4 w-4 text-muted-foreground" />
              )}
              <span className="sr-only">{showPassword ? "Hide password" : "Show password"}</span>
            </Button>
          </div>
        </div>

        {props.showAdvanced ? (
          <div className="grid gap-2">
            <Label htmlFor="server-url">Server URL</Label>
            <Input
              id="server-url"
              value={props.serverUrl}
              onChange={(e) => props.onChangeServerUrl(e.currentTarget.value)}
              inputMode="url"
              autoCapitalize="none"
              autoCorrect="off"
              spellCheck={false}
              placeholder={props.defaultServerUrl}
              disabled={props.submitting}
            />
          </div>
        ) : null}
      </div>

      <Button type="submit" disabled={!canSubmit} className="w-full">
        {props.submitting ? (
          <>
            <LoaderIcon className="h-4 w-4 animate-spin" />
            Signing in…
          </>
        ) : (
          "Sign in"
        )}
      </Button>
    </form>
  );
}

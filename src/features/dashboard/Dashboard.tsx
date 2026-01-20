import type { SessionInfo } from "@/bindings";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { FrontDeskDashboard } from "@/features/frontDesk/FrontDeskDashboard";

function PlusIcon(props: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={props.className}
    >
      <title>medxz</title>
      <path d="M12 5v14M5 12h14" />
    </svg>
  );
}

export function Dashboard(props: {
  session: SessionInfo;
  serverUrl: string;
  onLogout: () => void;
  signingOut: boolean;
}) {
  const role = props.session.user.role;

  return (
    <div className="min-h-screen bg-background">
      <header className="sticky top-0 z-10 border-b bg-background/80 backdrop-blur supports-[backdrop-filter]:bg-background/60">
        <div className="mx-auto flex h-14 max-w-6xl items-center justify-between px-6">
          <div className="flex items-center gap-3">
            <div className="flex h-8 w-8 items-center justify-center bg-primary text-primary-foreground">
              <PlusIcon className="h-5 w-5" />
            </div>
            <div className="leading-tight">
              <div className="text-sm font-semibold">medxz</div>
              <div className="text-xs text-muted-foreground">
                {props.session.organization.name}{" "}
                <span className="font-mono">({props.session.organization.code})</span>
              </div>
            </div>
          </div>

          <div className="flex items-center gap-3">
            <div className="hidden text-right sm:block">
              <div className="text-sm">{props.session.user.email}</div>
              <div className="text-xs text-muted-foreground">{props.serverUrl}</div>
            </div>
            <Badge variant="secondary" className="hidden sm:inline-flex">
              {role}
            </Badge>
            <Button variant="outline" onClick={props.onLogout} disabled={props.signingOut}>
              Sign out
            </Button>
          </div>
        </div>
      </header>

      <main className="mx-auto max-w-6xl px-6 py-8">
        {role === "front_desk" ? (
          <FrontDeskDashboard
            key={props.session.organization.id}
            organizationId={props.session.organization.id}
          />
        ) : (
          <Card>
            <CardHeader>
              <CardTitle>Dashboard</CardTitle>
            </CardHeader>
            <CardContent className="text-sm text-muted-foreground">
              This role is not wired up yet. Sign in with a front desk account to access patient
              registration and search.
            </CardContent>
          </Card>
        )}
      </main>
    </div>
  );
}

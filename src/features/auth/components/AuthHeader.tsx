import { Button } from "@/components/ui/button";

export function AuthHeader(props: {
  title: string;
  orgIsConfigured: boolean;
  organizationCode: string;
  onChangeOrg: () => void;
}) {
  return (
    <div className="grid gap-2">
      <div className="flex items-center gap-2 lg:hidden">
        <div className="flex h-8 w-8 items-center justify-center bg-primary text-primary-foreground">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="h-5 w-5"
          >
            <title>medxz</title>
            <path d="M12 5v14M5 12h14" />
          </svg>
        </div>
        <span className="text-lg font-medium">medxz</span>
      </div>
      <div className="grid gap-1">
        <div className="flex items-center justify-between">
          <h1 className="text-2xl font-semibold tracking-tight">{props.title}</h1>
          {props.orgIsConfigured ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={props.onChangeOrg}
              className="h-auto px-2 py-1 text-xs text-muted-foreground"
            >
              {props.organizationCode} · change
            </Button>
          ) : null}
        </div>
        <p className="text-sm text-muted-foreground">
          {props.orgIsConfigured
            ? "Enter your credentials to continue"
            : "Enter your organization and credentials"}
        </p>
      </div>
    </div>
  );
}

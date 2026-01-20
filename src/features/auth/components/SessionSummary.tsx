import { Badge } from "@/components/ui/badge";
import type { SessionInfo } from "../../../bindings";

export function SessionSummary(props: { session: SessionInfo }) {
  return (
    <div className="rounded-md border bg-muted/40 p-4">
      <div className="flex items-baseline justify-between gap-4">
        <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
          Organization
        </span>
        <span className="text-sm text-right">
          {props.session.organization.name}{" "}
          <span className="font-mono text-xs text-muted-foreground">
            ({props.session.organization.code})
          </span>
        </span>
      </div>
      <div className="mt-3 flex items-baseline justify-between gap-4 border-t border-dashed border-border/70 pt-3">
        <span className="text-xs font-medium uppercase tracking-wider text-muted-foreground">
          User
        </span>
        <span className="text-sm text-right">
          {props.session.user.email}{" "}
          <Badge variant="secondary" className="ml-2">
            {props.session.user.role}
          </Badge>
        </span>
      </div>
    </div>
  );
}

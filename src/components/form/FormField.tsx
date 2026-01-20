import type { ReactNode } from "react";

import { Label } from "@/components/ui/label";
import { cn } from "@/lib/utils";

export function FormField(props: {
  id: string;
  label: string;
  tip?: string;
  children: ReactNode;
  className?: string;
}) {
  return (
    <div className={cn("grid gap-2", props.className)}>
      <Label htmlFor={props.id}>{props.label}</Label>
      {props.children}
    </div>
  );
}

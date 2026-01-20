import type { ReactNode } from "react";

export function AppShell(props: { children: ReactNode }) {
  return (
    <div className="grid min-h-screen lg:grid-cols-2">
      <div className="hidden bg-muted lg:block">
        <div className="flex h-full flex-col justify-between p-10">
          <div className="flex items-center gap-2 text-lg font-medium">
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
            <span>medxz</span>
          </div>
          <blockquote className="space-y-2">
            <p className="text-lg">
              "Streamlined medical record management for modern healthcare practices."
            </p>
            <footer className="text-sm text-muted-foreground">Healthcare Platform</footer>
          </blockquote>
        </div>
      </div>
      <main className="flex items-center justify-center p-8">
        <div className="w-full max-w-[350px]">{props.children}</div>
      </main>
    </div>
  );
}

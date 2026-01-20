import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { AuthScreen } from "@/features/auth/AuthScreen";
import { useAuthController } from "@/features/auth/useAuthController";
import { Dashboard } from "@/features/dashboard/Dashboard";
import { AppShell } from "./AppShell";

function App() {
  const auth = useAuthController();

  return (
    <>
      <TooltipProvider delayDuration={120}>
        {auth.session ? (
          <Dashboard
            session={auth.session}
            serverUrl={auth.serverUrl}
            onLogout={auth.logout}
            signingOut={auth.submitting}
          />
        ) : (
          <AppShell>
            <AuthScreen auth={auth} />
          </AppShell>
        )}
      </TooltipProvider>
      <Toaster />
    </>
  );
}

export default App;

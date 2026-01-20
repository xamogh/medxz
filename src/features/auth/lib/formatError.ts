import type { AppError } from "../../../bindings";

type AuthErrorContext = "login" | "session" | "logout";

export function formatAuthError(err: AppError, context: AuthErrorContext): string {
  switch (err.type) {
    case "EmptyName":
      return "Name cannot be empty.";
    case "InvalidServerUrl":
      return "Enter a valid server URL.";
    case "Network":
      return context === "session"
        ? "Cannot reach the server to verify your session."
        : "Cannot reach the server. Check the URL and your connection.";
    case "Keychain":
      return "We could not access the system keychain for your session.";
    case "ServerError":
      return formatServerError(err.details.code, context);
  }
}

function formatServerError(code: string, context: AuthErrorContext): string {
  switch (code) {
    case "bad_request":
      return context === "login"
        ? "Please fill out all required fields."
        : "The request was invalid. Please try again.";
    case "unauthorized":
      if (context === "login") return "Incorrect email or password.";
      if (context === "logout") return "Your session has already expired.";
      return "Your session expired. Please sign in again.";
    case "forbidden":
      return "This account is disabled. Contact your administrator.";
    case "not_found":
      return context === "login"
        ? "No account found for that organization and email."
        : "We could not find your account. Please sign in again.";
    case "conflict":
      return "That request conflicts with existing data.";
    case "internal":
      return "The server had a problem. Please try again.";
    default:
      return "Unexpected server error. Please try again.";
  }
}

import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ErrorBoundary } from "./components/ErrorBoundary";
import "./index.css";

window.addEventListener("error", (e) => {
  console.error("[GameSpace] window error:", e.message, e.error);
});
window.addEventListener("unhandledrejection", (e) => {
  console.error("[GameSpace] unhandled rejection:", e.reason);
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ErrorBoundary>
      <App />
    </ErrorBoundary>
  </React.StrictMode>,
);

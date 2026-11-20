import { Component, type ReactNode } from "react";

interface Props {
  children: ReactNode;
}

interface State {
  error: Error | null;
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { error: null };

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: React.ErrorInfo) {
    console.error("[GameSpace] render error:", error, info.componentStack);
  }

  render() {
    if (this.state.error) {
      return (
        <div
          style={{
            minHeight: "100vh",
            background: "#070912",
            color: "#eceef8",
            padding: 32,
            fontFamily: "system-ui, sans-serif",
          }}
        >
          <h1 style={{ fontSize: 20, marginBottom: 12 }}>Render Error</h1>
          <pre
            style={{
              whiteSpace: "pre-wrap",
              color: "#f87171",
              fontSize: 13,
              background: "rgba(255,255,255,0.05)",
              padding: 16,
              borderRadius: 12,
            }}
          >
            {this.state.error.message}
            {"\n"}
            {this.state.error.stack}
          </pre>
          <button
            type="button"
            onClick={() => {
              localStorage.removeItem("gs-settings");
              window.location.hash = "/";
              window.location.reload();
            }}
            style={{
              marginTop: 16,
              padding: "10px 16px",
              borderRadius: 12,
              border: "1px solid rgba(255,255,255,0.15)",
              background: "#7c5cff",
              color: "#fff",
              cursor: "pointer",
            }}
          >
            Reset & Restart
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

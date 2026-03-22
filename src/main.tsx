import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { AppProviders } from "./providers/AppProviders";
import { UsageProvider } from "./context/UsageContext";
import "./styles/global.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <AppProviders>
      <UsageProvider>
        <App />
      </UsageProvider>
    </AppProviders>
  </React.StrictMode>
);

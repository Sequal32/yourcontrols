import "./index.css";

import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/components/App";
import { TooltipProvider } from "@ui/tooltip";
import { Toaster } from "@ui/toaster";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TooltipProvider>
      <App />
      <Toaster />
    </TooltipProvider>
  </React.StrictMode>,
);

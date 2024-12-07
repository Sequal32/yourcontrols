import "./index.css";

import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/components/App";
import { TooltipProvider } from "@/components/ui/tooltip";
import { Toaster } from "@/components/ui/toaster";
import GlobalListener from "@/components/GlobalListener";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TooltipProvider>
      <App />
      <Toaster />
      <GlobalListener />
    </TooltipProvider>
  </React.StrictMode>,
);

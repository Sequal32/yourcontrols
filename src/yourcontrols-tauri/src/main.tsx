import "./index.css";

import React from "react";
import ReactDOM from "react-dom/client";
import App from "@/components/App";
import { TooltipProvider } from "@ui/tooltip";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <TooltipProvider>
      <App />
    </TooltipProvider>
  </React.StrictMode>,
);
